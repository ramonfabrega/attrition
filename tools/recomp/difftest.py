#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""difftest.py — the emulator rung's sweep, natively and under unicorn, row by row.

    uv run tools/recomp/difftest.py <install>/riseofnations.exe diff  [SEED] [N] [--lib LIB]
    uv run tools/recomp/difftest.py <install>/riseofnations.exe sweep [SEED] [N] [--lib LIB]

`diff` runs `tools/emu/callfn.py`'s own sweep (`sweep_calls`, the same
sequence of calls) through both machines and prints every row on which
they disagree, then `rows=N agree=N disagree=M`; its exit status is M's
sign. `sweep` prints the native table alone, in `callfn.py`'s format, so
`crates/sim`'s `RON_EMU_TABLE` can point at it. `LIB` defaults to
`target/recomp/librecomp.dylib`, which `lift.py` builds.

The native machine is `librecomp.dylib` through ctypes: the guest space
from `rc_reserve`, the image's sections written into it at the preferred
base, a stack laid out exactly as the unicorn machine lays it out, and
`rc_call`. A trap (an unlifted target, a `#DE`, a wrong return address)
comes back as the runtime's message.
"""
import ctypes
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
import callfn  # noqa: E402
from callfn import SENTINEL, STACK_BASE, STACK_SIZE, signed  # noqa: E402

import struct  # noqa: E402


class X128(ctypes.Structure):
    _fields_ = [("d", ctypes.c_uint32 * 4)]


class Cpu(ctypes.Structure):
    """`cpu_t` of rt/recomp.h, field for field."""
    _fields_ = [(r, ctypes.c_uint32) for r in ("eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "eip",
                                                "cf", "pf", "af", "zf", "sf", "of", "df")]
    _fields_ += [("xmm", X128 * 8), ("mxcsr", ctypes.c_uint32), ("fs_base", ctypes.c_uint32),
                 ("tsc", ctypes.c_uint64), ("mem", ctypes.c_void_p)]


class NativeMachine:
    def __init__(self, image, lib_path):
        self.lib = ctypes.CDLL(os.path.abspath(lib_path))
        self.lib.rc_reserve.restype = ctypes.c_void_p
        self.lib.rc_call.argtypes = [ctypes.POINTER(Cpu), ctypes.c_uint32]
        self.lib.rc_trap_message.restype = ctypes.c_char_p
        self.mem = self.lib.rc_reserve()
        if not self.mem:
            raise RuntimeError("could not reserve the guest space")
        for _, va, _, raw in image.sections:
            if raw:
                ctypes.memmove(self.mem + image.base + va, raw, len(raw))
        n = ctypes.c_uint32.in_dll(self.lib, "rc_lifted_n").value
        self.lifted = list((ctypes.c_uint32 * n).in_dll(self.lib, "rc_lifted"))

    def call(self, entry, stack_args=(), ecx=0, edx=0):
        esp = STACK_BASE + STACK_SIZE - 0x100
        frame = struct.pack("<I", SENTINEL) + b"".join(struct.pack("<i", a) for a in stack_args)
        esp -= len(frame)
        ctypes.memmove(self.mem + esp, frame, len(frame))
        c = Cpu()
        c.esp, c.ecx, c.edx = esp, ecx & 0xFFFF_FFFF, edx & 0xFFFF_FFFF
        c.mem = self.mem
        if self.lib.rc_call(ctypes.byref(c), entry):
            raise RuntimeError(f"{entry:08x}: trap: {self.lib.rc_trap_message().decode()}")
        if c.eip != SENTINEL:
            raise RuntimeError(f"{entry:08x}: returned to {c.eip:08x}, not the sentinel")
        return c.eax


def answer(m, entry, args, ecx, edx):
    try:
        return signed(m.call(entry, args, ecx=ecx, edx=edx))
    except RuntimeError as e:
        return f"trap({e})"


def main(argv):
    args = argv[1:]
    lib = os.path.join(HERE, "..", "..", "target", "recomp", "librecomp.dylib")
    if "--lib" in args:
        k = args.index("--lib")
        lib = args[k + 1]
        del args[k:k + 2]
    if len(args) < 2 or args[1] not in ("diff", "sweep"):
        print(__doc__)
        return 2
    seed = int(args[2]) if len(args) > 2 else 424242
    n = int(args[3]) if len(args) > 3 else 2000
    image = callfn.Image(args[0])
    native = NativeMachine(image, lib)
    if args[1] == "sweep":
        callfn.sweep(native, seed, n)
        return 0
    emu = callfn.Machine(image)
    calls = list(callfn.sweep_calls(seed, n))
    t0 = time.perf_counter()
    ref = [answer(emu, e, a, x, d) for _, e, a, x, d in calls]
    t1 = time.perf_counter()
    got = [answer(native, e, a, x, d) for _, e, a, x, d in calls]
    t2 = time.perf_counter()
    bad = 0
    for (label, *_), r, g in zip(calls, ref, got):
        if r != g:
            bad += 1
            print(f"{label}: unicorn -> {r}  native -> {g}")
    print(f"lifted: {' '.join(f'{e:08x}' for e in native.lifted)}")
    print(f"rows={len(calls)} agree={len(calls) - bad} disagree={bad}  unicorn {t1 - t0:.3f}s  native {t2 - t1:.3f}s")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
