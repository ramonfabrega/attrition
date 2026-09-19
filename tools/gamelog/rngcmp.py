"""rngcmp.py A B — first sim-frame at which the game_random word (FRAME record) differs.

Names are taken relative to the archive directory (`RON_GAMELOG_DIR`, or the
bottle's `Logs`); an absolute path is used as given, which is what lets a
staged capture outside the archive — the golden record's own output tree — be
compared without being filed first.

**Nothing in common is not agreement.** Two traces with no frame number in
common once read as "0 differing"; the common count is printed and an empty
intersection exits non-zero.
"""
import os, struct, sys
L = os.environ.get("RON_GAMELOG_DIR",
                   os.path.expanduser("~/ron-data/AppData/Roaming/Microsoft Games/"
                                      "Rise of Nations/Logs")) + "/"
def words(p):
    d = {}
    with open(p, "rb") as f:
        f.read(32)
        while True:
            r = f.read(32)
            if len(r) < 32:
                break
            k, a, b, c, dd, e, ff, g = struct.unpack("<8I", r)
            if k == 2:
                d[a] = b
    return d
def path(name):
    return name if os.path.isabs(name) else L + name


a = words(path(sys.argv[1])); b = words(path(sys.argv[2]))
print(len(a), len(b))
common = sorted(set(a) & set(b))
print("frames in common:", len(common))
diff = sorted(k for k in common if a[k] != b[k])
print("differing frames:", len(diff), "first:", diff[:8])
same = sum(1 for k in common if a[k] == b[k])
print("identical frames:", same)
sys.exit(1 if not common or diff else 0)
