#!/usr/bin/env python3
"""bindkey.py ACTION KEY [--ctrl] [--shift] [--alt] — bind a key the game ships unbound.

    bindkey.py FORM_E_RIGHT 120          # F9 -> Echelon Right
    bindkey.py --list                    # every bindable action
    bindkey.py --restore                 # put the profile's <KEYS> back

Queue item 23's second half needs a group in an **Echelon**, and there is no
other way to ask for one. The console's 102 commands have no formation verb
(`tools/gamelog/console.py`), and the unit command card carries no formation
button for a selected group — with or without full military research, which
run48 and run49 each photographed.

But the action exists. `data/playerprofile.xml` lists `FORM_LINE`,
`FORM_REFUSED`, `FORM_ENVELOP`, `FORM_E_RIGHT` and `FORM_E_LEFT` as bindable
entries, and `KeyMap::init@007d5a90` loads that very file — so it is the
keymap, and its `FORM_*` entries have **no `<INPUT>` child at all**. The
formations ship unbound. That is why no key and no button reaches them.

So bind one. The element's shape is `KeyMap::save_entry@007d4220`'s, with
every attribute name resolved out of `int_str_array` (stride 0x14, the same
table `console.py` reads):

    <KEY enum="FORM_E_RIGHT" dependent="-1">
      <INPUT key="120" mouse="0" ctrl="0" shift="0" alt="0"/>
    </KEY>

and `KeyMap::load_entry@007d43d0` is the reader: `key` is taken whole, then
`ctrl` sets bit 0x20000, `shift` 0x10000 and `alt` 0x40000. `save_entry`
writes an entry only when its `dependent` is negative — that is what marks a
binding as the player's own rather than the shipped default — so a written
entry carries `dependent="-1"`.

**This writes to the profile, not to the install's shipped data.**
`PlayerProfile/Player.dat` is user state and already edited with the game
closed by `mapstyle.py`; `data/playerprofile.xml` is the game's own and is
left alone. `--restore` empties the `<KEYS>` element again, which is how the
profile ships.

The key numbers are Windows virtual-key codes: F9 is 120, F10 121, F11 122,
letters are their capitals (J is 74).
"""
import argparse
import os
import re
import sys

PROFILE = os.path.expanduser(
    "~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover"
    "/AppData/Roaming/Microsoft Games/Rise of Nations/PlayerProfile/Player.dat")
SHIPPED = "data/playerprofile.xml"


def actions(install):
    path = os.path.join(install, SHIPPED)
    with open(path, encoding="utf-8", errors="replace") as fh:
        return re.findall(r'<KEY enum="([A-Z_0-9a-z]+)"', fh.read())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("action", nargs="?")
    ap.add_argument("key", nargs="?", type=int)
    ap.add_argument("--ctrl", action="store_true")
    ap.add_argument("--shift", action="store_true")
    ap.add_argument("--alt", action="store_true")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--restore", action="store_true")
    ap.add_argument("--install", default=os.environ.get(
        "RON_INSTALL", "/Users/rf-studio/code/fun/attrition/game"))
    ap.add_argument("--profile", default=PROFILE)
    a = ap.parse_args()

    if a.list:
        for n in actions(a.install):
            print(n)
        return 0

    raw = open(a.profile, "rb").read().decode("utf-8", "replace")
    if "<KEYS" not in raw:
        sys.exit("no <KEYS> element in %s" % a.profile)

    if a.restore:
        out = re.sub(r"<KEYS\b.*?(?:/>|>.*?</KEYS>)", "<KEYS/>", raw, count=1, flags=re.S)
        open(a.profile, "wb").write(out.encode("utf-8"))
        print("profile <KEYS> emptied")
        return 0

    if not a.action or a.key is None:
        sys.exit("give an ACTION and a KEY, or --list / --restore")
    known = actions(a.install)
    if a.action not in known:
        sys.exit("%s is not a bindable action; --list shows the %d that are"
                 % (a.action, len(known)))

    entry = (
        '<KEY enum="%s" dependent="-1">'
        '<INPUT key="%d" mouse="0" ctrl="%d" shift="%d" alt="%d"/>'
        "</KEY>" % (a.action, a.key, int(a.ctrl), int(a.shift), int(a.alt))
    )
    # Replace the whole <KEYS> element, self-closing or not, keeping any
    # entries already in it that are not for this action.
    m = re.search(r"<KEYS\b.*?(?:/>|>(.*?)</KEYS>)", raw, flags=re.S)
    inner = (m.group(1) or "") if m else ""
    inner = re.sub(r'<KEY enum="%s".*?</KEY>' % re.escape(a.action), "",
                   inner, flags=re.S)
    out = raw[:m.start()] + "<KEYS>" + inner + entry + "</KEYS>" + raw[m.end():]
    open(a.profile, "wb").write(out.encode("utf-8"))
    mods = "".join(k for k, v in (("ctrl", a.ctrl), ("shift", a.shift),
                                  ("alt", a.alt)) if v) or "none"
    print("bound %s to key %d (modifiers: %s) in the profile" % (a.action, a.key, mods))
    return 0


if __name__ == "__main__":
    sys.exit(main())
