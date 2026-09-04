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

**The profile says the map style three times and the lobby reads the one
you would not guess.** `Player.dat` carries
`<SETTINGS><MAP_STYLE>N</MAP_STYLE>` near its head and a
`<MAP_STYLE value="N"/>` inside each of its `<SOLO>` and `<MULTI>` blocks.
**Solo Game → Quick Battle reads the `<SETTINGS>` one**: with `<MULTI>`
alone set the run came back on the old map (run34), and with `<SOLO>` and
`<MULTI>` both set it came back on the old map again (run36) — two
fifteen-minute captures of Great Lakes. All three are written now, and the
run's own `MAP_STYLE` line is still what confirms it took: **grep it before
reading anything else.**

The index is the position in `data/rules.xml`'s `mapstyles` category list,
which is what the dump's `MAP_STYLE` reports back — so a run's own log says
whether this took.
"""
import os
import re
import sys

G = os.environ.get("RON_INSTALL", "/Users/rf-studio/code/fun/attrition/game")
B = os.path.expanduser(
    "~/ron-data"
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
    # All three, in two spellings. `<SETTINGS><MAP_STYLE>N</MAP_STYLE>` is
    # the one Solo Game → Quick Battle actually reads; `<SOLO>` and
    # `<MULTI>` carry `<MAP_STYLE value="N"/>`. Writing fewer than all
    # three is how two captures came back on the wrong map.
    t, a = re.subn(r"<MAP_STYLE>\d+</MAP_STYLE>",
                   "<MAP_STYLE>%d</MAP_STYLE>" % want, t)
    t, b = re.subn(r'<MAP_STYLE value="\d+"/>',
                   '<MAP_STYLE value="%d"/>' % want, t)
    assert a >= 1 and b >= 2, "profile: %d <SETTINGS>, %d lobby MAP_STYLE" % (a, b)
    open(p, "wb").write(t.encode())
    print("map style %d (%s) in check.ini and the profile" % (want, name))
    return 0


if __name__ == "__main__":
    sys.exit(main())
