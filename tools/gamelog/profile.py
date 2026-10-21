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

`XPACK_LAST_TRIBE1=N` is the one field outside the game blocks: the first
computer's nation (19 the Lakota), written once, read back from the start
dump's `LEADERS` `tribe` (item 3417's peace-pair lobby).
"""
import os
import re
import sys

# Fields the profile carries once, outside the two game blocks.
ONE_BLOCK = ("XPACK_LAST_TRIBE1",)

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
        if key in ONE_BLOCK:
            # The AI's nation, as the click-free lane's `--ai-tribe` writes it
            # (`set_ai_tribe` in `tools/explore/unattended_capture.py`): one
            # field, and only under one computer in slot 1 (item 3417).
            assert 0 <= int(val) < 24, "profile: %s wants a nation 0..23" % key
            assert re.findall(r'<LAST_SLOT1 value="(-?\d+)"/>', t) == ["1"], \
                "profile: %s wants exactly one computer in player slot 1" % key
            t, n = re.subn(r'<%s value="-?\d+"/>' % key, '<%s value="%s"/>' % (key, val), t)
            assert n == 1, "profile: %d <%s value=…/> (want one)" % (n, key)
        else:
            t, n = re.subn(r'<%s value="-?\d+"/>' % key, '<%s value="%s"/>' % (key, val), t)
            assert n >= 2, "profile: %d <%s value=…/> (want <SOLO> and <MULTI>)" % (n, key)
        print("profile: %s=%s in %d blocks" % (key, val, n))
    open(p, "wb").write(t.encode())
    return 0


if __name__ == "__main__":
    sys.exit(main())
