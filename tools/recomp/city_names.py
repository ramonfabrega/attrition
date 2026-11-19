#!/usr/bin/env python3
"""city_names.py — every live city's `CityData +0x90` name, off each packet (item 4416).

    python3 tools/recomp/city_names.py ~/code/fun/attrition/game

For every `<run>/stream.tsv` under the packets directory (`RON_PACKETS_DIR`,
default item 4188's), it opens the run's one `frame-snapshot.bin` under the
golden directory (`RON_GOLDEN_DIR`, default `~/ron-golden`) and writes
`<run>/names.tsv` beside the stream:

    # packet logger frame <N>
    who	city	o	flags	name

one row per record of `cities.lists[who]` whose `city_flags & 1` holds — the
gate `CityData::log_data@004895c0` prints a `CITY` record under, so the rows
are the cities the dump would name. `rondata::diff::checksum_lines::packets`
reads it against our walk's names on the packet's frame.

The walk, from the PDB's types (`types.txt` in the export):

- `cities` (`?cities@@3VCities@@A`, its VA read off the linker map) holds
  `CitiesData::lists`, a `PtrArray<City>[8]` of `0x1c` each: the count at
  `+0x4` and the `City **` at `+0x10` (`Cities::init_city@007352c0`).
- A `City` begins with its `CityData`: `city_flags` `+0x4`, `city` `+0x6`,
  `o` `+0x8`, `who` `+0x5e`, `name` (a `String`, `0x14`) `+0x90`.
- A `String` holds its length in characters at `+0x8` (`curr_len`); with
  `flags & 2` its first word points to guts whose first word points to the
  characters, and without it the first word is the characters (the
  constant `loc_str_array_orig +0x15e0`, `" "`). The characters are
  UTF-16 (`Napata` reads `N\\0a\\0p\\0…` on ch94p).

The names are the install's own (each tribe's `<CITIES>` list) and never
enter the repository: the files are written outside it.
"""
import argparse
import os
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from snapshot import Snapshot  # noqa: E402

CITIES_SYMBOL = "?cities@@3VCities@@A"
LIST, COUNT_AT, LIST_AT = 0x1C, 0x4, 0x10
FLAGS_AT, CITY_AT, O_AT, WHO_AT, NAME_AT = 0x4, 0x6, 0x8, 0x5E, 0x90


def cities_va(install):
    for line in open(os.path.join(install, "sbl", "rise_z.map"), encoding="latin-1"):
        parts = line.split()
        if len(parts) >= 3 and parts[1] == CITIES_SYMBOL:
            return int(parts[2], 16)
    sys.exit(f"city_names.py: {CITIES_SYMBOL} not in the map")


def string(snap, head):
    """A `String`'s characters: `curr_len` (`+0x8`) UTF-16 units, at the
    first word itself (a constant, `flags` (`+0xa`) clear) or at the first
    word of the guts it points to (`flags & 2`, every city name on disk)."""
    (word,) = struct.unpack_from("<I", head, 0)
    (length,) = struct.unpack_from("<H", head, 0x8)
    flags = head[0xA]
    if not word or not length:
        return ""
    data = struct.unpack("<I", snap.read(word, 4))[0] if flags & 2 else word
    return snap.read(data, 2 * length).decode("utf-16-le")


def names(snap, cities):
    """(who, city, o, flags, name) for every live record, in list order."""
    out = []
    for who in range(8):
        rec = cities + who * LIST
        n, lst = snap.u32(rec + COUNT_AT), snap.u32(rec + LIST_AT)
        for i in range(n if lst else 0):
            c = snap.u32(lst + 4 * i)
            if not c:
                continue
            head = snap.read(c, NAME_AT + 0x14)
            flags, city, o = struct.unpack_from("<Hhh", head, FLAGS_AT)
            if not flags & 1:
                continue
            (cw,) = struct.unpack_from("<b", head, WHO_AT)
            if cw != who:
                raise ValueError(f"list {who} holds a live city of {cw}")
            out.append((who, city, o, flags, string(snap, head[NAME_AT:NAME_AT + 0xC])))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("install")
    ap.add_argument("--packets", default=os.environ.get(
        "RON_PACKETS_DIR", os.path.expanduser("~/ron-data/lab-experiments/2026-11-12-item-4188")))
    ap.add_argument("--golden", default=os.environ.get("RON_GOLDEN_DIR", os.path.expanduser("~/ron-golden")))
    args = ap.parse_args()
    cities = cities_va(args.install)
    written = rows = 0
    for run in sorted(os.listdir(args.packets)):
        if not os.path.exists(os.path.join(args.packets, run, "stream.tsv")):
            continue
        snaps = sorted(Path(args.golden, run).glob("*/frame-snapshot.bin"))
        if len(snaps) != 1:
            print(f"{run}: {len(snaps)} snapshots under {args.golden}/{run}, skipped", file=sys.stderr)
            continue
        snap = Snapshot(str(snaps[0]))
        got = names(snap, cities)
        with open(os.path.join(args.packets, run, "names.tsv"), "w", encoding="utf-8") as f:
            f.write(f"# packet logger frame {snap.frame}\n")
            for row in got:
                f.write("\t".join(str(v) for v in row) + "\n")
        written += 1
        rows += len(got)
    print(f"city_names.py: {written} packets, {rows} live cities")


if __name__ == "__main__":
    main()
