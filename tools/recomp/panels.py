#!/usr/bin/env python3
"""panels.py — the buttons the original offers a selected building, on a packet.

    python3 tools/recomp/panels.py <install> <frame-snapshot.bin> <out.tsv> [--who N ...]

Item 4305's oracle (`docs/CLIENT_INPUT.md` §32.4): what a building's panel
offers is `BuildOut::make_options@00629ed0`, and it is not dumped by any
logger. So it is run, under `step4.py`, on a packet's own state: for each
slot's buildings, every object, with the console's
`who` poked to the building's owner (`make_options` offers a building of
another slot nothing but `OPTION_EJECT_ALL`) and `Options::add@0072c290`
hooked, each call's six arguments read off the stack at its entry.

Each output row is one `Options::add` the builder made:

    frame  who  object  building  option  grid_x  grid_y  type  disable  extra

`object` is the building's id (`+0xa`), `building` its `TypeIndex`
(`+0x18`'s type, `+4`), `option` the `OptionIndex`, `grid_x`/`grid_y`
signed. A building whose run did not return (`stopped: the frame returned`)
is a row with option `-1` and the run's stop line in a comment, and the
tool exits 1.

The addresses are read from the install's `sbl/rise_z.map`; the layouts are
the PDB's (`types.txt` of the decompile export): `Objects` per slot a
0x1c-byte array record, its data at `+0x14` and its size at `+0x8`;
`Console.who` at `+0x298`. A reading instrument: nothing here is a source,
and neither the packet nor the TSV enters the repository.

**The citizen's build pages** (item 4337, `docs/CLIENT_INPUT.md` §32.6):

    python3 tools/recomp/panels.py <install> <packet> <out.tsv> --citizens [--every] [--who N ...]

enters `UnitOut::make_options@00615fc0` with `this` a citizen (type
`PEASANTS` 0x32 or `PEASANTSKOREAN` 0x33), `MiscAccess::options` (the
pointer at the map's `?options@MiscAccess@@2AAVOptions@@A`) poked at
`mode` `+0x74` = 1 (`BUILD_OPTIONS`; the listing's `cmp` at 0x61601b) and
`mode_data` `+0x78` = the page, one run for each of 0, 1 and 2, and keeps
the hook's adds whose caller is `make_options@UnitOut`. The page's loop
reads only the citizen's slot (`+0x9`) and id (`+0xa`, the scenario's
ignore list), so one citizen a slot is run unless `--every`. Each row:

    frame  who  object  unit  page  option  grid_x  grid_y  type  disable  extra

and a run that did not return is a row with option `-1`, as above.

**The citizen's standard panel** (item 4339, `docs/CLIENT_INPUT.md` §32.7):

    python3 tools/recomp/panels.py <install> <packet> <out.tsv> --citizens --standard [--who N ...]

is the same run with `mode` poked to 0 (`STANDARD_OPTIONS`) and
`mode_data` left as the packet holds it, one run a citizen: the arm that
adds the page buttons `OPTION_BUILD`, `OPTION_BUILD_MILITARY` and
`OPTION_BUILD_WONDER` (each with type −1), `OPTION_REPAIR` and
`OPTION_ALARM_PEASANT`, and every other standard add of the unit. Its
rows carry page −1.
"""
import os
import re
import struct
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from snapshot import Snapshot  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ADD = 0x72C290
ENTRY = 0x629ED0
WHO_AT = 0x298
BUILDTYPES = range(0x192, 0x21F)  # BASE_BUILDTYPES ..= 0x21e
GLOBALS = {
    "objects": "?objects@@3VObjects@@A",
    "console": "?console@MiscAccess@@2AAVConsole@@A",
    "options": "?options@MiscAccess@@2AAVOptions@@A",
}
UNIT_ENTRY = 0x615FC0
CITIZENS = (0x32, 0x33)  # PEASANTS, PEASANTSKOREAN
MODE_AT, MODE_DATA_AT, STANDARD_OPTIONS, BUILD_OPTIONS = 0x74, 0x78, 0, 1
HOOK = re.compile(r"^hook 0072c290 .*?((?: \[esp\+\d+\]=[0-9a-f]{8})+) ret=([0-9a-f]{8}) (\S+)")


def globals_of(install):
    """The data globals' VAs, off the linker map's `name VA` columns."""
    out = {}
    want = {v: k for k, v in GLOBALS.items()}
    for line in open(os.path.join(install, "sbl", "rise_z.map"), encoding="latin-1"):
        parts = line.split()
        if len(parts) >= 3 and parts[1] in want:
            out[want[parts[1]]] = int(parts[2], 16)
    missing = set(GLOBALS) - set(out)
    if missing:
        sys.exit(f"panels.py: {sorted(missing)} not in the map")
    return out


class Mem:
    def __init__(self, snap):
        self.s = snap
        self.r = sorted(snap.ranges)

    def u32(self, a):
        for b, sz, off in self.r:
            if b <= a < b + sz - 3:
                self.s.f.seek(off + a - b)
                return struct.unpack("<I", self.s.f.read(4))[0]
        return None

    def u8(self, a):
        w = self.u32(a & ~3)
        return None if w is None else (w >> (8 * (a & 3))) & 0xFF

    def s16(self, a):
        lo, hi = self.u8(a), self.u8(a + 1)
        return None if lo is None or hi is None else struct.unpack("<h", bytes([lo, hi]))[0]


