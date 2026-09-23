#!/usr/bin/env python3
"""report.py — read rontrace.log (tools/trace/tracer.c) and name what it holds.

    report.py <log> [--index INDEX.tsv] summary
    report.py <log> draws [FRAME ...]          every draw of the given sim-frames
                                               (default: all; `setup` = before frame 0),
                                               with the caller, the ebp chain, and the
                                               value the draw returned
    report.py <log> sites [FRAME ...]          the same, folded by caller site with counts
    report.py <log> coverage [FRAME ...]       functions entered, per frame (re-armed
                                               frames) — names, counts
    report.py <log> calls [FRAME ...]          every proxied call of the given sim-frames
                                               — the arguments, the answer, and the byte
                                               an out-argument came back with, nested
    report.py <log> blind <docs dir> [<log>...] every `name@00xxxxxx` cited in the docs
                                               that no given trace entered
    report.py <log> functions                  every function ever entered, with the
                                               frame it was first entered on
    report.py <log> when NAME [MIN] [FROM]     per-frame count of the draws made from
                                               one function — how a whole 24,000-frame
                                               trace is asked *when* something happened.
                                               `when Guy::init_real 2` dates every
                                               multi-unit birth in the game

Addresses are named through the Ghidra export's INDEX.tsv (default
~/ghidra-projects/decomp/INDEX.tsv): a return address is reported as
`Class::method+0xNN`, the function containing it. The log's frames are the
original's `Game::frame` at `do_frame` entry, i.e. sim-frames as docs/SYNC.md
counts them; the records before the first FRAME record are the setup path
(frame -1, `setup`).
"""
import bisect
import os
import re
import struct
import sys
from collections import Counter, OrderedDict, defaultdict

BASE = 0x400000
KINDS = {0: "HIT", 1: "get()", 2: "FRAME", 3: "get(a,b)", 4: "rand_real", 5: "INFO", 6: "reseed",
         7: "CALL", 8: "RET"}
INFO = {1: "attach", 2: "hook-mismatch", 3: "hooked", 4: "no-funcs", 5: "protect-fail",
        6: "armed", 7: "detach", 8: "declined", 9: "cmd", 10: "cmd-noconsole", 11: "cmds",
        12: "proxied", 13: "cover", 14: "dropped", 15: "unitid"}
RVA_GAME_RANDOM = 0xA37A8C  # VA 0xE37A8C
# the trampolined functions (tracer.c HOOKS): rva -> the record kind they emit
HOOKS = {0x191ef0: 2, 0x639cf0: 1, 0x639d70: 3, 0x5e18b0: 4, 0x639d30: 6}
# the proxied functions (tracer.c CALLS), by the site id their records carry.
# `args` names the arguments in order; a site with fewer is padded with zeros.
PROXIES = {
    0: (0x283770, "astar_path", ("stack", "step", "anti")),
    1: (0x284e50, "calc_cost",
        ("from.x", "from.y", "to.x", "to.y", "dir", "step", "depth", "transport")),
    2: (0x1e86d0, "do_air_physics", ("order", "goal.x", "goal.y")),
    3: (0x1ea390, "air_turn_speed", ("sign", "alt")),
    4: (0x1f8d20, "set_new_location", ("x", "y", "arg2", "arg3")),
    5: (0x285990, "astar_caravan_road",
        ("stack", "whoA", "whoB", "p4", "p5", "caravan", "p7")),
    6: (0x288740, "valid_roadcoord",
        ("x", "y", "from.x", "from.y", "p5", "p6", "p7", "p8")),
    7: (0x286300, "calc_road_cost", ("node", "whoA", "whoB", "dir", "p5")),
    # RON_TARGET_PROBE builds only. `RON_TURN_PROBE` claims 8 and 9 too, and
    # tracer.c refuses a build that defines both, so an id here is unambiguous
    # for any log a target-probe run wrote.
    8: (0x248da0, "find_nearby_target",
        ("max_dist", "who_out", "add_order", "cavarch", "flags")),
    9: (0x2488f0, "attack_dist", ("o", "who", "x", "y")),
    10: (0x24e5c0, "compare_target", ("o", "who", "in_range", "ai")),
}

