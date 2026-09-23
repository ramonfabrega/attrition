#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["capstone==5.0.3"]
# ///
"""lift.py — original functions, as C.

    uv run tools/recomp/lift.py <install>/riseofnations.exe [--out DIR] <va>…

Each named function, and every function it calls or tail-jumps to directly,
becomes one `void f_<va>(cpu_t *c)` in `<out>/lifted.c`, written against
`rt/recomp.h`, and `<out>/librecomp.dylib` is built from that and `rt/runtime.c`.
The default `<out>` is `target/recomp/` — the output is derived from the
user's executable, like a capture or a decompile, and stays out of git.

The shape is N64Recomp's: one C function per original function, registers
in C locals, the guest's memory a flat reservation, `goto` for every branch,
and a direct C call for every direct `call`. Flags are computed eagerly by
every instruction that defines them; the C compiler drops the ones no branch
reads. The decoder is capstone, declared inline (PEP 723) like `callfn.py`'s
unicorn.

An instruction outside the integer subset below stops the lift and names
itself, with its address — that list is step 3's taxonomy, not something to
guess around. Segment-relative addressing (`fs:`, the TIB) is on it.
"""
import os
import subprocess
import sys

from capstone import CS_ARCH_X86, CS_GRP_CALL, CS_GRP_JUMP, CS_GRP_RET, CS_MODE_32, Cs
from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from image import Functions, Image  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
RT = os.path.join(HERE, "rt")

REG32 = ("eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi")
FLAGS = ("cf", "pf", "af", "zf", "sf", "of", "df")
STRING = {"stosb": 1, "stosw": 2, "stosd": 4, "movsb": 1, "movsw": 2, "movsd": 4}
SUBREG = {}
for _r in REG32:
    _w = _r[1:]  # ax, cx, dx, bx, sp, bp, si, di
    SUBREG[_w] = (_r, 0, 16)
    if _w[1] == "x":
        SUBREG[_w[0] + "l"] = (_r, 0, 8)
        SUBREG[_w[0] + "h"] = (_r, 8, 8)
    else:
        SUBREG[_w + "l"] = (_r, 0, 8)

COND = {
    "o": "of", "no": "!of",
    "b": "cf", "c": "cf", "nae": "cf", "ae": "!cf", "nb": "!cf", "nc": "!cf",
    "e": "zf", "z": "zf", "ne": "!zf", "nz": "!zf",
    "be": "cf || zf", "na": "cf || zf", "a": "!cf && !zf", "nbe": "!cf && !zf",
    "s": "sf", "ns": "!sf", "p": "pf", "pe": "pf", "np": "!pf", "po": "!pf",
    "l": "sf != of", "nge": "sf != of", "ge": "sf == of", "nl": "sf == of",
    "le": "zf || sf != of", "ng": "zf || sf != of", "g": "!zf && sf == of", "nle": "!zf && sf == of",
}  # each is used whole inside `if (…)`; a `cmov`/`set` wraps it itself
TERMINATORS = ("ret", "int3", "hlt", "ud2")

SAVE = "; ".join(f"c->{r} = {r}" for r in REG32 + FLAGS) + ";"
LOAD = "; ".join(f"{r} = c->{r}" for r in REG32 + FLAGS) + ";"


class LiftError(Exception):
    pass


XMM = {f"xmm{i}": i for i in range(8)}
SSE_SS = {"addss": "ss_add", "subss": "ss_sub", "mulss": "ss_mul", "divss": "ss_div", "minss": "ss_min", "maxss": "ss_max"}
SSE_SD = {"addsd": "sd_add", "subsd": "sd_sub", "mulsd": "sd_mul", "divsd": "sd_div", "minsd": "sd_min", "maxsd": "sd_max"}
BITWISE = {"xorps": "^", "xorpd": "^", "pxor": "^", "andps": "&", "andpd": "&", "pand": "&", "orps": "|", "orpd": "|", "por": "|",
           "andnps": "&~", "andnpd": "&~", "pandn": "&~"}
MOV128 = ("movaps", "movups", "movapd", "movupd", "movdqa", "movdqu")


