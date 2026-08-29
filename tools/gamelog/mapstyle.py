#!/usr/bin/env python3
"""mapstyle.py N — put map style N in the two files the lobby reads.

    mapstyle.py 14      Great Lakes  (run10-14's game)
    mapstyle.py 18      East Indies  (run20-29's)
    mapstyle.py --list  the indices, from the install's own rules.xml

The Quick Battle lobby does **not** take its map style from `check.ini`
alone: it reads the profile (`docs/ORACLE.md`, "The lobby is a file"), and
a style picked from the combo does not survive a killed process. Both are
written here, with the game closed, which is faster than clicking and
cannot mis-click.

**The profile has two lobbies and Solo Game reads the first.** `Player.dat`
carries a `<SOLO>` block and a `<MULTI>` block, each with its own
`<MAP_STYLE>`. Writing only `<MULTI>` — which is what this did until
2026-08-29 — leaves Solo Game → Quick Battle on whatever `<SOLO>` said, and
the run comes back on the wrong map with nothing but its own `MAP_STYLE`
line to say so (run34). Every `<MAP_STYLE>` in the file is written now, and
the run's log is still what confirms it took.

The index is the position in `data/rules.xml`'s `mapstyles` category list,
which is what the dump's `MAP_STYLE` reports back — so a run's own log says
whether this took.
"""
import os
import re
import sys

G = os.environ.get("RON_INSTALL", "/Users/rf-studio/code/fun/attrition/game")
B = os.path.expanduser(
    "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover"
    "/AppData/Roaming/Microsoft Games/Rise of Nations")


def styles():
    rules = open(G + "/data/rules.xml", "rb").read().decode("utf-8", "replace")
    block = re.search(r'<CATEGORIES id="mapstyles".*?</CATEGORIES>', rules, re.S)
    return re.findall(r'<CATEGORY name="([^"]*)"', block.group(0))


def main():
    names = styles()
    if len(sys.argv) < 2 or sys.argv[1] == "--list":
        for i, n in enumerate(names):
            print("%3d  %s" % (i, n))
        return 0
    want = int(sys.argv[1])
    name = names[want]

    p = G + "/check.ini"
    t = open(p, "rb").read().decode("utf-8", "replace")
    open(p, "wb").write(re.sub(r"mapstyles=.*", "mapstyles=" + name, t).encode())

    p = B + "/PlayerProfile/Player.dat"
    t = open(p, "rb").read().decode("utf-8", "replace")
    # Every `<MAP_STYLE value="N"/>` — the `<SOLO>` lobby's and the
    # `<MULTI>` lobby's. The bare `<MAP_STYLE>` element near the head of
    # the file is a different tag (it has no `value` attribute) and is left
    # alone.
    t, n = re.subn(r'<MAP_STYLE value="\d+"/>',
                   '<MAP_STYLE value="%d"/>' % want, t)
    assert n >= 2, "expected a <MAP_STYLE> in both the SOLO and MULTI lobbies"
    open(p, "wb").write(t.encode())
    print("map style %d (%s) in check.ini and the profile" % (want, name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