def citizens(mem, objects, whos, every):
    """(who, object id, unit type, address): each slot's citizens, the first
    alone unless `every`."""
    seen = []
    for who in whos:
        rec = objects + who * 0x1C
        data, size = mem.u32(rec + 0x14), mem.u32(rec + 0x8)
        if not data or not size:
            continue
        mine = []
        for i in range(size):
            p = mem.u32(data + 4 * i)
            if not p:
                continue
            tp = mem.u32(p + 0x18)
            t = mem.u32(tp + 4) if tp else None
            if t in CITIZENS:
                mine.append((who, mem.s16(p + 0xA), t, p))
        mine.sort()
        seen += mine if every else mine[:1]
    return seen


def buildings(mem, objects, whos):
    """(who, object id, building type, address), every building of the slots."""
    seen = []
    for who in whos:
        rec = objects + who * 0x1C
        data, size = mem.u32(rec + 0x14), mem.u32(rec + 0x8)
        if not data or not size:
            continue
        for i in range(size):
            p = mem.u32(data + 4 * i)
            if not p:
                continue
            tp = mem.u32(p + 0x18)
            t = mem.u32(tp + 4) if tp else None
            if t in BUILDTYPES:
                seen.append((who, mem.s16(p + 0xA), t, p))
    return sorted(seen)


def signed(v):
    return v - (1 << 32) if v & 0x80000000 else v


def run(install, packet, who, addr, console, entry=ENTRY, caller="make_options@BuildOut", pokes=()):
    exe = os.path.join(install, "riseofnations.exe")
    cmd = ["uv", "run", os.path.join(HERE, "step4.py"), exe, packet, "--entry", f"{entry:x}", "--this", f"{addr:x}",
           "--poke", f"{console + WHO_AT:x}={struct.pack('<I', who).hex()}", "--hook", f"{ADD:x}:400",
           "--max-insns", "40000000"]
    for va, v in pokes:
        cmd += ["--poke", f"{va:x}={struct.pack('<I', v).hex()}"]
    out = subprocess.run(cmd, capture_output=True, text=True).stdout
    adds = []
    for line in out.splitlines():
        m = HOOK.match(line)
        if m and caller in m.group(3):
            words = [int(w.split("=")[1], 16) for w in m.group(1).split()]
            adds.append([signed(w) for w in words[:6]])
    stop = next((ln for ln in out.splitlines() if ln.startswith("stopped:")), "stopped: no stop line")
    return adds, stop


def main(argv):
    if len(argv) < 3:
        sys.exit(__doc__)
    install, packet, out = argv[:3]
    whos = [int(argv[i + 1]) for i, a in enumerate(argv) if a == "--who"] or list(range(8))
    g = globals_of(install)
    snap = Snapshot(packet)
    mem = Mem(snap)
    console = mem.u32(g["console"])
    if "--citizens" in argv:
        return main_citizens(install, packet, out, whos, "--every" in argv, g, snap, mem, console,
                             "--standard" in argv)
    rows, bad = [], 0
    for who, oid, t, addr in buildings(mem, g["objects"], whos):
        adds, stop = run(install, packet, who, addr, console)
        if stop != "stopped: the frame returned":
            bad += 1
            rows.append(f"{snap.frame}\t{who}\t{oid}\t{t}\t-1\t0\t0\t-1\t0\t0\t# {stop}")
            continue
        for option, gx, gy, typ, disable, extra in adds:
            rows.append(f"{snap.frame}\t{who}\t{oid}\t{t}\t{option}\t{gx}\t{gy}\t{typ}\t{disable}\t{extra}")
        print(f"who {who} object {oid} type {t}: {len(adds)} options", flush=True)
    with open(out, "w") as f:
        f.write("# frame\twho\tobject\tbuilding\toption\tgrid_x\tgrid_y\ttype\tdisable\textra\n")
        f.write("\n".join(rows) + "\n")
    print(f"{len(rows)} rows to {out}; {bad} run(s) that did not return")
    return 1 if bad else 0


def main_citizens(install, packet, out, whos, every, g, snap, mem, console, standard=False):
    """The citizen's three build pages, one run a page (item 4337); with
    `standard`, its standard panel, one run (item 4339), page -1."""
    options = mem.u32(g["options"])
    rows, bad = [], 0
    for who, oid, t, addr in citizens(mem, g["objects"], whos, every):
        for page in ((-1,) if standard else (0, 1, 2)):
            if page < 0:
                pokes = [(options + MODE_AT, STANDARD_OPTIONS)]
            else:
                pokes = [(options + MODE_AT, BUILD_OPTIONS), (options + MODE_DATA_AT, page)]
            adds, stop = run(install, packet, who, addr, console, UNIT_ENTRY, "make_options@UnitOut", pokes)
            if stop != "stopped: the frame returned":
                bad += 1
                rows.append(f"{snap.frame}\t{who}\t{oid}\t{t}\t{page}\t-1\t0\t0\t-1\t0\t0\t# {stop}")
                continue
            for option, gx, gy, typ, disable, extra in adds:
                rows.append(f"{snap.frame}\t{who}\t{oid}\t{t}\t{page}\t{option}\t{gx}\t{gy}\t{typ}\t{disable}\t{extra}")
            print(f"who {who} citizen {oid} type {t} page {page}: {len(adds)} options", flush=True)
    with open(out, "w") as f:
        f.write("# frame\twho\tobject\tunit\tpage\toption\tgrid_x\tgrid_y\ttype\tdisable\textra\n")
        f.write("\n".join(rows) + "\n")
    print(f"{len(rows)} rows to {out}; {bad} run(s) that did not return")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
