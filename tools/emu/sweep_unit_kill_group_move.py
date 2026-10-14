#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_kill_group_move.py — `Unit::kill_group_move@005e3400` under unicorn (item 2368, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_unit_kill_group_move.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a 256-byte object of random bytes (no bias), 1 stack argument(s);
16 random bytes at 0x00e90000; 16 random bytes at 0x00e90010; 16 random bytes at 0x00e90020; 16 random bytes at 0x00e90030; 64 random bytes at 0x00e90100; 64 random bytes at 0x00e90180; 64 random bytes at 0x00e90200; 64 random bytes at 0x00e90280; 16 random bytes at 0x00e90300; vtable slot +0x14 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x24 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; callee 0x005e31d0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; the stubbed vtable laid at `c4+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `c5+0` as well as at `this`; the stubbed vtable laid at `c6+0` as well as at `this`; the stubbed vtable laid at `c7+0` as well as at `this`.
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

ENTRY = 0x005e3400
WIN = 256
OWN = 1
BIAS = []
ARGSET = [(0, [5, 6, 7])]
ARGFROM = []
CELLS = []
AT = [(15269888, 16), (15269904, 16), (15269920, 16), (15269936, 16), (15270144, 64), (15270272, 64), (15270400, 64), (15270528, 64), (15270656, 16)]
PTRARGS = []
PNULLS = []
STUBS = [('slot', 20, 0), ('slot', 16, 0), ('slot', 36, 0), ('slot', 148, 0), ('call', 6173136, 0), ('call', 6171824, 1)]
PLACES = []
NULLS = []
WBIAS = [('m', 220, ('set', [15269888, 15269888, 15269888, 15269888, 0])), ('m', 216, ('set', [1, 2, 3, 4, 0])), ('c0', 4, ('set', [15269888, 15269904])), ('c1', 4, ('set', [15269888, 15269920])), ('c2', 4, ('set', [15269888, 15269936])), ('c3', 4, ('set', [15269888])), ('c0', 8, ('set', [15270144])), ('c1', 8, ('set', [15270272])), ('c2', 8, ('set', [15270400])), ('c3', 8, ('set', [15270528])), ('c4', 8, ('set', [0, 1, 1, 1])), ('c5', 8, ('set', [0, 1, 1, 1])), ('c6', 8, ('set', [0, 1, 1, 1])), ('c7', 8, ('set', [0, 1, 1, 1])), ('c4', 12, ('set', [18, 21, 1, 4, 19])), ('c5', 12, ('set', [18, 21, 1, 4, 19])), ('c6', 12, ('set', [18, 21, 1, 4, 19])), ('c7', 12, ('set', [18, 21, 1, 4, 19])), ('c4', 16, ('set', [0, 1])), ('c5', 16, ('set', [0, 1])), ('c6', 16, ('set', [0, 1])), ('c7', 16, ('set', [0, 1])), ('c4', 20, ('set', [0, 1])), ('c5', 20, ('set', [0, 1])), ('c6', 20, ('set', [0, 1])), ('c7', 20, ('set', [0, 1])), ('c4', 24, ('set', [15270656])), ('c5', 24, ('set', [15270656])), ('c6', 24, ('set', [15270656])), ('c7', 24, ('set', [15270656])), ('c4', 36, ('set', [5, 6, 7])), ('c5', 36, ('set', [5, 6, 7])), ('c6', 36, ('set', [5, 6, 7])), ('c7', 36, ('set', [5, 6, 7])), ('c8', 4, ('set', [-24]))]
FILLS = []
ARGRANGE = []
SLOTBASE = []
ECXINT = None
EDX = None
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = 0.1
ANSWERS = []
VTAT = ('c4', 0)
VTATHALF = []
ANSWERWIN = []
CDECL = False
ARGEQ = []
SEQ = []
ANSRANGE = []
CELLTHIS = []
VTALSO = [('c5', 0), ('c6', 0), ('c7', 0)]
SPARSE = []
RECORD = [0, 1, 2, 3, 4, 5]
RECORDORDER = True
ROWSPLIT = {}
LINKS = []
OUTARG = []
CDECLSTUBS = []
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
    for i in range(600):
        h.brk = base  # the bump heap is reused a row; 600 rows of windows outgrow it
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
        # Hand edit (item 2368; no sweep_gen shape lays a circular doubly
        # linked list): each node's `+0` (its `prev`) is its predecessor on
        # the `next` walk from the tail `m+0xdc`, so the walk's stop — the
        # advanced node's `+0` is the tail — ends on the wrap; a fifth of
        # the rows then lay one later node's `+0` as the tail, an early stop.
        tail = i32(mem, 0xdc) & 0xFFFFFFFF
        if tail:
            node = lambda a_: (a_ - 0xe90000) // 0x10
            walk = [node(tail)]
            while True:
                nxt = node(i32(c[walk[-1]], 4) & 0xFFFFFFFF)
                if nxt == node(tail):
                    break
                walk.append(nxt)
            order = walk[1:] + walk[:1]
            for j, k in enumerate(order):
                c[k][0:4] = (0xe90000 + 0x10 * order[j - 1]).to_bytes(4, "little")
            if len(order) > 1 and rng.random() < 0.2:
                c[rng.choice(order[1:])][0:4] = tail.to_bytes(4, "little")
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
        # Hand edit (item 2368; no sweep_gen shape answers a slot from its
        # own `ecx`, as `update_action`'s): each order answers from its own
        # window — `is_move` (+0x14) its word +8, `get_type` (+0x10) +0xc,
        # `is_pathed` (+0x24) +0x10, and `get_group_move_order` (+0x94)
        # `ecx+0x18` when its word +0x14 is set, else null; that record's
        # vbtable (`c8`, `+4` = -0x18) leads back to the order itself.
        h.answers["slot14"] = lambda ecx, args: h.r32(ecx + 8) & 0xFFFFFFFF
        h.answers["slot10"] = lambda ecx, args: h.r32(ecx + 0xc) & 0xFFFFFFFF
        h.answers["slot24"] = lambda ecx, args: h.r32(ecx + 0x10) & 0xFFFFFFFF
        h.answers["slot94"] = lambda ecx, args: (ecx + 0x18) & 0xFFFFFFFF if h.r32(ecx + 0x14) else 0
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

# sweep_gen: ["obj", "Unit::kill_group_move@005e3400", "--item", "2368", "--win", "0x100", "--own", "1", "--ret", "none", "--rows", "600", "--argset", "0:5,6,7", "--stub", "slot:0x14:0", "--stub", "slot:0x10:0", "--stub", "slot:0x24:0", "--stub", "slot:0x94:0", "--stub", "0x5e31d0:0", "--stub", "0x5e2cb0:1", "--at", "0xe90000:16", "--at", "0xe90010:16", "--at", "0xe90020:16", "--at", "0xe90030:16", "--at", "0xe90100:64", "--at", "0xe90180:64", "--at", "0xe90200:64", "--at", "0xe90280:64", "--at", "0xe90300:16", "--vtat", "c4+0", "--vtat-also", "c5+0", "--vtat-also", "c6+0", "--vtat-also", "c7+0", "--wbias", "m+0xdc:0xe90000,0xe90000,0xe90000,0xe90000,0", "--wbias", "m+0xd8:1,2,3,4,0", "--wbias", "c0+4:0xe90000,0xe90010", "--wbias", "c1+4:0xe90000,0xe90020", "--wbias", "c2+4:0xe90000,0xe90030", "--wbias", "c3+4:0xe90000", "--wbias", "c0+8:0xe90100", "--wbias", "c1+8:0xe90180", "--wbias", "c2+8:0xe90200", "--wbias", "c3+8:0xe90280", "--wbias", "c4+8:0,1,1,1", "--wbias", "c5+8:0,1,1,1", "--wbias", "c6+8:0,1,1,1", "--wbias", "c7+8:0,1,1,1", "--wbias", "c4+0xc:0x12,0x15,1,4,0x13", "--wbias", "c5+0xc:0x12,0x15,1,4,0x13", "--wbias", "c6+0xc:0x12,0x15,1,4,0x13", "--wbias", "c7+0xc:0x12,0x15,1,4,0x13", "--wbias", "c4+0x10:0,1", "--wbias", "c5+0x10:0,1", "--wbias", "c6+0x10:0,1", "--wbias", "c7+0x10:0,1", "--wbias", "c4+0x14:0,1", "--wbias", "c5+0x14:0,1", "--wbias", "c6+0x14:0,1", "--wbias", "c7+0x14:0,1", "--wbias", "c4+0x18:0xe90300", "--wbias", "c5+0x18:0xe90300", "--wbias", "c6+0x18:0xe90300", "--wbias", "c7+0x18:0xe90300", "--wbias", "c4+0x24:5,6,7", "--wbias", "c5+0x24:5,6,7", "--wbias", "c6+0x24:5,6,7", "--wbias", "c7+0x24:5,6,7", "--wbias", "c8+4:-24", "--record", "0", "--record", "1", "--record", "2", "--record", "3", "--record", "4", "--record", "5", "--record-order"]
