"""orderhead.py — the order list's head, laid out for the four `UnitData`
predicates that rewind the list's cursor and ask the head order (item 2061:
`order_type`, `is_moving`, `is_packing`, `is_unpacking`; hand-written on
`tools/emu/hooks.py`, the Loop's next shape — parked under item 2061).

Each opens the same way (`0060aa60`, `00610af0`, `0060a4b0`, `00616e80`):
`this+0xdc` is the list's head node, or null; a non-null head's `+4` is the
first link, which is stored at `this+0xd4`, the link's `+8` (the order) at
`this+0xcc` and its byte `+0xc` at `this+0xd0`. The order is then read back
from `this+0xcc`.

`Layout(h).lay(rng, order_vt, has_head, has_order)` writes a fresh random
unit and list a row and returns the unit; `after()` names what
became of the three cursor fields — `n` the list's value, `o` the old one —
and asserts no other byte of the unit, the head or the link moved.
"""
import struct

UNIT = 0x100
HEAD, LINK = 0xDC, 0xD4
ORDER, FLAG = 0xCC, 0xD0


class Layout:
    def __init__(self, h):
        self.h = h
        self.this = h.alloc(UNIT)
        self.head = h.alloc(0x20)
        self.link = h.alloc(0x20)
        self.order = h.alloc(0x40)

    def lay(self, rng, order_vt, has_head, has_order):
        h, uc = self.h, self.h.m.uc
        for a, n in ((self.this, UNIT), (self.head, 0x20), (self.link, 0x20), (self.order, 0x40)):
            uc.mem_write(a, bytes(rng.randrange(256) for _ in range(n)))
        h.w32(self.this + HEAD, self.head if has_head else 0)
        h.w32(self.head + 4, self.link)
        h.w32(self.link + 8, self.order if has_order else 0)
        h.w32(self.order, order_vt)
        # The old cursor differs from the list's in every field, so `n` and `o` never tie.
        h.w32(self.this + LINK, self.link ^ 0x10)
        h.w32(self.this + ORDER, h.r32(self.link + 8) ^ 0x10)
        h.w8(self.this + FLAG, uc.mem_read(self.link + 0xC, 1)[0] ^ 0xFF)
        self.before = self.snap()
        return self.this

    def snap(self):
        uc = self.h.m.uc
        return (bytearray(uc.mem_read(self.this, UNIT)), bytes(uc.mem_read(self.head, 0x20)),
                bytes(uc.mem_read(self.link, 0x20)), bytes(uc.mem_read(self.order, 0x40)))

    def after(self):
        """`<0xd4><0xcc><0xd0>`, each `n` (the list's) or `o` (untouched)."""
        unit, head, link, order = self.snap()
        old = self.before[0]
        assert (head, link, order) == self.before[1:], "the list moved"
        want = {LINK: struct.pack("<I", self.link), ORDER: bytes(link[8:12]), FLAG: bytes(link[0xC:0xD])}
        marks = ""
        for off, new in want.items():
            got = bytes(unit[off:off + len(new)])
            if got == new:
                marks += "n"
            elif got == bytes(old[off:off + len(new)]):
                marks += "o"
            else:
                marks += "x"
            unit[off:off + len(new)] = old[off:off + len(new)]
        assert unit == old, "a byte of the unit outside the cursor moved"
        return marks
