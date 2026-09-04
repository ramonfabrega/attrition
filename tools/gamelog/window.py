"""Stage a DUMP_ALL window run, or undo one.

    window.py stage LO HI   gamelog.ini DUMP_ALL=1 and [Start Game] WORLD=6;
                            rise.ini InitialDump=1; rise2.ini LogStartFrame=LO
                            LogEndFrame=HI; rontrace.cfg cover=1 window=LO-(HI-1);
                            rontrace.cmd `5 !ffwd 30` and `HI+1 !quit`
    window.py frames LO HI  the frame window alone — rise2.ini and the start
                            dump, leaving gamelog.ini's own detail levels
                            (setlog.py's job) and rontrace.cmd untouched. This
                            is the cheap window: a per-frame dump at the
                            `[End Frame]` thresholds runs at full speed, so it
                            can be hundreds of frames wide rather than three.
    window.py restore       DUMP_ALL=0, WORLD=0, InitialDump=0, the window off

A `FRAME n` block is the end of sim-frame n-1, so a window [LO, HI) shows
the effects of sim-frames LO-1 .. HI-2; run22 (docs/ORACLE.md) used
[3579, 3582) for a Dock::init the trace placed at 3579. Edit the cmd file
after staging if the run wants more than the quit.
"""
import os, re, sys
B = os.path.expanduser("~/ron-data/AppData/Roaming/Microsoft Games/Rise of Nations")
G = "/Users/rf-studio/code/fun/attrition/game"

def edit(path, fn):
    raw = open(path, "rb").read()
    crlf = b"\r\n" in raw
    text = raw.decode("utf-8", errors="replace").replace("\r\n", "\n")
    text = fn(text)
    open(path, "wb").write(text.replace("\n", "\r\n" if crlf else "\n").encode("utf-8"))

def set_key(text, key, value, section=None):
    out, cur, done = [], None, False
    for line in text.split("\n"):
        s = line.strip()
        if s.startswith("[") and s.endswith("]"):
            cur = s
        if (section is None or cur == section) and re.match(re.escape(key) + r"\s*=", s):
            line = "%s=%s" % (key, value); done = True
        out.append(line)
    assert done, (key, section)
    return "\n".join(out)

mode = sys.argv[1]
if mode == "stage":
    lo, hi = int(sys.argv[2]), int(sys.argv[3])
    edit(B + "/gamelog.ini", lambda t: set_key(set_key(t, "DUMP_ALL", "1"), "WORLD", "6", "[Start Game]"))
    edit(B + "/rise.ini", lambda t: set_key(t, "InitialDump", "1"))
    edit(B + "/rise2.ini", lambda t: set_key(set_key(t, "LogStartFrame", str(lo)), "LogEndFrame", str(hi)))
    open(G + "/rontrace.cfg", "w").write("cover=1\nwindow=%d-%d\n" % (lo, hi - 1))
    open(G + "/rontrace.cmd", "w").write(
        "# window.py: DUMP_ALL window [%d, %d); dump off until then.\n"
        "5 !ffwd 30\n%d !quit\n" % (lo, hi, hi + 1))
    print("staged", lo, hi)
elif mode == "frames":
    lo, hi = int(sys.argv[2]), int(sys.argv[3])
    edit(B + "/rise.ini", lambda t: set_key(t, "InitialDump", "1"))
    edit(B + "/rise2.ini", lambda t: set_key(set_key(t, "LogStartFrame", str(lo)), "LogEndFrame", str(hi)))
    print("frame window", lo, hi)
else:
    edit(B + "/gamelog.ini", lambda t: set_key(set_key(t, "DUMP_ALL", "0"), "WORLD", "0", "[Start Game]"))
    edit(B + "/rise.ini", lambda t: set_key(t, "InitialDump", "0"))
    edit(B + "/rise2.ini", lambda t: set_key(set_key(t, "LogStartFrame", "0"), "LogEndFrame", "0"))
    print("restored")