class Lifter:
    def __init__(self, image, funcs):
        self.image, self.funcs = image, funcs
        self.imports = {iat: name for iat, _, name in image.imports()}  # IAT slot -> import name
        self.md = Cs(CS_ARCH_X86, CS_MODE_32)
        self.md.detail = True
        self.lifted = {}  # entry -> C text
        self.tables = {}  # a `jmp [4*reg + table]`'s address -> (reg, its targets)

    def import_slot(self, i, o):
        """The import an absolute `[slot]` operand names, or None."""
        if o.type == X86_OP_MEM and not o.mem.base and not o.mem.index:
            return self.imports.get(o.mem.disp & 0xFFFFFFFF)
        return None

    # ---- XMM operands: the registers live in `c`, not in locals ----

    def xmm(self, i, o):
        return f"c->xmm[{XMM[i.reg_name(o.reg)]}]"

    def x128(self, i, o):
        return self.xmm(i, o) if o.type == X86_OP_REG else f"ld128(m, {self.addr(i, o)})"

    def x32(self, i, o):
        if o.type == X86_OP_REG:
            return f"{self.xmm(i, o)}.d[0]" if i.reg_name(o.reg) in XMM else self.reg_read(i.reg_name(o.reg))
        return f"ld32(m, {self.addr(i, o)})"

    def x64(self, i, o):
        return f"lane64({self.xmm(i, o)}, 0)" if o.type == X86_OP_REG else f"ld64(m, {self.addr(i, o)})"

    # ---- discovery: one function's instructions, by following its branches ----

    def decode(self, va):
        i = next(self.md.disasm(self.image.read(va, 16), va, count=1), None)
        if i is None:
            raise LiftError(f"{va:08x}: undecodable")
        return i

    def jump_table(self, i, trail):
        """`jmp [4*reg + table]` after `cmp reg, N` in the same run: the N+1
        targets read from the image. None if the shape is not that."""
        o = i.operands[0]
        m = o.mem
        if o.type != X86_OP_MEM or m.base or not m.index or m.scale != 4:
            return None
        reg = i.reg_name(m.index)
        table = m.disp & 0xFFFFFFFF
        bound_reg, first = reg, None  # the register the `cmp` bounds, and a byte table it indexes
        for p in reversed(trail):
            if p.mnemonic == "movzx" and p.operands[0].type == X86_OP_REG and i.reg_name(p.operands[0].reg) == bound_reg \
                    and p.operands[1].type == X86_OP_MEM and p.operands[1].size == 1 and p.operands[1].mem.base \
                    and not p.operands[1].mem.index and first is None:
                # MSVC's two-level table: `movzx idx, byte [orig + first]` before `jmp [4*idx + table]`
                first = p.operands[1].mem.disp & 0xFFFFFFFF
                bound_reg = i.reg_name(p.operands[1].mem.base)
            elif p.mnemonic == "cmp" and p.operands[0].type == X86_OP_REG and i.reg_name(p.operands[0].reg) == bound_reg \
                    and p.operands[1].type == X86_OP_IMM:
                n = (p.operands[1].imm & 0xFFFFFFFF) + 1
                if n > 4096:
                    return None
                if first is not None:
                    n = max(self.image.read(first, n)) + 1
                targets = [self.image.u32(table + 4 * k) for k in range(n)]
                if all(self.image.is_code(t) for t in targets):
                    return reg, targets
                return None
        return None

    def discover(self, entry):
        insns, targets, callees = {}, set(), set()
        bound = self.funcs.next_after(entry)
        work = [entry]
        while work:
            va = work.pop()
            trail = []  # this run's instructions, for the `cmp` that bounds a jump table
            while va not in insns:
                if bound is not None and va >= bound:
                    raise LiftError(f"{entry:08x}: runs past {bound:08x} ({self.funcs.name(bound)})")
                if len(insns) > 50_000:
                    raise LiftError(f"{entry:08x}: over 50k instructions; discovery is looping")
                i = self.decode(va)
                insns[va] = i
                trail.append(i)
                nxt = i.address + i.size
                if i.mnemonic in TERMINATORS:
                    break
                if i.group(CS_GRP_JUMP):
                    o = i.operands[0]
                    if o.type != X86_OP_IMM:
                        jt = self.jump_table(i, trail) if i.mnemonic == "jmp" else None
                        if jt:
                            self.tables[i.address] = jt
                            targets.update(jt[1])
                            work.extend(jt[1])
                        break  # otherwise indirect: rc_dispatch decides at run time
                    t = o.imm & 0xFFFFFFFF
                    if i.mnemonic == "jmp" and t != entry and t in self.funcs.by_va:
                        callees.add(t)  # a tail call into another function
                        break
                    targets.add(t)
                    work.append(t)
                    if i.mnemonic == "jmp":
                        break
                elif i.group(CS_GRP_CALL):
                    o = i.operands[0]
                    if o.type == X86_OP_IMM:
                        callees.add(o.imm & 0xFFFFFFFF)
                va = nxt
        return insns, targets, callees

    # ---- operands ----

    @staticmethod
    def reg_read(name):
        if name in REG32:
            return name
        base, sh, bits = SUBREG[name]
        mask = (1 << bits) - 1
        return f"(({base} >> {sh}) & {mask:#x}u)" if sh else f"({base} & {mask:#x}u)"

    @staticmethod
    def reg_write(name, v):
        if name in REG32:
            return f"{name} = (uint32_t)({v});"
        base, sh, bits = SUBREG[name]
        mask = (1 << bits) - 1
        keep = ~(mask << sh) & 0xFFFFFFFF
        return f"{base} = ({base} & {keep:#x}u) | ((uint32_t)({v}) & {mask:#x}u) << {sh};"

    def addr(self, i, o):
        m = o.mem
        parts = []
        if m.segment:
            seg = i.reg_name(m.segment)
            if seg == "fs":
                parts.append("c->fs_base")  # the guest TIB: SEH registration, TLS
            elif seg not in ("es", "ds", "ss", "cs"):  # the flat segments
                raise LiftError(f"{i.address:08x}: {i.mnemonic} {i.op_str} — segment-relative addressing ({seg})")
        if m.base:
            parts.append(self.reg_read(i.reg_name(m.base)))
        if m.index:
            parts.append(f"{self.reg_read(i.reg_name(m.index))} * {m.scale}")
        disp = m.disp & 0xFFFFFFFF
        if disp or not parts:
            parts.append(f"{disp:#x}u")
        return "(uint32_t)(" + " + ".join(parts) + ")"

    def rd(self, i, o):
        if o.type == X86_OP_REG:
            return self.reg_read(i.reg_name(o.reg))
        if o.type == X86_OP_IMM:
            return f"{o.imm & ((1 << (8 * o.size)) - 1):#x}u"
        if o.type == X86_OP_MEM:
            return f"ld{8 * o.size}(m, {self.addr(i, o)})"
        raise LiftError(f"{i.address:08x}: {i.mnemonic} {i.op_str} — operand kind {o.type}")

    def wr(self, i, o, v):
        if o.type == X86_OP_REG:
            return self.reg_write(i.reg_name(o.reg), v)
        if o.type == X86_OP_MEM:
            return f"st{8 * o.size}(m, {self.addr(i, o)}, {v});"
        raise LiftError(f"{i.address:08x}: {i.mnemonic} {i.op_str} — write to operand kind {o.type}")

    # ---- flags: x86's, for an operation of `bits` width on a, b, r ----

    @staticmethod
    def flags_zsp(bits):
        return [f"zf = (r == 0); sf = (r >> {bits - 1}) & 1; pf = parity8(r);"]

    def flags_add(self, bits):
        return [f"cf = (w >> {bits}) & 1; of = (((a ^ r) & (b ^ r)) >> {bits - 1}) & 1; af = ((a ^ b ^ r) >> 4) & 1;"] + self.flags_zsp(bits)

    def flags_sub(self, bits):
        return [f"cf = (w >> {bits}) & 1; of = (((a ^ b) & (a ^ r)) >> {bits - 1}) & 1; af = ((a ^ b ^ r) >> 4) & 1;"] + self.flags_zsp(bits)

    def flags_logic(self, bits):
        return ["cf = 0; of = 0;"] + self.flags_zsp(bits)

    # ---- one instruction ----

    def emit(self, i, entry, insns):
        mn, ops = i.mnemonic, i.operands
        out = []
        bits = 8 * ops[0].size if ops else 32
        mask = (1 << bits) - 1
        nxt = (i.address + i.size) & 0xFFFFFFFF

        def unsupported(why=""):
            raise LiftError(f"{i.address:08x}: {mn} {i.op_str}" + (f" — {why}" if why else ""))

        def binop(kind, write=True):
            out.append(f"a = {self.rd(i, ops[0])}; b = {self.rd(i, ops[1])};")
            if kind in ("add", "adc"):
                out.append(f"w = (uint64_t)a + b{' + cf' if kind == 'adc' else ''}; r = (uint32_t)w & {mask:#x}u;")
                out.extend(self.flags_add(bits))
            elif kind in ("sub", "sbb", "cmp"):
                out.append(f"w = (uint64_t)a - b{' - cf' if kind == 'sbb' else ''}; r = (uint32_t)w & {mask:#x}u;")
                out.extend(self.flags_sub(bits))
            else:
                op = {"and": "&", "test": "&", "or": "|", "xor": "^"}[kind]
                out.append(f"r = (a {op} b) & {mask:#x}u;")
                out.extend(self.flags_logic(bits))
            if write:
                out.append(self.wr(i, ops[0], "r"))

        if mn in ("add", "adc", "sub", "sbb", "and", "or", "xor"):
            binop(mn)
        elif mn in ("cmp", "test"):
            binop(mn, write=False)
        elif mn in ("inc", "dec"):
            out.append(f"a = {self.rd(i, ops[0])}; b = 1;")
            out.append(f"w = (uint64_t)a {'+' if mn == 'inc' else '-'} b; r = (uint32_t)w & {mask:#x}u;")
            out += [s.replace("cf = (w >> %d) & 1; " % bits, "") for s in (self.flags_add if mn == "inc" else self.flags_sub)(bits)]
            out.append(self.wr(i, ops[0], "r"))
        elif mn == "neg":
            out.append(f"a = 0; b = {self.rd(i, ops[0])}; w = (uint64_t)a - b; r = (uint32_t)w & {mask:#x}u;")
            out += self.flags_sub(bits)
            out.append(self.wr(i, ops[0], "r"))
        elif mn == "not":
            out.append(self.wr(i, ops[0], f"~{self.rd(i, ops[0])}"))
        elif mn in ("mov", "movzx"):
            out.append(self.wr(i, ops[0], self.rd(i, ops[1])))
        elif mn == "movsx":
            sb = 8 * ops[1].size
            out.append(self.wr(i, ops[0], f"(uint32_t)(int32_t)(int{sb}_t){self.rd(i, ops[1])}"))
        elif mn == "lea":
            out.append(self.wr(i, ops[0], self.addr(i, ops[1])))
        elif mn in ("shl", "sal", "shr", "sar"):
            out.append(f"a = {self.rd(i, ops[0])}; n = {self.rd(i, ops[1])} & 31;")
            out.append("if (n) {")
            if mn in ("shl", "sal"):
                out.append(f"  r = (a << n) & {mask:#x}u; cf = (a >> ({bits} - n)) & 1; of = ((r >> {bits - 1}) & 1) ^ cf;")
            elif mn == "shr":
                out.append(f"  r = (a >> n) & {mask:#x}u; cf = (a >> (n - 1)) & 1; of = (a >> {bits - 1}) & 1;")
            else:
                out.append(f"  r = (uint32_t)(((int32_t)(a << {32 - bits})) >> ({32 - bits} + n)) & {mask:#x}u; cf = (a >> (n - 1)) & 1; of = 0;")
            out += ["  " + s for s in self.flags_zsp(bits)]
            out.append("  " + self.wr(i, ops[0], "r"))
            out.append("}")
        elif mn == "imul":
            if bits != 32:
                unsupported("only the 32-bit forms")
            if len(ops) == 1:
                out.append(f"sw = (int64_t)(int32_t)eax * (int32_t){self.rd(i, ops[0])}; eax = (uint32_t)sw; edx = (uint32_t)(sw >> 32);")
                out.append("cf = of = (sw != (int64_t)(int32_t)eax);")
            else:
                src, mul = (ops[0], ops[1]) if len(ops) == 2 else (ops[1], ops[2])
                out.append(f"sw = (int64_t)(int32_t){self.rd(i, src)} * (int32_t){self.rd(i, mul)}; r = (uint32_t)sw;")
                out.append("cf = of = (sw != (int64_t)(int32_t)r);")
                out.append(self.wr(i, ops[0], "r"))
        elif mn == "mul":
            if bits != 32:
                unsupported("only the 32-bit form")
            out.append(f"w = (uint64_t)eax * {self.rd(i, ops[0])}; eax = (uint32_t)w; edx = (uint32_t)(w >> 32); cf = of = (edx != 0);")
        elif mn == "div":
            if bits != 32:
                unsupported("only the 32-bit form")
            out.append(f"b = {self.rd(i, ops[0])}; w = ((uint64_t)edx << 32) | eax;")
            out.append(f'if (b == 0 || w / b > 0xffffffffull) rc_trap(c, {i.address:#010x}u, "div: #DE");')
            out.append("eax = (uint32_t)(w / b); edx = (uint32_t)(w % b);")
        elif mn == "idiv":
            if bits != 32:
                unsupported("only the 32-bit form")
            out.append(f"b = {self.rd(i, ops[0])}; sw = (int64_t)(((uint64_t)edx << 32) | eax);")
            out.append(f'if (b == 0 || (sw == INT64_MIN && (int32_t)b == -1) || sw / (int32_t)b != (int32_t)(sw / (int32_t)b)) rc_trap(c, {i.address:#010x}u, "idiv: #DE");')
            out.append("eax = (uint32_t)(sw / (int32_t)b); edx = (uint32_t)(sw % (int32_t)b);")
        elif mn in ("bt", "bts", "btr", "btc"):
            # a register offset is signed and, on memory, addresses past the operand; an immediate is modulo the width
            if ops[1].type == X86_OP_IMM:
                out.append(f"n = {self.rd(i, ops[1])} & {bits - 1}; t = {self.addr(i, ops[0]) if ops[0].type == X86_OP_MEM else '0'};")
            else:
                out.append(f"b = {self.rd(i, ops[1])}; n = b & {bits - 1};")
                adj = f" + (uint32_t)(((int32_t)b >> {bits.bit_length() - 1}) * {bits // 8})" if ops[0].type == X86_OP_MEM else ""
                out.append(f"t = {self.addr(i, ops[0]) if ops[0].type == X86_OP_MEM else '0'}{adj};")
            out.append(f"a = {f'ld{bits}(m, t)' if ops[0].type == X86_OP_MEM else self.rd(i, ops[0])}; cf = (a >> n) & 1;")
            if mn != "bt":
                r = {"bts": "a | (1u << n)", "btr": "a & ~(1u << n)", "btc": "a ^ (1u << n)"}[mn]
                out.append(f"st{bits}(m, t, {r});" if ops[0].type == X86_OP_MEM else self.wr(i, ops[0], r))
        elif mn in ("rol", "ror"):
            out.append(f"a = {self.rd(i, ops[0])}; n = ({self.rd(i, ops[1])} & 31) % {bits};")
            out.append("if (n) {")
            if mn == "rol":
                out.append(f"  r = ((a << n) | (a >> ({bits} - n))) & {mask:#x}u; cf = r & 1; of = ((r >> {bits - 1}) & 1) ^ cf;")
            else:
                out.append(f"  r = ((a >> n) | (a << ({bits} - n))) & {mask:#x}u; cf = (r >> {bits - 1}) & 1; of = cf ^ ((r >> {bits - 2}) & 1);")
            out.append("  " + self.wr(i, ops[0], "r"))
            out.append("}")
        elif mn.split()[-1] in STRING and (not mn.endswith("movsd") or (ops[0].type == X86_OP_MEM and ops[1].type == X86_OP_MEM)):
            rep, _, base = mn.rpartition(" ")
            size = STRING[base]
            step = f"(df ? (uint32_t)-{size} : {size}u)"
            if base.startswith("stos"):
                body = f"st{8 * size}(m, edi, eax & {(1 << (8 * size)) - 1:#x}u); edi += {step};"
            else:
                body = f"st{8 * size}(m, edi, ld{8 * size}(m, esi)); esi += {step}; edi += {step};"
            if rep:
                if rep != "rep":
                    unsupported(f"{rep} prefix")
                out.append(f"while (ecx) {{ {body} ecx--; }}")
            else:
                out.append(body)
        elif mn in ("cld", "std"):
            out.append(f"df = {int(mn == 'std')};")
        elif mn == "cdq":
            out.append("edx = (uint32_t)((int32_t)eax >> 31);")
        elif mn == "cwde":
            out.append("eax = (uint32_t)(int32_t)(int16_t)eax;")
        elif mn == "push":
            if ops[0].size != 4:
                unsupported("only the 32-bit form")
            out.append(f"t = {self.rd(i, ops[0])}; esp -= 4; st32(m, esp, t);")
        elif mn == "pop":
            if ops[0].size != 4:
                unsupported("only the 32-bit form")
            out.append("t = ld32(m, esp); esp += 4;")
            out.append(self.wr(i, ops[0], "t"))
        elif mn == "nop":
            pass
        elif mn == "call":
            o = ops[0]
            out.append(f"esp -= 4; st32(m, esp, {nxt:#010x}u);")
            if o.type == X86_OP_IMM:
                out.append(f"{SAVE} f_{o.imm & 0xFFFFFFFF:08x}(c); {LOAD}")
            elif self.import_slot(i, o):
                out.append(f'{SAVE} rc_import(c, "{self.import_slot(i, o)}"); {LOAD}')
            else:
                out.append(f"t = {self.rd(i, o)}; {SAVE} rc_dispatch(c, t); {LOAD}")
            out.append(f"if (c->eip != {nxt:#010x}u) rc_badret(c, {i.address:#010x}u, {nxt:#010x}u);")
        elif mn == "ret":
            adj = 4 + (ops[0].imm if ops else 0)
            out.append(f"t = ld32(m, esp); esp += {adj}; c->eip = t; {SAVE} return;")
        elif mn == "jmp":
            o = ops[0]
            if o.type == X86_OP_IMM:
                t = o.imm & 0xFFFFFFFF
                if t in insns:
                    out.append(f"goto L_{t:08x};")
                else:
                    out.append(f"{SAVE} f_{t:08x}(c); return;")  # a tail call
            elif self.import_slot(i, o):
                out.append(f'{SAVE} rc_import(c, "{self.import_slot(i, o)}"); return;')  # an import thunk
            elif i.address in self.tables:
                reg, tgts = self.tables[i.address]
                out.append(f"switch ({self.reg_read(reg)}) {{")
                out += [f"  case {k}: goto L_{t:08x};" for k, t in enumerate(tgts)]
                out.append(f'  default: rc_trap(c, {i.address:#010x}u, "jump table index out of range");')
                out.append("}")
            else:
                out.append(f"t = {self.rd(i, o)}; {SAVE} rc_dispatch(c, t); return;")
        elif mn[0] == "j" and mn[1:] in COND:
            out.append(f"if ({COND[mn[1:]]}) goto L_{ops[0].imm & 0xFFFFFFFF:08x};")
        elif mn.startswith("cmov") and mn[4:] in COND:
            out.append(f"if ({COND[mn[4:]]}) {self.wr(i, ops[0], self.rd(i, ops[1]))}")
        elif mn.startswith("set") and mn[3:] in COND:
            out.append(self.wr(i, ops[0], f"(uint32_t)({COND[mn[3:]]})"))
        elif mn in TERMINATORS:
            out.append(f'rc_trap(c, {i.address:#010x}u, "{mn}");')
        # ---- SSE, scalar and the 128-bit moves; the registers live in c->xmm ----
        elif mn in ("movss", "movsd"):
            d, s = ops
            if d.type == X86_OP_MEM:
                out.append(f"st32(m, {self.addr(i, d)}, {self.x32(i, s)});" if mn == "movss" else f"st64(m, {self.addr(i, d)}, {self.x64(i, s)});")
            elif s.type == X86_OP_MEM:  # a load zeroes the upper lanes
                if mn == "movss":
                    out.append(f"{self.xmm(i, d)}.d[0] = {self.x32(i, s)}; {self.xmm(i, d)}.d[1] = {self.xmm(i, d)}.d[2] = {self.xmm(i, d)}.d[3] = 0;")
                else:
                    out.append(f"setlane64(&{self.xmm(i, d)}, 0, {self.x64(i, s)}); setlane64(&{self.xmm(i, d)}, 1, 0);")
            else:  # register to register keeps them
                out.append(f"{self.xmm(i, d)}.d[0] = {self.x32(i, s)};" if mn == "movss" else f"setlane64(&{self.xmm(i, d)}, 0, {self.x64(i, s)});")
        elif mn in MOV128:
            d, s = ops
            out.append(f"{self.xmm(i, d)} = {self.x128(i, s)};" if d.type == X86_OP_REG else f"st128(m, {self.addr(i, d)}, {self.x128(i, s)});")
        elif mn == "movd":
            d, s = ops
            if d.type == X86_OP_REG and i.reg_name(d.reg) in XMM:
                out.append(f"{self.xmm(i, d)}.d[0] = {self.rd(i, s)}; {self.xmm(i, d)}.d[1] = {self.xmm(i, d)}.d[2] = {self.xmm(i, d)}.d[3] = 0;")
            else:
                out.append(self.wr(i, d, f"{self.xmm(i, s)}.d[0]"))
        elif mn == "movq":
            d, s = ops
            if d.type == X86_OP_REG:
                out.append(f"setlane64(&{self.xmm(i, d)}, 0, {self.x64(i, s)}); setlane64(&{self.xmm(i, d)}, 1, 0);")
            else:
                out.append(f"st64(m, {self.addr(i, d)}, {self.x64(i, s)});")
        elif mn in SSE_SS:
            d, s = ops
            out.append(f"{self.xmm(i, d)}.d[0] = {SSE_SS[mn]}({self.xmm(i, d)}.d[0], {self.x32(i, s)});")
        elif mn in SSE_SD:
            d, s = ops
            out.append(f"setlane64(&{self.xmm(i, d)}, 0, {SSE_SD[mn]}(lane64({self.xmm(i, d)}, 0), {self.x64(i, s)}));")
        elif mn == "sqrtss":
            d, s = ops
            out.append(f"{self.xmm(i, d)}.d[0] = ss_sqrt({self.x32(i, s)});")
        elif mn == "sqrtsd":
            d, s = ops
            out.append(f"setlane64(&{self.xmm(i, d)}, 0, sd_sqrt({self.x64(i, s)}));")
        elif mn in BITWISE:
            d, s = ops
            op = BITWISE[mn]
            out.append(f"xs = {self.x128(i, s)};")
            out.append(" ".join(f"{self.xmm(i, d)}.d[{j}] = {'~' if op == '&~' else ''}{self.xmm(i, d)}.d[{j}] {op[0]} xs.d[{j}];" for j in range(4)))
        elif mn in ("ucomiss", "comiss", "ucomisd", "comisd"):
            d, s = ops
            cmp = f"ucom32({self.x32(i, d)}, {self.x32(i, s)})" if mn.endswith("ss") else f"ucom64({self.x64(i, d)}, {self.x64(i, s)})"
            out.append(f"t = {cmp}; cf = t & 1; pf = (t >> 2) & 1; zf = (t >> 6) & 1; of = sf = af = 0;")
        elif mn in ("cvttss2si", "cvtss2si", "cvttsd2si", "cvtsd2si"):
            d, s = ops
            fn = {"cvttss2si": "cvtt_ss", "cvtss2si": "cvt_ss", "cvttsd2si": "cvtt_sd", "cvtsd2si": "cvt_sd"}[mn]
            src = self.x32(i, s) if mn.endswith("ss2si") else self.x64(i, s)
            out.append(self.wr(i, d, f"{fn}({src})"))
        elif mn == "cvtsi2ss":
            d, s = ops
            out.append(f"{self.xmm(i, d)}.d[0] = cvt_si_ss({self.rd(i, s)});")
        elif mn == "cvtsi2sd":
            d, s = ops
            out.append(f"setlane64(&{self.xmm(i, d)}, 0, cvt_si_sd({self.rd(i, s)}));")
        elif mn == "cvtss2sd":
            d, s = ops
            out.append(f"setlane64(&{self.xmm(i, d)}, 0, cvt_ss_sd({self.x32(i, s)}));")
        elif mn == "cvtsd2ss":
            d, s = ops
            out.append(f"{self.xmm(i, d)}.d[0] = cvt_sd_ss({self.x64(i, s)});")
        elif mn == "lahf":
            out.append("eax = (eax & 0xffff00ffu) | (((sf << 7) | (zf << 6) | (af << 4) | (pf << 2) | 2 | cf) << 8);")
        elif mn == "sahf":
            out.append("t = (eax >> 8) & 0xff; sf = (t >> 7) & 1; zf = (t >> 6) & 1; af = (t >> 4) & 1; pf = (t >> 2) & 1; cf = t & 1;")
        else:
            unsupported()
        return out

    # ---- one function ----

    def lift(self, entry):
        insns, targets, callees = self.discover(entry)
        lines = [f"/* {self.funcs.name(entry)} */", f"void f_{entry:08x}(cpu_t *c) {{"]
        lines.append("    uint8_t *m = c->mem; uint32_t a, b, r, t, n; uint64_t w; int64_t sw; x128 xs; (void)xs;")
        lines.append("    " + "; ".join(f"uint32_t {r} = c->{r}" for r in REG32) + ";")
        lines.append("    " + "; ".join(f"uint32_t {r} = c->{r}" for r in FLAGS) + ";")
        lines.append("    (void)a; (void)b; (void)r; (void)t; (void)n; (void)w; (void)sw; (void)m;")
        for va in sorted(insns):
            i = insns[va]
            if va in targets:
                lines.append(f"L_{va:08x}:")
            lines.append(f"    /* {va:08x}  {i.mnemonic} {i.op_str} */")
            lines += ["    " + s for s in self.emit(i, entry, insns)]
        lines.append(f'    rc_trap(c, {entry:#010x}u, "fell off the end");')
        lines.append("}")
        self.lifted[entry] = "\n".join(lines)
        return callees

    def lift_all(self, entries):
        """Lift the entries and their direct callees; a function that cannot
        be lifted becomes a stub that traps with the reason, so a set builds
        and a run names what it is missing."""
        self.stubbed = {}
        work = list(entries)
        while work:
            e = work.pop()
            if e in self.lifted:
                continue
            try:
                callees = self.lift(e)
            except LiftError as err:
                self.stubbed[e] = str(err)
                why = str(err).replace("\\", "\\\\").replace('"', '\\"')
                self.lifted[e] = f'/* {self.funcs.name(e)}: not lifted */\nvoid f_{e:08x}(cpu_t *c) {{ rc_trap(c, {e:#010x}u, "not lifted: {why}"); }}'
                continue
            for callee in callees:
                if callee not in self.lifted:
                    work.append(callee)

    def c_source(self):
        entries = sorted(self.lifted)
        parts = ['#include "recomp.h"', "#include <stdint.h>", ""]
        parts += [f"void f_{e:08x}(cpu_t *c);" for e in entries]
        parts.append("")
        parts += [self.lifted[e] for e in entries]
        parts.append("")
        parts.append("void rc_dispatch(cpu_t *c, uint32_t target) {")
        parts.append("    switch (target) {")
        parts += [f"    case {e:#010x}u: f_{e:08x}(c); return;" for e in entries]
        parts.append('    default: rc_trap(c, target, "no lifted function at this address");')
        parts.append("    }")
        parts.append("}")
        parts.append("")
        parts.append(f"const uint32_t rc_lifted_n = {len(entries)};")
        parts.append("const uint32_t rc_lifted[] = {" + ", ".join(f"{e:#010x}u" for e in entries) + "};")
        return "\n".join(parts) + "\n"