# Every site any probe build knows, by the RVA it patches. Since the seventh
# pass a PROXIED record carries its site id in `d`, so a log names its own
# sites and the id table above is only the fallback for older logs.
# RON_LEADER_PROBE's three (ids 8-10 in that build): the AI's own offers.
BY_RVA = {rva: (name, names) for rva, name, names in PROXIES.values()}
BY_RVA.update({
    0x2c40a0: ("create_units", ()),
    # `num` is the eighth argument and the RET record carries `out` in its
    # slot, so it does not ride; the make list's `num` (LEADERS=9) is where
    # it is read.
    0x2c9be0: ("make_me", ("t", "val", "escrow", "cat", "city", "up", "p7")),
    0x2c94f0: ("make_this", ("slot",)),
})
# RON_COLLIDE_PROBE's five (ids 8-12 in that build): the collision sweep read
# from inside, `docs/COLLISION.md` 9. `will_be_corner`'s first two arguments
# are the hit cell, which `collide_here` carries only behind out pointers.
BY_RVA.update({
    0x217060: ("detect_unit_collision",
               ("x", "y", "quick", "boats", "p5", "nocoll", "top_only")),
    0x282540: ("collide_here",
               ("o", "who", "ucx", "ucy", "coll_size", "hit_x*", "hit_y*", "nocoll")),
    0x209fa0: ("will_be_corner", ("hit_x", "hit_y", "ucx", "ucy")),
    0x20a0c0: ("is_here", ("hit_x*", "hit_y*")),
    0x20a040: ("is_corner", ("hit_x", "hit_y", "self")),
})


def unit_names(recs):
    """UnitData* -> "who/o", from RON_COLLIDE_PROBE's INFO 15 records. The
    collision proxies are `__thiscall` on a pointer and the dump beside them
    is keyed on the pair, so without this the two cannot be put side by
    side."""
    names = {}
    for r in recs:
        if r[0] == 5 and r[1] == 15:
            o = r[4] - 0x10000 if r[4] >= 0x8000 else r[4]
            names[r[3]] = f"{r[5]}/{o}"
    return names


def site_table(recs):
    """id -> (rva, name, arg names), from the log's own PROXIED records when
    they carry ids (any nonzero `d`, or a single site), else the id table."""
    proxied = [(r[5], r[2]) for r in recs if r[0] == 5 and r[1] == 12]
    if proxied and (len(proxied) == 1 or any(i for i, _ in proxied)):
        table = dict(PROXIES)
        for i, rva in proxied:
            name, names = BY_RVA.get(rva, (f"site{i}", ()))
            table[i] = (rva, name, names)
        return table
    return PROXIES


class Index:
    def __init__(self, path):
        self.addrs, self.names = [], []
        for line in open(path):
            a, n = line.rstrip("\n").split("\t")[:2]
            self.addrs.append(int(a, 16))
            self.names.append(n)
        order = sorted(range(len(self.addrs)), key=lambda i: self.addrs[i])
        self.addrs = [self.addrs[i] for i in order]
        self.names = [self.names[i] for i in order]
        self.by_addr = dict(zip(self.addrs, self.names))

    def name(self, va, base=BASE):
        """`Class::method+0xNN` for the function containing va (an absolute VA)."""
        if not va:
            return "-"
        va = va - base + BASE
        i = bisect.bisect_right(self.addrs, va) - 1
        if i < 0:
            return f"{va:#x}"
        off = va - self.addrs[i]
        return self.names[i] if off == 0 else f"{self.names[i]}+{off:#x}"

    def func(self, va, base=BASE):
        va = va - base + BASE
        i = bisect.bisect_right(self.addrs, va) - 1
        return self.names[i] if i >= 0 else f"{va:#x}"


def read_log(path):
    data = open(path, "rb").read()
    n = len(data) // 32
    recs = [struct.unpack_from("<8I", data, i * 32) for i in range(n)]
    hdr = recs[0]
    assert hdr[0] == 0x544E4F52, "not a rontrace.log"
    head = dict(version=hdr[1], base=hdr[2], text_rva=hdr[3], text_size=hdr[4], nfuncs=hdr[5],
                win_lo=struct.unpack("<i", struct.pack("<I", hdr[6]))[0],
                win_hi=struct.unpack("<i", struct.pack("<I", hdr[7]))[0])
    return head, recs[1:]


