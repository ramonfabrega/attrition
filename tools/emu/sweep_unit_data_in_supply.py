#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_in_supply.py — `UnitData::in_supply@00609ef0` under unicorn (item 2217, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_data_in_supply.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 32-byte object of random bytes (no bias), 2 stack argument(s);
the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; argument 1 names 4 random bytes; callee 0x0073aba0 (3 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `c1+0x134` points at 28 random bytes less `(i32(c[1],0)*i32(c[0],4*((i32(m,0x14)^0x63637)>>8))+i32(c[0],4*((i32(m,0x10)^0x63637)>>8)))*0x1c` (placed); stub 0 answered from [-1, 0, 1, 5, -5] (90%).
One row per input, `<args… stubs…> | <this>;<c…>;<p…> -> <eax> | <this>;<c…>;<p…>` (hex); the stubs'
answers follow the arguments, and the test holds every byte of every window.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x00609ef0
WIN = 32
OWN = 2
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = [(13297148, 1024), (12607952, 320)]
AT = []
PTRARGS = [(1, 4)]
STUBS = [('call', 7580576, 3)]
PLACES = [('m', '0x18', '0', 544), ('c1', '0x134', '(i32(c[1],0)*i32(c[0],4*((i32(m,0x14)^0x63637)>>8))+i32(c[0],4*((i32(m,0x10)^0x63637)>>8)))*0x1c', 28)]
WBIAS = [('c1', 0, ('set', [21])), ('m', 16, ('set', [407095, 407482, 406829, 406160, 405507, 406006, 409465, 408812, 408159, 408514, 407989, 403256, 402603, 401950, 402305, 401780, 405223, 404586, 404957, 404288, 403763, 399014, 398377, 398748, 398095, 397554, 400997, 401384, 400731, 400078, 399537, 394788, 395159, 394522, 393869, 393328, 393699, 397142, 396505, 395852, 395327, 407095, 407240, 407351, 406728, 406839, 395336])), ('m', 20, ('set', [407095, 407482, 406829, 406160, 405507, 406006, 409465, 408812, 408159, 408514, 407989, 403256, 402603, 401950, 402305, 401780, 405223, 404586, 404957, 404288, 403763, 399014, 398377, 398748, 398095, 397554, 400997, 401384, 400731, 400078, 399537, 394788, 395159, 394522, 393869, 393328, 393699, 397142, 396505, 395852, 395327, 407095, 407240, 407351, 406728, 406839, 395336])), ('m', 8, ('set', [0, 256, 512, 32512, 65280])), ('p1', 536, ('set', [0, 0, 0, 1, 2])), ('p2', 12, ('set', [0, 16777216, 4278190080, 33554432, 2130706432, 2147483648]))]
FILLS = [('c0', 'i//3')]
ARGRANGE = []
SLOTBASE = []
ECXINT = None
ANSWERS = [(0, [-1, 0, 1, 5, -5])]
VTAT = None
ANSWERWIN = []
CDECL = False
ARGEQ = []
SEQ = []
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def i32(w, off):
    return int.from_bytes(w[off:off + 4], "little", signed=True)


def i16(w, off):
    return int.from_bytes(w[off:off + 2], "little", signed=True)


def i8(w, off):
    return int.from_bytes(w[off:off + 1], "little", signed=True)


def u16(w, off):
    return int.from_bytes(w[off:off + 2], "little")


def u8(w, off):
    return w[off]


def wdraw(rng, vals):
    """A bias's value: a set's member, or a range's — its own edges and the
    small integers inside it three rows in ten, so a boundary inside a wide
    range is drawn on purpose rather than by luck (Loop 2003, item 2181)."""
    if vals[0] == "set":
        return rng.choice(vals[1])
    lo, hi = vals[1], vals[2]
    if rng.random() < 0.3:
        return rng.choice([v for v in (lo, lo + 1, -1, 0, 1, 2, hi - 1, hi) if lo <= v <= hi])
    return rng.randint(lo, hi)


def shape(rng, name, buf):
    """The window `name`'s fills, then its biases, every row (parked 2113)."""
    for win, expr in FILLS:
        if win == name:
            for i in range(len(buf) // 4):
                buf[4 * i:4 * i + 4] = (eval(expr, {"i": i}) & 0xFFFFFFFF).to_bytes(4, "little")
    for win, off, vals in WBIAS:
        if win != name:
            continue
        for o in (range(0, len(buf) - 3, 4) if off is None else (off,)):
            v = wdraw(rng, vals)
            buf[o:o + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")


def main(argv):
    from hooks import Harness
    rng = random.Random(2217)
    h = Harness(argv[1], {a: (f"{a:x}", n) for k, a, n in STUBS if k == "call"})
    uc = h.m.uc
    slots = {off: (f"slot{off:x}", n, 0) for k, off, n in STUBS if k == "slot"}
    vt = h.vtable(slots) if slots else None
    names = [f"slot{a:x}" if k == "slot" else f"{a:x}" for k, a, _ in STUBS]
    stubbed = {off: h.r32(vt + off) & 0xFFFFFFFF for off, _ in SLOTBASE}
    base = h.brk
    for i in range(220):
        h.brk = base  # the bump heap is reused a row; 220 rows of windows outgrow it
        this = h.alloc(WIN + 0x20)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        for off, vals in BIAS:
            v = rng.choice(vals) if rng.random() < 0.9 else rng.randint(-0x80000000, 0x7FFFFFFF)
            mem[off:off + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")
        shape(rng, "m", mem)
        if vt is not None and VTAT is None:
            mem[0:4] = vt.to_bytes(4, "little")
        args = [rand32(rng) for _ in range(OWN)]
        for idx, vals in ARGSET:
            args[idx] = rng.choice(vals) if rng.random() < 0.9 else args[idx]
        for idx, off, size in ARGFROM:
            if rng.random() < 0.45:
                args[idx] = int.from_bytes(mem[off:off + size], "little", signed=True)
        for idx, lo, hi in ARGRANGE:
            args[idx] = rng.randint(lo, hi)
        wins = []
        for va, size in CELLS:
            rec = h.alloc(size + 0x20)
            buf = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(rec, bytes(buf))
            uc.mem_write(va, rec.to_bytes(4, "little"))
            wins.append((rec, buf))
        for va, size in AT:
            buf = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(va, bytes(buf))
            wins.append((va, buf))
        for idx, size in PTRARGS:
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(buf, bytes(b))
            args[idx] = buf
            wins.append((buf, b))
        nc = len(CELLS) + len(AT)
        c = [b for _, b in wins[:nc]]
        p = [b for _, b in wins[nc:]]
        for k, b in enumerate(c):
            shape(rng, f"c{k}", b)
        for k, b in enumerate(p):
            shape(rng, f"p{k}", b)
        # The stubs' answers are drawn before the windows are placed, so a
        # placed table may be indexed by one (`s`, item 2160); `--answers`
        # draws stub IDX's from a list (parked 2141).
        answers = [rand32(rng) for _ in STUBS]
        for idx, vals in ANSWERS:
            answers[idx] = rng.choice(vals) if rng.random() < 0.9 else answers[idx]
        for win, loc, expr, size in PLACES:
            # A table pointer laid as `buffer - (the offset the function adds)`,
            # so its read lands in a buffer of SIZE random bytes (parked 2113).
            env = {"a": args, "s": answers, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}
            off = eval(expr, env)
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{len(p) - 1}", b)
            ptr = ((buf - off) & 0xFFFFFFFF).to_bytes(4, "little")
            at = eval(loc, env)
            if win is None:
                uc.mem_write(at & 0xFFFFFFFF, ptr)
            else:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[at:at + 4] = ptr
        for idx, size in ANSWERWIN:
            # A stub that answers a record: the address of SIZE random bytes,
            # a new `p` window after the places (parked 2151, item 2170).
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{len(p) - 1}", b)
            answers[idx] = buf
        if VTAT is not None:
            # The stubbed vtable laid in a window: a slot of an object the
            # function reaches through a table, not of `this` (item 2170).
            win, off = VTAT
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = vt.to_bytes(4, "little")
        for idx, win, off in ARGEQ:
            # An argument the function compares with a word of a window laid
            # after the arguments were drawn: the word copied (item 2200).
            if rng.random() < 0.45:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                args[idx] = i32(w, off)
        for a_, b in wins:
            uc.mem_write(a_, bytes(b))
        based = []
        for off, addr in SLOTBASE:
            on = rng.random() < 0.5
            h.w32(vt + off, addr if on else stubbed[off])
            based.append(int(on))
        for name, v in zip(names, answers):
            h.answers[name] = v & 0xFFFFFFFF
        # A callee called more than once a row answers each call in turn:
        # N values drawn a row (from its `--answers` list when it has one),
        # the last repeated past N, printed after the answers (item 2217).
        seqs = []
        for idx, n in SEQ:
            pick = dict(ANSWERS).get(idx)
            vals = [rng.choice(pick) if pick and rng.random() < 0.9 else rand32(rng) for _ in range(n)]
            seqs.append(vals)
            turn = [0]

            def answer(ecx, args, vals=vals, turn=turn):
                v = vals[min(turn[0], len(vals) - 1)]
                turn[0] += 1
                return v & 0xFFFFFFFF
            h.answers[names[idx]] = answer
        uc.mem_write(this, bytes(mem))
        guard = bytes(uc.mem_read(this + WIN, 0x20))
        # `ecx` an integer rather than the object (parked 2141), the row's last.
        ecx = None
        if ECXINT is not None:
            kind, *vals = ECXINT
            if rng.random() < 0.1:
                ecx = rand32(rng)
            elif kind == "set":
                ecx = rng.choice(vals[0])
            else:
                ecx = rng.randint(vals[0], vals[1])
        h.calls.clear()
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx)  # Harness.call signs eax; signed() again would double a negative
        esp = uc.reg_read(UC_X86_REG_ESP)
        # A `__cdecl` function leaves its arguments for the caller (item 2181).
        want_esp = STACK_BASE + STACK_SIZE - 0x100 - (4 * len(args) if CDECL else 0)
        assert esp == want_esp, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based]
                 + ([] if ECXINT is None else [str(ecx)]))
        print(" ".join(shown) + f" | {before} -> {r} | {after}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
