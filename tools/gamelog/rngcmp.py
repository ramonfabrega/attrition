"""rngcmp.py A B — first sim-frame at which the game_random word (FRAME record) differs."""
import struct, sys
L = "/Users/rf-studio/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/Logs/"
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
a = words(L + sys.argv[1]); b = words(L + sys.argv[2])
print(len(a), len(b))
diff = sorted(k for k in a if k in b and a[k] != b[k])
print("differing frames:", len(diff), "first:", diff[:8])
same = sum(1 for k in a if k in b and a[k] == b[k])
print("identical frames:", same)