def s32(u):
    return u - (1 << 32) if u >= (1 << 31) else u


def lcg(seed):
    """One step of `Random::get`'s generator, from the seed a record carries."""
    return (seed * 0x19660D + 0x3C6EF35F) & 0xFFFFFFFF


def value(rec):
    """What a draw record actually returned.

    The record's `seed` is the word *before* the step (docs/ORACLE.md, "The
    draw-site trace"), so the outcome is recoverable without the game: step
    once, then apply `Random::get(lo, hi)`'s own scaling,
    `((seed & 0xffff) * (hi - lo)) >> 16) + lo`. A draw that leaves no
    outcome in any dump — a coin inside `Setup::build_empire`, a direction
    nothing records — is readable this way and only this way.

    `get()` and `get(a, b)` are both `(0, 0xffff)` at every site the
    documents cite; `rand_real` and `reseed` return None rather than a
    number that would be a guess.
    """
    kind = rec[0]
    if kind not in (1, 3):
        return None
    return ((lcg(rec[3]) & 0xFFFF) * 0xFFFF) >> 16


def frame_of(rec):
    return s32(rec[7])


def frame_arg(a):
    return -1 if a == "setup" else int(a)


def main():
    args = sys.argv[1:]
    index_path = os.path.expanduser("~/ghidra-projects/decomp/INDEX.tsv")
    if "--index" in args:
        i = args.index("--index")
        index_path = args[i + 1]
        del args[i:i + 2]
    log, cmd, rest = args[0], args[1] if len(args) > 1 else "summary", args[2:]
    head, recs = read_log(log)
    base = head["base"]
    idx = Index(index_path)

    def nm(va):
        return idx.name(va, base)

    def rng(self):
        return "game" if self - base == RVA_GAME_RANDOM else f"rng@{self:#x}"

    if cmd == "summary":
        kinds = Counter(KINDS.get(r[0], r[0]) for r in recs)
        print(f"base {base:#x}  functions listed {head['nfuncs']}  window "
              f"{head['win_lo']}..{head['win_hi']}  records {len(recs)}")
        for k, v in sorted(kinds.items(), key=lambda kv: str(kv[0])):
            print(f"  {k:<10} {v}")
        for r in recs:
            if r[0] == 5:
                print(f"  INFO {INFO.get(r[1], r[1]):<14} {r[2]:#x} {r[3]:#x} {r[4]:#x} {r[5]:#x} {r[6]:#x}")
        frames = [r for r in recs if r[0] == 2]
        draws = defaultdict(int)
        for r in recs:
            if r[0] in (1, 3, 4) and r[2] - base == RVA_GAME_RANDOM:
                draws[frame_of(r)] += 1
        print(f"frames {len(frames)}: " + ", ".join(
            f"{s32(r[1])}:{draws[s32(r[1])]}@{r[2]:08x}" for r in frames[:12])
              + (" ..." if len(frames) > 12 else ""))
        print(f"setup draws (before frame 0): {draws[-1]}")
        return

    if cmd == "when":
        # Per-frame count of the draws made from one function — how a *whole*
        # 24,000-frame trace is asked when something happened, without taking
        # a capture to find out. `Guy::init_real` is one draw per unit
        # created, so `when Guy::init_real 2` dates every squad birth in the
        # game: three at once on Great Lakes 6612 is run76's Archers, and the
        # same count over East Indies names 15782 (docs/ORACLE.md, run77).
        # The match is on the *containing* function, so a call site anywhere
        # inside it counts.
        if not rest:
            print("report.py <log> when <Class::method[<caller[<caller]]> "
                  "[min-per-frame] [from-frame]")
            return
        # The chain, not just the site. `Guy::init_real` alone is NOT a birth
        # counter — it is also reached when an existing unit's guy is
        # re-initialised, and on Great Lakes 6736 it fires three times while
        # the dump's object counts do not move at all. Naming the callers is
        # what makes the count mean something: the birth is
        # `Guy::init_real<Unit::init<Objects::init_unit`.
        want_chain = [s.strip() for s in rest[0].split("<")]
        least = int(rest[1]) if len(rest) > 1 else 1
        since = int(rest[2]) if len(rest) > 2 else -(1 << 30)
        slots = (1, 4, 5)
        per_frame = Counter()
        for r in recs:
            if r[0] not in (1, 3, 4, 6):
                continue
            f = frame_of(r)
            if f < since:
                continue
            if len(want_chain) > len(slots):
                print("at most three names: site < caller < caller")
                return
            if all(nm(r[slots[i]]).split("+")[0] == want_chain[i]
                   for i in range(len(want_chain))):
                per_frame[f] += 1
        for f in sorted(per_frame):
            if per_frame[f] >= least:
                print(f"f{f:<7} {per_frame[f]}")
        if not per_frame:
            print(f"no draw in this trace is made from {' < '.join(want_chain)}")
        return

    want = set(frame_arg(a) for a in rest) if rest and cmd != "blind" else None

    if cmd in ("draws", "sites"):
        per_site = Counter()
        k = 0
        for r in recs:
            if r[0] not in (1, 3, 4, 6):
                continue
            f = frame_of(r)
            if want is not None and f not in want:
                continue
            kind = KINDS[r[0]]
            chain = f"{nm(r[1])} < {nm(r[4])} < {nm(r[5])}"
            if cmd == "draws":
                extra = f" arg0={s32(r[6])}" if r[0] in (3, 6) else ""
                v = value(r)
                got = f" = {v:<5}" if v is not None else " " * 8
                print(f"f{f:<5} {k:>4} {kind:<10} {rng(r[2]):<5} seed {r[3]:08x}{got}  "
                      f"{chain}{extra}")
                k += 1
            else:
                per_site[(f, kind, rng(r[2]), chain)] += 1
        if cmd == "sites":
            for (f, kind, g, chain), n in sorted(per_site.items(), key=lambda kv: (kv[0][0], -kv[1])):
                print(f"f{f:<5} {n:>6}  {kind:<10} {g:<5} {chain}")
        return

    if cmd == "calls":
        # CALL and RET nest, so one stack pairs them: a RET belongs to the
        # innermost open CALL. A trace killed mid-search leaves CALLs open;
        # they print with `= ?` rather than being dropped.
        stack = []
        sites = site_table(recs)
        units = unit_names(recs)
        half = {}
        for r in recs:
            if r[0] == 5 and r[1] == 16:
                # RON_COLLIDE_PROBE's INFO 16: the occupancy block the next
                # `collide_here` reads, live and (when held) the
                # pathfinder's copy. Printed as the unit cells set in it.
                f = frame_of(r)
                if want is not None and f not in want:
                    continue
                cx, cy, src, part = r[2] & 0xFF, (r[2] >> 8) & 0xFF, (r[2] >> 16) & 0xFF, r[2] >> 24
                pad = '  ' * len(stack)
                if src == 2:
                    print(f"f{f:<5} {pad}  block ({cx},{cy}) live: none ({r[3]:#x})")
                    continue
                if part == 0:
                    half[(cx, cy, src)] = r[3:7]
                    continue
                words = list(half.pop((cx, cy, src), [0, 0, 0, 0])) + list(r[3:7])
                cells = [(cx * 16 + (i >> 4), cy * 16 + (i & 15))
                         for i in range(256) if words[i // 32] >> (i % 32) & 1]
                which = "copy" if src == 1 else "live"
                print(f"f{f:<5} {pad}  block ({cx},{cy}) {which}: "
                      + " ".join(f"{x},{y}" for x, y in cells))
                continue
            if r[0] not in (7, 8):
                continue
            f = frame_of(r)
            if want is not None and f not in want:
                continue
            site = r[1]
            name = sites.get(site, (0, f"site{site}", ()))[1]
            names = sites.get(site, (0, "", ()))[2]
            if r[0] == 7:
                stack.append((f, site, r[2], list(r[3:7])))
                continue
            if not stack:
                print(f"f{f:<5} {'  ' * 0}{name} = {s32(r[2])}  (no matching CALL)")
                continue
            cf, csite, this, a03 = stack.pop()
            if csite != site:  # a proxy whose CALL was outside the window
                stack.append((cf, csite, this, a03))
                continue
            depth = len(stack)
            # The RET record carries args 4..6 and, in its last slot, the
            # byte behind arg 7 where the site names one: **arg 7 itself is
            # never logged**. Its name prints with `?`, never a value — the
            # `0` this once printed read as `collide_here`'s `nocoll = 0`
            # inside `nocoll = 1` probes for two items (item 566).
            args = (a03 + list(r[3:6]) + [None])[:max(len(names), 1)]
            parts = [f"this={units[this]}" if this in units else f"this={this:#x}"]
            for n, v in zip(names, args):
                parts.append(f"{n}=?" if v is None else f"{n}={s32(v)}")
            if name == "valid_roadcoord":
                parts = [f"tile {args[0] // 0xc0},{args[1] // 0xc0}",
                         f"from {args[2] // 0xc0},{args[3] // 0xc0}"]
            if name == "calc_cost":
                step = args[5] or 1
                parts = [f"from {args[0]},{args[1]} (c{args[0] // step},{args[1] // step})",
                         f"to {args[2]},{args[3]} (c{args[2] // step},{args[3] // step})",
                         f"dir {args[4]} step {args[5]} depth {args[6]}"]
            out = "" if r[6] == 0xFFFFFFFF else f"  out {r[6]}"
            print(f"f{cf:<5} {'  ' * depth}{name}  {'  '.join(parts)} = {s32(r[2])}{out}")
        for cf, csite, this, a03 in stack:
            name = sites.get(csite, (0, f"site{csite}", ()))[1]
            print(f"f{cf:<5} {name}  this={this:#x} {a03} = ?")
        return

    if cmd == "coverage":
        per_frame = OrderedDict()
        for r in recs:
            if r[0] == 0:
                per_frame.setdefault(frame_of(r), []).append(r[1])
        for f, hits in per_frame.items():
            if want is not None and f not in want:
                continue
            names = sorted(idx.func(a, base) for a in hits)
            print(f"== frame {f}: {len(hits)} functions entered")
            for n in names:
                print(f"   {n}")
        return

    if cmd == "functions":
        first = OrderedDict()
        for r in recs:
            if r[0] == 0 and r[1] not in first:
                first[r[1]] = frame_of(r)
        for a, f in sorted(first.items(), key=lambda kv: kv[0]):
            print(f"{a - base + BASE:08x} f{f:<6} {idx.func(a, base)}")
        print(f"{len(first)} functions entered", file=sys.stderr)
        return

    if cmd == "blind":
        docs = rest[0]
        logs = [log] + rest[1:]
        entered = set()
        for l in logs:
            h, rs = read_log(l)
            kinds = set()
            for r in rs:
                if r[0] == 0:
                    entered.add(r[1] - h["base"] + BASE)
                else:
                    kinds.add(r[0])
            # the hooked functions carry no int3; they were entered iff their records exist
            for rva, kind in HOOKS.items():
                if kind in kinds:
                    entered.add(rva + BASE)
            # nor do the proxied ones — a CALL record is their entry
            sites = site_table(rs)
            for r in rs:
                if r[0] == 7 and r[1] in sites:
                    entered.add(sites[r[1]][0] + BASE)
        cited = defaultdict(set)
        pat = re.compile(r"([A-Za-z_][A-Za-z0-9_:~<>]*)@(00[0-9a-f]{6})")
        for root, _, files in os.walk(docs):
            for fn in files:
                if not fn.endswith(".md"):
                    continue
                p = os.path.join(root, fn)
                for m in pat.finditer(open(p, encoding="utf-8", errors="replace").read()):
                    cited[int(m.group(2), 16)].add(os.path.relpath(p, docs))
        known = [a for a in cited if a in idx.by_addr]
        never = sorted(a for a in known if a not in entered)
        hit = [a for a in known if a in entered]
        print(f"{len(known)} functions cited by address under {docs}; "
              f"{len(hit)} entered by {len(logs)} trace(s), {len(never)} never:")
        for a in never:
            print(f"  {a:08x} {idx.by_addr[a]:<50} {', '.join(sorted(cited[a]))}")
        return

    print(__doc__)


if __name__ == "__main__":
    main()