def build(out_dir, c_path):
    lib = os.path.join(out_dir, "librecomp.dylib")
    cmd = ["clang", "-std=c11", "-O2", "-ffp-contract=off", "-fno-strict-aliasing", "-Wall", "-Wextra",
           "-Wno-unused-parameter", "-shared", "-fPIC", "-I", RT, "-o", lib, c_path, os.path.join(RT, "runtime.c")]
    subprocess.run(cmd, check=True)
    return lib


def main(argv):
    args = argv[1:]
    out_dir = os.path.join(HERE, "..", "..", "target", "recomp")
    if "--out" in args:
        k = args.index("--out")
        out_dir = args[k + 1]
        del args[k:k + 2]
    if len(args) < 2:
        print(__doc__)
        return 2
    exe, entries = args[0], [int(a, 16) for a in args[1:]]
    image = Image(exe)
    funcs = Functions(image, os.path.dirname(os.path.abspath(exe)))
    lifter = Lifter(image, funcs)
    lifter.lift_all(entries)
    os.makedirs(out_dir, exist_ok=True)
    c_path = os.path.join(out_dir, "lifted.c")
    with open(c_path, "w") as f:
        f.write(lifter.c_source())
    lib = build(out_dir, c_path)
    print(f"{len(lifter.lifted)} functions -> {c_path} -> {lib}")
    for e in sorted(lifter.lifted):
        print(f"  {funcs.name(e)}@{e:08x}" + (f"  NOT LIFTED: {lifter.stubbed[e]}" if e in lifter.stubbed else ""))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except LiftError as e:
        print(f"lift: {e}", file=sys.stderr)
        sys.exit(1)
