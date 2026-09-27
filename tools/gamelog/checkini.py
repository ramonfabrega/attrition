#!/usr/bin/env python3
"""checkini.py key=Value [key=Value ...] — set lobby lines in `check.ini`.

    checkini.py difficulties=Toughest      rules.xml `difficulties`, index 5

`-config check.ini` builds the lobby with `GameInfo::load_from_config`,
whose category list reads each `<category>=<name>` line of the `[CHECK]`
section (`docs/AI.md` §80.1), so a lobby field of the Great Lakes captures
— which keep `-config` — is set here, with the game closed. The profile's
twin is `profile.py`, for the captures without `-config`.

**Nothing restores this by itself.** `longtrace.sh` copies `check.ini`
before it calls this (the `CHECKINI` hook, a stanza's `checkini:` key) and
copies it back when the take ends; a take that dies first leaves the copy
at `$RON_TMP/check.ini.run<N>`. The run's own `GAME INFO` block is the
read-back, and some lines of this file never take (`docs/ORACLE.md`, "The
lobby is a file"), so read it.
"""
import os
import re
import sys

G = os.environ.get("RON_INSTALL", "/Users/rf-studio/code/fun/attrition/game")


def main():
    pairs = sys.argv[1:]
    if not pairs or any(not re.fullmatch(r"[a-zA-Z_0-9]+=[^=\n]+", p) for p in pairs):
        print(__doc__.strip().splitlines()[0], file=sys.stderr)
        return 64
    p = G + "/check.ini"
    t = open(p, "rb").read().decode("utf-8", "replace")
    for pair in pairs:
        key, val = pair.split("=", 1)
        t, n = re.subn(r"(?m)^%s=.*$" % re.escape(key), "%s=%s" % (key, val), t)
        assert n == 1, "check.ini: %d lines of %s= (want exactly one)" % (n, key)
        print("check.ini: %s=%s" % (key, val))
    open(p, "wb").write(t.encode())
    return 0


if __name__ == "__main__":
    sys.exit(main())
