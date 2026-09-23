"""snapshot.py — the lab's end-frame snapshot, as guest memory.

`frame-snapshot.bin` is the lab's `frame-snapshot-v1` stream (the end-frame
collector on `codex/typed-state-oracle`, `tools/explore/frame_snapshot.py`
there): a 128-word header, the process's memory inventory, the anchors, then
the selected ranges — a 12-byte `(inventory index, base, size)` record and
the bytes of each — and a footer. This reader takes only what a loader
needs, the ranges and where their bytes sit in the file, and checks the
magics and the counts the header promises. The lab's own reader is the
validator; this one assumes a file it has passed.

The ranges are the process's committed, readable, non-executable private
data and its main-image data — every singleton, the heap, the unit records —
at the addresses the game had them. Mapped into a machine at those
addresses, with the image's sections beside them, an original function that
reads the object graph runs on the state of a real frame. Nothing here is
atomic across threads (the lab says so), and nothing read from it enters
the repository.
"""
import struct

MAGIC, FOOTER = 0x31534652, 0x45465352
CHUNK = 1 << 20


class Snapshot:
    def __init__(self, path):
        self.path = path
        f = self.f = open(path, "rb")
        h = struct.unpack("<32I", f.read(128))
        if h[0] != MAGIC or h[1:3] != (1, 1):
            raise ValueError(f"{path}: not a frame-snapshot-v1 stream")
        self.frame, self.trace_frame, self.game = h[3], h[4], h[6]
        rows, count, total, anchors, anchor_bytes = h[15], h[16], h[17], h[18], h[19]
        f.seek(128 + rows * 28 + anchors * 12 + anchor_bytes)
        self.ranges = []  # (base, size, file offset)
        for _ in range(count):
            _, base, size = struct.unpack("<3I", f.read(12))
            self.ranges.append((base, size, f.tell()))
            f.seek(size, 1)
        foot = struct.unpack("<7I", f.read(28))
        if foot[0] != FOOTER or foot[3:5] != (count, total) or sum(r[1] for r in self.ranges) != total:
            raise ValueError(f"{path}: the footer does not close the ranges the header promised")
        self.ranges.sort()

    def read(self, va, n):
        for base, size, off in self.ranges:
            if base <= va and va + n <= base + size:
                self.f.seek(off + va - base)
                return self.f.read(n)
        raise KeyError(f"{va:08x}: not in the snapshot")

    def u32(self, va):
        return struct.unpack("<I", self.read(va, 4))[0]

    def load(self, write, ranges=None):
        """Feed every range (or `ranges`) to `write(base, bytes)`, a megabyte at a time."""
        for base, size, off in self.ranges if ranges is None else ranges:
            self.f.seek(off)
            done = 0
            while done < size:
                n = min(CHUNK, size - done)
                write(base + done, self.f.read(n))
                done += n
