import sys
s = int(sys.argv[1], 16)
n = int(sys.argv[2])
want = set()
for a in sys.argv[3:]:
    lo, _, hi = a.partition("-")
    want.update(range(int(lo), int(hi or lo) + 1))


def variant(p):
    return 0 if p <= 69 else 1 if p <= 82 else 2 if p <= 95 else 3


for i in range(n):
    s = (s * 0x19660D + 0x3C6EF35F) & 0xFFFFFFFF
    r = ((s & 0xFFFF) * 0xFFFF) >> 16
    if i in want:
        print(f"{i:3d} roll {r:5d} %100={r % 100:2d}(v{variant(r % 100)}) %1000={r % 1000:3d}{' SPROUT' if r % 1000 < 20 else ''} %5={r % 5} %4={r % 4} %3={r % 3}")
print(f"end {s:#x}")
