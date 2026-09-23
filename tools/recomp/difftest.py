#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""difftest.py — original functions natively and under unicorn, row by row.

    uv run tools/recomp/difftest.py <exe> diff  [SEED] [N] [--lib LIB]
    uv run tools/recomp/difftest.py <exe> sweep [SEED] [N] [--lib LIB]
    uv run tools/recomp/difftest.py <exe> frame <frame-snapshot.bin> <typed-state.json> [--lib LIB] [--table OUT]

`diff` runs `tools/emu/callfn.py`'s own sweep (`sweep_calls`, the same
sequence of calls) through both machines and prints every row on which
they disagree, then `rows=N agree=N disagree=M`; its exit status is M's
sign. `sweep` prints the native table alone, in `callfn.py`'s format, so
`crates/sim`'s `RON_EMU_TABLE` can point at it.

`frame` loads the lab's end-frame snapshot (`snapshot.py`) into both
machines — every range at the address the game had it, the image's sections
beside them — and calls `GuyData::turn_speed@005de340` on every guy of every
unit the lab's decode marks active, in both modes, comparing the two
machines row for row. `--table` also writes each row with the inputs the
original read (the type's turn speed, the pack bit, the squad size, the
guy's number, track offset, speeds and flags) and the answer, for a third
reader to assert against.

The native machine is `librecomp.dylib` through ctypes: a `PROT_NONE`
reservation with the image, a stack and the snapshot's ranges opened in it,
so a read of anything not laid out faults and names its guest address —
the same answer unicorn's unmapped-access hook gives. A trap (an unlifted
target, a `#DE`, a wrong return address, a fault) comes back as the
runtime's message.
"""
import ctypes
import json
import os
import struct
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "emu"))
sys.path.insert(0, HERE)
import callfn  # noqa: E402
from callfn import SENTINEL, STACK_BASE, STACK_SIZE, align, signed  # noqa: E402
from snapshot import Snapshot  # noqa: E402

TURN_SPEED = 0x005DE340
UNITS = 0x00C0AEB0  # `units`: ten 0x1c-byte bands, `length` at +4 and `list` at +0x10


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
        self.lib.rc_map.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_uint32]
        self.lib.rc_call.argtypes = [ctypes.POINTER(Cpu), ctypes.c_uint32]
        self.lib.rc_trap_message.restype = ctypes.c_char_p
        self.mem = self.lib.rc_reserve()
        if not self.mem:
            raise RuntimeError("could not reserve the guest space")
        self.map(image.base, align(image.size))
        self.map(STACK_BASE, STACK_SIZE)
        for _, va, _, raw in image.sections:
            if raw:
                self.write(image.base + va, raw)
        n = ctypes.c_uint32.in_dll(self.lib, "rc_lifted_n").value
        self.lifted = list((ctypes.c_uint32 * n).in_dll(self.lib, "rc_lifted"))

    def map(self, base, size):
        if self.lib.rc_map(self.mem, base, size):
            raise RuntimeError(f"could not map {base:08x}+{size:x}")

    def write(self, base, data):
        ctypes.memmove(self.mem + base, data, len(data))

    def load(self, snap):
        for base, size, _ in snap.ranges:
            self.map(base, size)
        snap.load(self.write)

    def call(self, entry, stack_args=(), ecx=0, edx=0):
        esp = STACK_BASE + STACK_SIZE - 0x100
        frame = struct.pack("<I", SENTINEL) + b"".join(struct.pack("<i", a) for a in stack_args)
        esp -= len(frame)
        self.write(esp, frame)
        c = Cpu()
        c.esp, c.ecx, c.edx = esp, ecx & 0xFFFF_FFFF, edx & 0xFFFF_FFFF
        c.mem = self.mem
        if self.lib.rc_call(ctypes.byref(c), entry):
            raise RuntimeError(f"{entry:08x}: trap: {self.lib.rc_trap_message().decode()}")
        if c.eip != SENTINEL:
            raise RuntimeError(f"{entry:08x}: returned to {c.eip:08x}, not the sentinel")
        return c.eax


def load_into_unicorn(m, image, snap):
    """The snapshot's ranges into `callfn.Machine` `m`: mapped where nothing
    is, written over the image block where it already is."""
    from unicorn import UcError
    lo, hi = image.base, image.base + align(image.size)
    loaded, skipped = [], []
    for r in snap.ranges:
        base, size, _ = r
        if not (lo <= base and base + size <= hi):
            try:
                m.uc.mem_map(base, size)
            except UcError as e:
                skipped.append((base, size, str(e)))
                continue
        loaded.append(r)
    snap.load(lambda base, data: m.uc.mem_write(base, data), loaded)
    return skipped


def answer(m, entry, args, ecx, edx):
    try:
        return signed(m.call(entry, args, ecx=ecx, edx=edx))
    except RuntimeError as e:
        return f"trap({e})"


def active_units(typed_state):
    """The unit addresses the lab's decode marks active on the logger's frame."""
    d = json.load(open(typed_state))
    return sorted(u["address"] for u in d["unit_registry"]["units"] if u["logger_active_flag"])


def frame_calls(snap, units):
    """Every guy of every unit, both modes, with the inputs the original reads."""
    u8 = lambda va: snap.read(va, 1)[0]  # noqa: E731
    s8 = lambda va: struct.unpack("<b", snap.read(va, 1))[0]  # noqa: E731
    s16 = lambda va: struct.unpack("<h", snap.read(va, 2))[0]  # noqa: E731
    s32 = lambda va: struct.unpack("<i", snap.read(va, 4))[0]  # noqa: E731
    for unit in units:
        who, o = u8(unit + 0x9), s16(unit + 0xA)
        ptype = snap.u32(unit + 0x18)
        if snap.u32(snap.u32(UNITS + who * 0x1C + 0x10) + o * 4) != unit:
            raise RuntimeError(f"unit {unit:08x}: `units` band {who} slot {o} does not point back at it")
        n, lst = snap.u32(unit + 0xE8), snap.u32(unit + 0xF4)
        for k in range(n):
            guy = snap.u32(lst + 4 * k)
            if u8(guy + 0xA1) != who or s16(guy + 0x8C) != o:
                raise RuntimeError(f"guy {guy:08x} of unit {unit:08x}: who/o disagree with the unit")
            inputs = dict(
                type_turn=snap.u32(ptype + 0x2C4), packed=int(bool(snap.u32(unit + 0x68) & 0x80000)),
                squad=s32(ptype + 0x304), guy_num=s8(guy + 0xA2),
                track_dx=s32(guy + 0x54), track_dy=s32(guy + 0x58),
                last_speed=s32(guy + 0x80), avg_speed=s32(guy + 0x84), instant=int(bool(u8(guy + 0x9A) & 0x10)),
            )
            for mode in (0, 1):
                label = f"turn_speed unit={unit:08x} guy={guy:08x} who={who} o={o} k={k} mode={mode}"
                yield label, TURN_SPEED, (mode,), guy, 0, inputs


def run_frame(exe, snap_path, typed_state, lib, table):
    image = callfn.Image(exe)
    snap = Snapshot(snap_path)
    native = NativeMachine(image, lib)
    t0 = time.perf_counter()
    native.load(snap)
    t1 = time.perf_counter()
    emu = callfn.Machine(image)
    skipped = load_into_unicorn(emu, image, snap)
    t2 = time.perf_counter()
    print(f"snapshot: frame {snap.frame} (trace {snap.trace_frame}), {len(snap.ranges)} ranges, "
          f"{sum(r[1] for r in snap.ranges)} bytes; loaded natively in {t1 - t0:.2f}s, into unicorn in {t2 - t1:.2f}s")
    for base, size, why in skipped:
        print(f"  unicorn could not map {base:08x}+{size:x}: {why}")
    units = active_units(typed_state)
    calls = list(frame_calls(snap, units))
    t0 = time.perf_counter()
    ref = [answer(emu, e, a, x, d) for _, e, a, x, d, _ in calls]
    t1 = time.perf_counter()
    got = [answer(native, e, a, x, d) for _, e, a, x, d, _ in calls]
    t2 = time.perf_counter()
    bad = 0
    for (label, *_), r, g in zip(calls, ref, got):
        if r != g:
            bad += 1
            print(f"{label}: unicorn -> {r}  native -> {g}")
    traps = sum(1 for r in ref if isinstance(r, str))
    if table:
        with open(table, "w") as f:
            for (label, _, (mode,), guy, _, inp), r in zip(calls, ref):
                if not isinstance(r, str):
                    f.write(f"turn_speed {mode} " + " ".join(str(v) for v in inp.values()) + f" -> {r}\n")
        print(f"table: {table} ({len(calls) - traps} rows, columns: mode {' '.join(calls[0][5])} -> answer)")
    print(f"units={len(units)} rows={len(calls)} agree={len(calls) - bad} disagree={bad} traps={traps}  "
          f"unicorn {t1 - t0:.3f}s  native {t2 - t1:.3f}s")
    return 1 if bad else 0


def main(argv):
    args = argv[1:]
    lib = os.path.join(HERE, "..", "..", "target", "recomp", "librecomp.dylib")
    table = None
    for flag in ("--lib", "--table"):
        if flag in args:
            k = args.index(flag)
            if flag == "--lib":
                lib = args[k + 1]
            else:
                table = args[k + 1]
            del args[k:k + 2]
    if len(args) < 2 or args[1] not in ("diff", "sweep", "frame"):
        print(__doc__)
        return 2
    if args[1] == "frame":
        if len(args) < 4:
            print(__doc__)
            return 2
        return run_frame(args[0], args[2], args[3], lib, table)
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
