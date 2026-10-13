"""sweeplib.py — the helpers every hand-written sweep script carried (parked
1761, the thirtieth pass: fourteen lanes threw away the same `stub`,
`argstub` and `setup()`; `tools/sweep_gen.py` generates against this).

A generated sweep lays out a little address space and calls one function of
the original under unicorn (`callfn.Machine`); these are the regions and the
two code stubs that laying-out needed every time.

    OBJ   0x10000000  the object `ecx` points at (0x20000 bytes)
    TYPE  0x10100000  the type record a unit's `this+0x18` points at
    ARG   0x20000000  a page for a stack argument that is a pointer
    VT    0x30000000  a page for a fake vtable
    CODE  0x31000000  stubs, a fresh 0x20 bytes a row (unicorn serves a
                      stale block for code rewritten in place)
    REC   0x40000000  the record a global cell points at
    OUT   0x50000000  a page an out-pointer argument names
"""
import struct

OBJ, TYPE = 0x10000000, 0x10100000
ARG, VT, CODE, REC, OUT = 0x20000000, 0x30000000, 0x31000000, 0x40000000, 0x50000000
REGIONS = ((OBJ, 0x20000), (TYPE, 0x1000), (ARG, 0x1000), (VT, 0x1000), (CODE, 0x20000),
           (REC, 0x2000), (OUT, 0x1000))


def setup(uc):
    """Map every region; a script's `main` calls this once."""
    for base, size in REGIONS:
        uc.mem_map(base, size)


def stub_bytes(v, ret=0):
    """`mov eax, v; ret` (or `ret N`)."""
    tail = b"\xc3" if not ret else b"\xc2" + struct.pack("<H", ret)
    return b"\xb8" + struct.pack("<i", v) + tail


def argstub_bytes(v):
    """`eax = v + 1000*[esp+4] + [esp+8]; ret 8`: the answer carries the two
    stack arguments the caller pushed, so a row can see what a virtual call
    was handed."""
    return (b"\x8b\x44\x24\x04" + b"\x69\xc0" + struct.pack("<i", 1000) + b"\x03\x44\x24\x08"
            + b"\x05" + struct.pack("<i", v) + b"\xc2\x08\x00")


_next = [0]


def stub(uc, v, ret=0):
    """Write a `mov eax, v; ret` stub at a fresh CODE address and return it."""
    a = CODE + _next[0]
    _next[0] += 0x20
    uc.mem_write(a, stub_bytes(v, ret))
    return a


def argstub(uc, v):
    a = CODE + _next[0]
    _next[0] += 0x20
    uc.mem_write(a, argstub_bytes(v))
    return a


def cell(uc, va, ptr):
    """Point the global cell at `va` (in the image's own .data, already
    mapped) at `ptr`."""
    uc.mem_write(va, struct.pack("<I", ptr))


def answer_regs(answer, ecx, args):
    """`(eax, ecx_out)` for a hooked callee's answer: a value, or a
    function of `ecx` and the arguments, either of which may be a pair
    `(eax, ecx)` — `Tribes`, `Lands` and `Herds` read a callee's `ecx` on
    return, the decompiler's `extraout_ECX` (parked 1922, Loop 1890's sixth
    reach), which an answer that sets `eax` alone cannot feed."""
    out = answer(ecx, args) if callable(answer) else answer
    if isinstance(out, tuple):
        return out[0], out[1]
    return out, None


def xmm_bits(value, raw=False):
    """The low lane of an xmm register for a float argument (parked 1867:
    `Guy::set_bank`/`set_pitch` take a float in `xmm1`): a Python float's
    IEEE-754 single bits, or `value` itself with `raw=True`. Pass it as
    `Machine.call(..., xmm={1: xmm_bits(0.5)})`."""
    if raw:
        return int(value) & 0xFFFFFFFF
    return struct.unpack("<I", struct.pack("<f", value))[0]
