"""flags.py — `step4.py`'s command line as one pure function (parked 1869,
the thirtieth pass), so the flags are testable without unicorn.

    --max-insns N        the instruction cap
    --out DIR            where the lifted C and `step4-entered.txt` go
    --entry HEX          the function entered (default `Game::do_frame`)
    --this HEX           `ecx` at entry (default the packet's `game`)
    --args A,B,…         stack arguments, pushed first-nearest
    --dump VA+N          print a range of N bytes before and after
    --watch VA+N         print who writes a range of N bytes; a field is
                         VA+OFF:LEN (`unit+0x58` watched 0x58 bytes from
                         the unit and missed the field, 3269; a bare
                         `+N` past 0x20 is refused, Loop 3280)
    --find VALUE         name the instruction that first produces a value
    --poke ADDR=HEXBYTES write bytes into the packet before the entry
                         (1845's `game->frame` poke)
    --hook ADDR[:CAP]    print the registers and the top of the stack each
                         time ADDR is reached, CAP times (default 20;
                         1738's `s4h.py`, hand-written)
"""
from types import SimpleNamespace

VALUED = ("--max-insns", "--out", "--entry", "--this", "--dump", "--watch", "--find", "--args", "--poke", "--hook")


def parse(argv):
    """`(flags, rest)`: the flags as attributes, `rest` the positional
    arguments (`exe`, `snapshot`)."""
    args = list(argv)
    f = SimpleNamespace(max_insns=20_000_000, out=None, entry=None, this=None, dump=[], watch=[],
                        find=set(), args=[], poke=[], hook=[])
    for flag in VALUED:
        while flag in args:
            k = args.index(flag)
            v = args[k + 1]
            if flag == "--max-insns":
                f.max_insns = int(v)
            elif flag == "--out":
                f.out = v
            elif flag == "--entry":
                f.entry = int(v, 16)
            elif flag == "--this":
                f.this = int(v, 16)
            elif flag == "--args":
                f.args = [int(a, 0) for a in v.split(",")]
            elif flag == "--find":
                f.find.add(int(v, 0) & 0xFFFFFFFF)
            elif flag == "--poke":
                addr, _, hexbytes = v.partition("=")
                f.poke.append((int(addr, 16), bytes.fromhex(hexbytes)))
            elif flag == "--hook":
                addr, _, cap = v.partition(":")
                f.hook.append((int(addr, 16), int(cap) if cap else 20))
            else:
                (f.dump if flag == "--dump" else f.watch).append(parse_range(flag, v))
            del args[k:k + 2]
    return f, args


# A bare `VA+N` past this many bytes reads as an offset, not a length.
BARE_RANGE_MAX = 0x20


def parse_range(flag, v):
    """`VA+N` (N bytes from VA) or `VA+OFF:LEN` (LEN bytes at VA+OFF);
    `(start, length)`. A bare `+N` past `BARE_RANGE_MAX` is refused: the
    first watch of `unit+0x58` covered `+0..+0x57` and missed the field
    (3269's line, Loop 3280)."""
    va, _, n = v.partition("+")
    if ":" in n:
        off, length = n.split(":")
        return int(va, 16) + int(off, 16), int(length, 16)
    length = int(n, 16)
    if length > BARE_RANGE_MAX:
        raise SystemExit(f"{flag} {v}: `+{n}` is a length of {length:#x} bytes from {va}; a field is "
                         f"VA+OFF:LEN (`{va}+{n}:4`), a range past {BARE_RANGE_MAX:#x} bytes VA+0:LEN "
                         "(Loop 3280)")
    return int(va, 16), length


def hook_line(address, function, offset, count, regs, stack):
    """One line per hit: `hook <va> <fn>+<off> #<n> eax=… ecx=… [esp+4]=…`."""
    parts = [f"hook {address:08x} {function}+{offset:#x} #{count}"]
    parts += [f"{name}={regs[name] & 0xFFFFFFFF:08x}" for name in ("eax", "ecx", "edx", "ebx", "esi", "edi", "esp")
              if name in regs]
    parts += [f"[esp+{4 * (i + 1)}]={v & 0xFFFFFFFF:08x}" for i, v in enumerate(stack)]
    return " ".join(parts)
