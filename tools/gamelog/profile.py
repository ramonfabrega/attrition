#!/usr/bin/env python3
"""profile.py KEY=N [KEY=N ...] — set lobby fields in the profile's game blocks.

    profile.py STARTING_TOWN=3      Large Town (rules.xml `startingtowns`)

The profile's `<SOLO>` and `<MULTI>` blocks carry one `<KEY value="N"/>` per
`gameinfo_strings` name, and `PlayerProfile::load@005cb430` reads each block
into `prefs.game_params[0]` and `[1]` by that table — so a field the
`-config` path does not take, or a capture on a map that must drop
`-config` (`docs/ORACLE.md`, "The lobby is a file"), is set here, with the
game closed. Both blocks are written, as `mapstyle.py` writes both. The
map style is not one of these: `mapstyle.py` owns it, and the lobby reads
it from `<SETTINGS>`.

**Nothing restores this by itself.** `longtrace.sh` copies `Player.dat`
before it calls this (the `PROFILE` hook, a stanza's `profile:` key) and
copies it back when the take ends; a take that dies first leaves the copy
at `$RON_TMP/Player.dat.run<N>` for the hand that restores the INIs. The
run's own `GAME INFO` block (`STARTING_TOWN 3`) is the read-back.
"""
import os
import re
import sys

B = os.path.expanduser(
    "~/ron-data"
    "/AppData/Roaming/Microsoft Games/Rise of Nations")


def main():
    pairs = sys.argv[1:]
    if not pairs or any(not re.fullmatch(r"[A-Z_0-9]+=-?\d+", p) for p in pairs):
        print(__doc__.strip().splitlines()[0], file=sys.stderr)
        return 64
    p = B + "/PlayerProfile/Player.dat"
    t = open(p, "rb").read().decode("utf-8", "replace")
    for pair in pairs:
        key, val = pair.split("=")
        t, n = re.subn(r'<%s value="-?\d+"/>' % key, '<%s value="%s"/>' % (key, val), t)
        assert n >= 2, "profile: %d <%s value=…/> (want <SOLO> and <MULTI>)" % (n, key)
        print("profile: %s=%s in %d blocks" % (key, val, n))
    open(p, "wb").write(t.encode())
    return 0


if __name__ == "__main__":
    sys.exit(main())
