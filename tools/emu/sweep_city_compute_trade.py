#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_city_compute_trade.py — `City::compute_trade@00739640` under unicorn (item 2368, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_city_compute_trade.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 144-byte object of random bytes (no bias), 0 stack argument(s);
callee 0x0073d9d0 (2 args) stubbed; the word at `m+0x84` points at 40 random bytes less `0` (placed); the word at `0xe3a2a0` points at 8 random bytes less `0` (placed); the word at `0xe3a2bc` points at 8 random bytes less `0` (placed); the word at `p3+0x0` points at 16 random bytes less `0` (placed); the word at `p3+0x4` points at 16 random bytes less `0` (placed); the word at `p4+0x0` points at 16 random bytes less `0` (placed); the word at `p4+0x4` points at 16 random bytes less `0` (placed); the word at `0xc09970` points at 8 random bytes less `0` (placed); the word at `0xc0998c` points at 8 random bytes less `0` (placed); the word at `p9+0x0` points at 8 random bytes less `0` (placed); the word at `p9+0x4` points at 8 random bytes less `0` (placed); the word at `p10+0x0` points at 8 random bytes less `0` (placed); the word at `p10+0x4` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3, 5, -1, -7, 4096, 2147483647, -2147483648] (90%); stub 0 answers 5 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places).
One row per input, `<args… stubs…> | <this>;<c…>;<p…> -> <eax> | <this>;<c…>;<p…>` (hex); the stubs'
answers follow the arguments, and the test holds every byte of every window.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402
from sweeplib import link_value  # noqa: E402
from sweeplib import draw_reg  # noqa: E402

