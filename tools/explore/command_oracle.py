#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Execute the original command issuer, and replay its touched-memory closure.

uv run tools/explore/command_oracle.py INSTALL > /tmp/command-oracle.txt
cargo run -p rondata --release --example command_oracle_check < /tmp/command-oracle.txt

Synthetic single-player fixture, NOT a live capture or scheduling test.
Original code runs unchanged. The imported memcpy has a bounded host adapter.
Output contains original-derived packets; keep it outside the repository.
Layout evidence and limits: docs/lab/2026-09-09-command-loop-experiment.md.
"""
import importlib.util
from pathlib import Path
import struct
import sys
import time

from unicorn import UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE, UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_ESP, UC_X86_REG_EIP, UC_X86_REG_EAX

spec = importlib.util.spec_from_file_location(
    'callfn', Path(__file__).resolve().parents[1] / 'emu/callfn.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


class Fixture:
    def __init__(self, install):
        self.m = c.Machine(c.Image(Path(install) / 'riseofnations.exe'))
        self.u = self.m.uc
        self.u.mem_map(0, 4096)  # FS:0 exception chain, no exceptions exercised
        self.u.mem_map(0x10000000, 0x5000)
        self.game, self.group, objects, slots, self.unit = (
            0x10000000 + i * 4096 for i in range(5))
        self.package = 0xe8ff88
        self.put(0xc061ec, self.game)
        self.put(0xc0618c, objects)
        self.put(objects + 0x14, slots)
        self.put(0xc0aec0, slots)
        self.put(objects + 0x15c, 10)
        self.put(slots + 4, self.unit)
        self.put(self.unit, 0xb417d0)  # original Unit vtable
        self.put(self.unit + 8, 1)  # active
        self.u.mem_write(self.unit + 0x8e, struct.pack('<H', 0x8000))  # captain
        self.put(self.unit + 0x30, 42)  # UID
        self.put(self.group + 0xc, 1)
        self.u.mem_write(self.group + 0x8cc, struct.pack('<h', 1))
        self.before = {}
        self.tracking = False
        self.u.hook_add(UC_HOOK_MEM_READ | UC_HOOK_MEM_WRITE, self.access)
        self.u.hook_add(UC_HOOK_CODE, self.memcpy, begin=0x55e0ac, end=0x55e0ac)
        self.calls = 0
        self.pages = set()

    def put(self, address, value):
        self.u.mem_write(address, struct.pack('<I', value & 0xffffffff))

    def remember(self, address, size):
        if self.tracking and size:
            for page in range(address & ~4095, (address + size - 1 & ~4095) + 4096, 4096):
                if page not in self.before:
                    self.before[page] = bytes(self.u.mem_read(page, 4096))

    def access(self, uc, access, address, size, value, data):
        self.remember(address, size)

    def memcpy(self, uc, address, size, data):
        esp = uc.reg_read(UC_X86_REG_ESP)
        self.remember(esp, 16)
        ret, dst, src, n = struct.unpack('<IIII', uc.mem_read(esp, 16))
        if n > 512:
            raise ValueError(f'memcpy exceeds fixture bound: {n}')
        self.remember(src, n)
        self.remember(dst, n)
        if n:
            uc.mem_write(dst, bytes(uc.mem_read(src, n)))
        uc.reg_write(UC_X86_REG_EAX, dst)
        uc.reg_write(UC_X86_REG_ESP, esp + 4)  # cdecl: caller removes arguments
        uc.reg_write(UC_X86_REG_EIP, ret)

    def issue(self, x=25600, y=28160, queue=2, set_angle=0, angle=0,
              orders=0, form=0, width=0, disembark=0):
        args = (self.group, x, y, queue, set_angle, angle, orders, form, width, disembark)
        context = self.u.context_save()
        self.before = {}
        self.tracking = True
        first = self.m.call(0x941720, args)
        self.tracking = False
        after = {p: bytes(self.u.mem_read(p, 4096)) for p in self.before}
        for page, data in self.before.items():
            self.u.mem_write(page, data)
        self.u.context_restore(context)
        second = self.m.call(0x941720, args)
        assert first == second, 'replay return differs'
        assert all(bytes(self.u.mem_read(p, 4096)) == data for p, data in after.items()), \
            'replay memory effects differ'
        self.calls += 2
        self.pages.update(self.before)
        n = struct.unpack('<h', self.u.mem_read(self.package + 0x10, 2))[0]
        assert 0 <= n <= 512
        return bytes(self.u.mem_read(self.package + 0x12, n))

    def clear_packet(self, size=0):
        self.u.mem_write(self.package + 0x10, struct.pack('<h', size))
        self.u.mem_write(self.package + 0x12, b'\x01' * 512)


def main():
    start = time.perf_counter()
    f = Fixture(sys.argv[1])
    # Tab-separated rows: label, expected selection encoding, move fields, packet.
    count = 0

    def emit(label, selection, args, packet):
        nonlocal count
        print(label, selection, *args, packet.hex() or '-', sep='\t')
        count += 1

    args = (25600, 28160, 2, 0, 0, 0, 0, 0, 0)
    emit('first', 'new', args, f.issue(*args))
    f.clear_packet()
    emit('reuse', 'reuse', args, f.issue(*args))
    # Packing boundaries and all queue modes; these validate serialization,
    # not whether a real game would accept arbitrary coordinates/enum values.
    for i, x in enumerate((-2147483648, -1, 0, 1, 2147483647)):
        for queue in range(3):
            args = (x, 28160, queue, 1, -1, 1, 2, 3, 1)
            f.clear_packet()
            emit(f'fields-{i}-{queue}', 'reuse', args, f.issue(*args))
    args = (25600, 28160, 2, 0, 0, 0, 0, 0, 0)
    for size in (480, 485, 486, 487, 507, 508, 511, 512):
        f.clear_packet(size)
        f.put(f.unit + 0x30, size)  # UID change forces full selection emission
        packet = f.issue(*args)
        expected = 'new' if size <= 485 else 'group' if size <= 507 else 'none'
        emit(f'capacity-{size}', expected, args, packet[size:])
        f.clear_packet()
        emit(f'after-capacity-{size}', 'reuse', args, f.issue(*args))
    for label, addr, value in (('playback', 0xe8ff60 + 0x15264, 1),
                               ('semaphore', f.game + 0x820, 0x10)):
        f.clear_packet()
        f.u.mem_write(addr, bytes([value]))
        emit(label, 'none', args, f.issue(*args))
        f.u.mem_write(addr, b'\0')
    print(f'rows={count} executions={f.calls} replay_pairs={f.calls//2} '
          f'data_pages_union={len(f.pages)} elapsed_s={time.perf_counter()-start:.6f}',
          file=sys.stderr)


if __name__ == '__main__':
    main()
