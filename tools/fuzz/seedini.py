#!/usr/bin/env python3
"""seedini.py SEED — put a seed in `rise.ini`, which is where the game reads it.

    seedini.py 12345          set it
    seedini.py --show         report what it is now

`rise.ini`'s `Seed (0 for random)` is the master game seed, and it is what
the dump's own `game_random seed` line reports back -- so this is the one
knob that makes a run reproducible and a batch varied. `check.ini` looks like
it should own this (it has a `SEED=`), but `docs/ORACLE.md` records that
`-config check.ini` does not supply the lobby the game actually plays: the
profile's last-used lobby is what appears, and the clicks confirm it. The
seed is the part that does carry, through `rise.ini`.

Deliberately not a general ini editor: it asserts the key exists rather than
adding it, because a typo that silently appends a second key would give a
whole batch of runs the same seed and nothing would look wrong.
"""
import argparse
import os
import re
import sys

RISE = os.path.expanduser(
    "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover"
    "/AppData/Roaming/Microsoft Games/Rise of Nations/rise.ini")
KEY = "Seed (0 for random)"


def read(path):
    raw = open(path, "rb").read()
    return raw, b"\r\n" in raw, raw.decode("utf-8", errors="replace").replace("\r\n", "\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("seed", nargs="?", type=lambda s: int(s, 0))
    ap.add_argument("--show", action="store_true")
    ap.add_argument("--path", default=RISE)
    args = ap.parse_args()

    _, crlf, text = read(args.path)
    pat = re.compile(r"^" + re.escape(KEY) + r"\s*=\s*(\S*)\s*$", re.M)
    found = pat.search(text)
    if not found:
        sys.exit(f"{args.path}: no {KEY!r} key -- refusing to add one")
    if args.show or args.seed is None:
        print(found.group(1))
        return
    if not 0 <= args.seed <= 0xFFFFFFFF:
        sys.exit(f"seed out of range: {args.seed}")
    text = pat.sub(f"{KEY}={args.seed}", text, count=1)
    out = text.replace("\n", "\r\n") if crlf else text
    open(args.path, "wb").write(out.encode("utf-8"))
    print(f"{KEY}={args.seed}")


if __name__ == "__main__":
    main()