ENTRY = 0x00739640
WIN = 144
OWN = 0
BIAS = []
ARGSET = []
ARGFROM = []
CELLS = []
AT = []
PTRARGS = []
PNULLS = []
STUBS = [('call', 7592400, 2)]
PLACES = [('m', '0x84', '0', 40), (None, '0xe3a2a0', '0', 8), (None, '0xe3a2bc', '0', 8), ('p3', '0x0', '0', 16), ('p3', '0x4', '0', 16), ('p4', '0x0', '0', 16), ('p4', '0x4', '0', 16), (None, '0xc09970', '0', 8), (None, '0xc0998c', '0', 8), ('p9', '0x0', '0', 8), ('p9', '0x4', '0', 8), ('p10', '0x0', '0', 8), ('p10', '0x4', '0', 8)]
NULLS = []
WBIAS = [('m', 120, ('range', -1, 5)), ('m', 92, ('set', [0, 65536, 1509999570, 1510065106])), ('m', 80, ('set', [0, 524288, 1053236, 1572864, 2097152, 4194304, 2147483648, 4294443008])), ('p2', None, ('range', 0, 1)), ('p5', 0, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p5', 4, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p6', 0, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p6', 4, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p7', 0, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p7', 4, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p8', 0, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p8', 4, ('set', [0, 1, 65536, 65537, 0, 1, 65536, 65537, 65535, 131071, 32768])), ('p5', 12, ('set', [4, 5, 6, 7, 4294967292, 4, 6, 0, 3, 4294967291])), ('p6', 12, ('set', [4, 5, 6, 7, 4294967292, 4, 6, 0, 3, 4294967291])), ('p7', 12, ('set', [4, 5, 6, 7, 4294967292, 4, 6, 0, 3, 4294967291])), ('p8', 12, ('set', [4, 5, 6, 7, 4294967292, 4, 6, 0, 3, 4294967291])), ('p11', 4, ('set', [1, 1, 1, 3, 2147483649, 0, 4294967294])), ('p12', 4, ('set', [1, 1, 1, 3, 2147483649, 0, 4294967294])), ('p13', 4, ('set', [1, 1, 1, 3, 2147483649, 0, 4294967294])), ('p14', 4, ('set', [1, 1, 1, 3, 2147483649, 0, 4294967294]))]
FILLS = []
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = [(0, [0, 1, 2, 3, 5, -1, -7, 4096, 2147483647, -2147483648])]
VTAT = None
VTATHALF = []
ANSWERWIN = []
CDECL = False
ARGEQ = []
SEQ = [(0, 5, None)]
ANSRANGE = []
CELLTHIS = []
VTALSO = []
SPARSE = [('@', '0xe3a390', 28396, 2, [(0, 4)])]
RECORD = [0]
RECORDORDER = False
ROWSPLIT = {}
LINKS = []
OUTARG = []
CDECLSTUBS = [0]
STUBWRITE = []
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


def token(v, spans):
    """A recorded value: a pointer into a window by the window's name and
    offset (`m+0x0`, `p0+0x10`), a local by its depth (`sp-0x14`), anything
    else as a signed word (item 2281)."""
    v &= 0xFFFFFFFF
    for name, at, size in spans:
        if at <= v <= at + size:
            return f"{name}+{v - at:#x}"
    # A caller's local handed to a callee: its depth under the caller's
    # `esp`, which the harness holds every row (item 2336).
    top = STACK_BASE + STACK_SIZE - 0x100
    if top - 0x1000 <= v < top:
        return f"sp-{top - v:#x}"
    return str(v - (1 << 32) if v & 0x80000000 else v)


def by_row(calls, base, stride, n, spans):
    """`--record IDX@S` (item 2309): a stub's calls `(ecx, args)` split by
    the row of a sparse table their `ecx` lies in — one token a row, `ecx`
    written `r+OFF` within it, `-` for a row not called — and a last token
    for the calls whose `ecx` lies outside the table."""
    cols = [[] for _ in range(n + 1)]
    for ecx_, args_ in calls:
        k = ((ecx_ & 0xFFFFFFFF) - base) // stride
        if 0 <= k < n:
            head = f"r+{(ecx_ & 0xFFFFFFFF) - base - k * stride:#x}"
        else:
            k, head = n, token(ecx_, spans)
        cols[k].append(",".join([head] + [token(v, spans) for v in args_]))
    return ["/".join(col) or "-" for col in cols]


def in_order(calls, record, names, spans):
    """`--record-order` (item 2341): the recorded stubs' calls as one token
    a call, in the order they were made, each headed by the callee's stub
    index (`s3:m+0x0`), so two argument-less callees on one `ecx` are
    ordered (`add_think_order`, parked 2318); `-` when none was called."""
    want = {names[idx]: idx for idx in record}
    return [f"s{want[n]}:" + ",".join(token(v, spans) for v in [e] + list(a))
            for n, e, a in calls if n in want] or ["-"]


def out_answer(inner, k, v, write):
    """`--outarg IDX:K:VALS` (item 2325): a stub that writes `v` through its
    argument K — the address of the caller's local, as `get_inside` writes
    the player index — unless that argument is null, then answers as
    `inner` would (a value, or a function of `ecx` and the arguments)."""
    def answer(ecx, args):
        if args[k]:
            write(args[k] & 0xFFFFFFFF, v)
        return inner(ecx, args) if callable(inner) else inner
    return answer


def win_write(inner, at, size, v, write):
    """`--stub-write IDX:WIN+OFF:SIZE:VAL` (item 2346): a stub that writes VAL
    at a window's word each call — a callee that unlinks the head of the
    container its caller loops over (`Object::kill_contents`) — then answers
    as `inner` would."""
    def answer(ecx, args):
        write(at, (v & ((1 << 8 * size) - 1)).to_bytes(size, "little"))
        return inner(ecx, args) if callable(inner) else inner
    return answer


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


def seq_values(rng, n, pick, terms, rand32):
    """`--seq IDX:N[:T,T..]`'s answers a row: N drawn from the stub's
    `--answers` list (90%) or a random word, the last from the terminators
    T when given — answered past N, so a loop that runs while the stub
    answers inside a set ends (`Unit::repath`, item 2368, parked 2342)."""
    vals = [rng.choice(pick) if pick and rng.random() < 0.9 else rand32(rng) for _ in range(n)]
    if terms:
        vals[-1] = rng.choice(terms)
    return vals


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
    rng = random.Random(2368)
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {a: (f"{a:x}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"})
    uc = h.m.uc
    slots = {off: (f"slot{off:x}", n, 0) for k, off, n in STUBS if k == "slot"}
    vt = h.vtable(slots) if slots else None
    names = [f"slot{a:x}" if k == "slot" else f"iat{a:x}" if k == "iat" else f"{a:x}" for k, a, _ in STUBS]
    # `--stub iat:VA:N`: a stub whose address is laid in an import's IAT cell,
    # which the emulator resolves to nothing, so `call [VA]` lands on it
    # (item 2336: `timeGetTime`, parked 2327).
    iats = [(a, h.stub(f"iat{a:x}", n, 0)) for k, a, n in STUBS if k == "iat"]
    stubbed = {off: h.r32(vt + off) & 0xFFFFFFFF for off, _ in SLOTBASE}
    base = h.brk
    for i in range(400):
        h.brk = base  # the bump heap is reused a row; 400 rows of windows outgrow it
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
            # `--ptrarg-null IDX`: the argument laid as 0 a third of the rows,
            # its buffer still drawn so every later window keeps its index
            # (item 2271; a guarded null argument ran one arm).
            if idx in PNULLS and rng.random() < 1 / 3:
                args[idx] = 0
            wins.append((buf, b))
        nc = len(CELLS) + len(AT)
        c = [b for _, b in wins[:nc]]
        p = [b for _, b in wins[nc:]]
        for k, b in enumerate(c):
            shape(rng, f"c{k}", b)
        for k, b in enumerate(p):
            shape(rng, f"p{k}", b)
        tables = []
        for win, loc, stride, n, slices in SPARSE:
            # A table of N rows STRIDE apart of which the function reads a
            # few slices: the table zeroed, each row's slices `p` windows, the
            # pointer laid at LOC — or, `@`, the slices written at the image's
            # own table (parked 2201, item 2247: the leaders table).
            env = {"a": args, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}
            if win == "@":
                tbase = eval(loc, env) & 0xFFFFFFFF
            else:
                tbase = h.alloc(stride * n + 0x20)
                uc.mem_write(tbase, bytes(stride * n))
            tables.append((tbase, stride, n))
            for k in range(n):
                for off, size in slices:
                    b = bytearray(rng.randrange(256) for _ in range(size))
                    p.append(b)
                    wins.append((tbase + k * stride + off, b))
                    shape(rng, f"p{len(p) - 1}", b)
            if win != "@":
                ptr = tbase.to_bytes(4, "little")
                if win is None:
                    uc.mem_write(eval(loc, env) & 0xFFFFFFFF, ptr)
                else:
                    w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                    at = int(loc, 0)
                    w[at:at + 4] = ptr
        # The stubs' answers are drawn before the windows are placed, so a
        # placed table may be indexed by one (`s`, item 2160); `--answers`
        # draws stub IDX's from a list (parked 2141).
        answers = [rand32(rng) for _ in STUBS]
        for idx, vals in ANSWERS:
            answers[idx] = rng.choice(vals) if rng.random() < 0.9 else answers[idx]
        # `--answer-range`: an answer the function indexes a table by, held
        # to the range every row (item 2217).
        for idx, lo, hi in ANSRANGE:
            answers[idx] = rng.randint(lo, hi)
        # `--outarg`: the values stubs write through a pointer argument, drawn
        # with the answers so a place may index by one (`o`, item 2325).
        outs = [wdraw(rng, vals) for _, _, vals in OUTARG]
        k = -1
        for win, loc, expr, size in PLACES:
            # A table pointer laid as `buffer - (the offset the function adds)`,
            # so its read lands in a buffer of SIZE random bytes (parked 2113).
            k += 1
            env = {"a": args, "s": answers, "o": outs, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}
            off = eval(expr, env)
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{len(p) - 1}", b)
            ptr = ((buf - off) & 0xFFFFFFFF).to_bytes(4, "little")
            # `--place-null K`: place K's pointer laid as 0 a third of the
            # rows, the buffer still drawn so every later window keeps its
            # index (item 2253; a guarded null ran one arm).
            if k in NULLS and rng.random() < 1 / 3:
                ptr = bytes(4)
            at = eval(loc, env)
            if win is None:
                uc.mem_write(at & 0xFFFFFFFF, ptr)
            elif win[0] == "s":
                # A stub answers the placed buffer, a record a later place
                # may reach into (item 2241).
                answers[int(win[1:])] = (buf - off) & 0xFFFFFFFF if any(ptr) else 0
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
        for win, off in VTALSO:
            # The stubbed vtable laid in a window as well, `this` keeping it:
            # a record the function reaches whose slots it asks too (item 2217).
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off in VTATHALF:
            # The stubbed vtable laid half the rows at a record the function
            # compares with a class's vtable, the other half left as drawn
            # (a `--wbias` of the class's address; item 2223).
            if rng.random() < 0.5:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = vt.to_bytes(4, "little")
        for lo, hi, off, size, d, terms in LINKS:
            # A link field (`o_up`, a type's upgrade) of figures p(lo)..p(hi),
            # drawn from the figures on one side of each one's own index or a
            # terminator, so a walk through them ends (item 2276).
            n = hi - lo + 1
            for k in range(n):
                v = link_value(rng, n, k, d, terms)
                p[lo + k][off:off + size] = (v & ((1 << 8 * size) - 1)).to_bytes(size, "little")
        for idx, win, off in ARGEQ:
            # An argument the function compares with a word of a window laid
            # after the arguments were drawn: the word copied (item 2200).
            if rng.random() < 0.45:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                args[idx] = i32(w, off)
        for a_, b in wins:
            uc.mem_write(a_, bytes(b))
        for cell, stub in iats:
            uc.mem_write(cell, stub.to_bytes(4, "little"))
        based = []
        for off, addr in SLOTBASE:
            on = rng.random() < 0.5
            h.w32(vt + off, addr if on else stubbed[off])
            based.append(int(on))
        for name, v in zip(names, answers):
            h.answers[name] = v & 0xFFFFFFFF
        # A callee called more than once a row answers each call in turn:
        # N values drawn a row (from its `--answers` list when it has one),
        # the last repeated past N — from the terminators when given (item
        # 2368) — printed after the answers (item 2217).
        seqs = []
        for idx, n, terms in SEQ:
            vals = seq_values(rng, n, dict(ANSWERS).get(idx), terms, rand32)
            seqs.append(vals)
            turn = [0]

            def answer(ecx, args, vals=vals, turn=turn):
                v = vals[min(turn[0], len(vals) - 1)]
                turn[0] += 1
                return v & 0xFFFFFFFF
            h.answers[names[idx]] = answer
        for (idx, k, _), v in zip(OUTARG, outs):
            h.answers[names[idx]] = out_answer(h.answers[names[idx]], k, v,
                                               lambda a_, v_: uc.mem_write(a_, (v_ & 0xFFFFFFFF).to_bytes(4, "little")))
        for idx, win, off, size, v in STUBWRITE:
            at = this if win == "m" else wins[(0 if win[0] == "c" else nc) + int(win[1:])][0]
            h.answers[names[idx]] = win_write(h.answers[names[idx]], at + off, size, v, uc.mem_write)
        uc.mem_write(this, bytes(mem))
        # A global cell that holds `this` itself: a method that reads the
        # global as well as its object sees one object (item 2217).
        for va in CELLTHIS:
            uc.mem_write(va, this.to_bytes(4, "little"))
        guard = bytes(uc.mem_read(this + WIN, 0x20))
        # `ecx` an integer rather than the object (parked 2141), the row's last.
        ecx = None if ECXINT is None else draw_reg(rng, ECXINT, WILD, rand32)
        # `--edx` (item 2346): an integer in `edx` beside `ecx`, after it on the row.
        edx = None if EDX is None else draw_reg(rng, EDX, WILD, rand32)
        h.calls.clear()
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx,
                   edx=0 if EDX is None else edx)  # Harness.call signs eax; signed() again would double a negative
        esp = uc.reg_read(UC_X86_REG_ESP)
        # A `__cdecl` function leaves its arguments for the caller (item 2181).
        want_esp = STACK_BASE + STACK_SIZE - 0x100 - (4 * len(args) if CDECL else 0)
        assert esp == want_esp, f"esp {esp:x}: wrong pop"
        assert bytes(uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers] + [str(v) for v in outs]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based]
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)]))
        # `--record IDX`: what stub IDX was handed, each call `ecx,arg..`,
        # calls joined by `/`, `-` when it was not called, after ` # `
        # (item 2281: a tail target's `ecx` and argument, parked 2272).
        log = ""
        if RECORD:
            spans = [("m", this, WIN)] + [(f"c{k}" if k < nc else f"p{k - nc}", a_, len(b))
                                          for k, (a_, b) in enumerate(wins)]
            cols = []
            if RECORDORDER:
                cols = in_order([(n_, e_, a_) for n_, e_, a_, _ in h.calls], RECORD, names, spans)
            for idx in ([] if RECORDORDER else RECORD):
                calls = [(ecx_, list(args_)) for name_, ecx_, args_, _ in h.calls if name_ == names[idx]]
                if idx in ROWSPLIT:
                    cols += by_row(calls, *tables[ROWSPLIT[idx]], spans)
                else:
                    cols.append("/".join(",".join(token(v, spans) for v in [e] + a_)
                                         for e, a_ in calls) or "-")
            log = " # " + " ".join(cols)
        print(" ".join(shown) + f" | {before} -> {r} | {after}" + log)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

# sweep_gen: ["obj", "City::compute_trade@00739640", "--item", "2368", "--ret", "none", "--win", "0x90", "--wbias", "m+0x78:-1..5", "--wbias", "m+0x5c:0x0,0x10000,0x5a00c3d2,0x5a01c3d2", "--wbias", "m+0x50:0x0,0x80000,0x101234,0x180000,0x200000,0x400000,0x80000000,0xfff80000", "--sparse", "@0xe3a390:0x6eec:2:0,4", "--place", "m+0x84:0:0x28", "--wbias", "p2+*:0..1", "--place", "0xe3a2a0:0:8", "--place", "0xe3a2bc:0:8", "--place", "p3+0x0:0:16", "--place", "p3+0x4:0:16", "--place", "p4+0x0:0:16", "--place", "p4+0x4:0:16", "--place", "0xc09970:0:8", "--place", "0xc0998c:0:8", "--place", "p9+0x0:0:8", "--place", "p9+0x4:0:8", "--place", "p10+0x0:0:8", "--place", "p10+0x4:0:8", "--wbias", "p5+0:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p5+4:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p6+0:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p6+4:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p7+0:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p7+4:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p8+0:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p8+4:0x0,0x1,0x10000,0x10001,0x0,0x1,0x10000,0x10001,0xffff,0x1ffff,0x8000", "--wbias", "p5+0xc:4,5,6,7,0xfffffffc,4,6,0,3,0xfffffffb", "--wbias", "p6+0xc:4,5,6,7,0xfffffffc,4,6,0,3,0xfffffffb", "--wbias", "p7+0xc:4,5,6,7,0xfffffffc,4,6,0,3,0xfffffffb", "--wbias", "p8+0xc:4,5,6,7,0xfffffffc,4,6,0,3,0xfffffffb", "--wbias", "p11+4:1,1,1,3,0x80000001,0,0xfffffffe", "--wbias", "p12+4:1,1,1,3,0x80000001,0,0xfffffffe", "--wbias", "p13+4:1,1,1,3,0x80000001,0,0xfffffffe", "--wbias", "p14+4:1,1,1,3,0x80000001,0,0xfffffffe", "--stub", "0x73d9d0:2", "--stub-cdecl", "0", "--answers", "0:0,1,2,3,5,-1,-7,0x1000,0x7fffffff,-0x80000000", "--seq", "0:5", "--record", "0", "--rows", "400"]
