#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Bind/replay one opt-in live CommandPackage::clear capture in a fresh emulator.

bind INSTALL CAPTURE_DIRECTORY: record staged executable/tracer identities.
replay INSTALL CAPTURE_DIRECTORY: verify live outputs twice, including writes.
No full-image mapping, import adapters, inferred zero-filled dependencies or
original data outputs in the repository. This is a bounded leaf-call capsule.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

from unicorn import (Uc, UcError, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE,
                     UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE, UC_MEM_WRITE,
                     UC_PROT_READ, UC_PROT_WRITE, UC_PROT_EXEC)
from unicorn import x86_const as x

REG_NAMES = ('EDI', 'ESI', 'EBP', 'ESP', 'EBX', 'EDX', 'ECX', 'EAX', 'EFLAGS')
REG_IDS = [getattr(x, 'UC_X86_REG_' + name) for name in REG_NAMES]
ENTRY, CODE_SIZE = 0x94c1c0, 21


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_trace(path):
    raw = path.read_bytes()
    require(len(raw) >= 32 and len(raw) % 32 == 0, 'incomplete trace')
    require(struct.unpack_from('<8I', raw)[:3] == (0x544e4f52, 2, 0x400000), 'unexpected trace header')
    records = list(struct.iter_unpack('<8I', raw[32:]))
    require(not any(r[0] == 5 and r[1] in (2, 5, 14) for r in records), 'trace health error')
    return records


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def image_bytes(path, address, size):
    raw = path.read_bytes()
    pe = struct.unpack_from('<I', raw, 0x3c)[0]
    require(raw[pe:pe+4] == b'PE\0\0', 'not PE')
    opt = pe + 24
    require(struct.unpack_from('<H', raw, opt)[0] == 0x10b, 'not PE32')
    base = struct.unpack_from('<I', raw, opt+28)[0]
    count, opt_size = struct.unpack_from('<H', raw, pe+6)[0], struct.unpack_from('<H', raw, pe+20)[0]
    for i in range(count):
        h = opt + opt_size + 40*i
        _, rva, raw_size, off = struct.unpack_from('<IIII', raw, h+8)
        at = address-base-rva
        if 0 <= at and at+size <= raw_size:
            return raw[off+at:off+at+size]
    raise ValueError('code outside image sections')


def bind(install, directory):
    files = {'source': install / 'riseofnations.exe',
             'staged_source': directory / 'riseofnations.exe',
             'traced': directory / 'riseofnations_trace.exe',
             'tracer': directory / 'rontrace.dll'}
    hashes = {name: digest(path) for name, path in files.items()}
    require(hashes['source'] == hashes['staged_source'], 'staged original differs')
    with (directory / 'capsule-image.json').open('x') as out:
        json.dump({'schema': 1, 'sha256': hashes}, out, indent=2)


class Capsule:
    def __init__(self, raw):
        require(len(raw) == 1224, 'truncated or oversized capsule')
        header = struct.unpack_from('<27I', raw)
        magic, version, self.entry, self.stop, self.frame, self.self_, self.global_, self.game, self.caller = header[:9]
        require((magic, version, self.entry) == (0x31504352, 1, ENTRY), 'unsupported capsule')
        self.before_regs, self.after_regs = header[9:18], header[18:27]
        self.code = raw[108:132]
        self.before, self.after = raw[132:668], raw[668:1204]
        self.seed_before, self.seed_after, self.ptr_before, self.ptr_after, self.stack_return = struct.unpack_from('<5I', raw, 1204)
        require(20 <= self.frame <= 35, 'outside capture window')
        require(self.before_regs[6] == self.self_, 'ECX does not identify package')
        require(self.game == self.ptr_before and self.global_ == 0xc061ec, 'invalid singleton dependency')
        require(self.stack_return == self.stop, 'inconsistent stop address')
        require(self.after_regs[3] == self.before_regs[3]+4, 'unexpected live stack effect')
        require(struct.unpack_from('<H', self.before, 16)[0] > 0, 'no live package to clear')
        self.regions = [
            ('code', self.entry, self.code, self.code),
            ('package', self.self_, self.before, self.after),
            ('singleton', self.global_, struct.pack('<I', self.ptr_before), struct.pack('<I', self.ptr_after)),
            ('game_seed', self.game+16, struct.pack('<I', self.seed_before), struct.pack('<I', self.seed_after)),
            ('return', self.before_regs[3], struct.pack('<I', self.stack_return), struct.pack('<I', self.stack_return)),
        ]
        for i, (_, start, data, _) in enumerate(self.regions):
            require(start > 0 and start+len(data) <= 2**32, 'invalid region')
            for _, other, b, _ in self.regions[:i]:
                require(start+len(data) <= other or other+len(b) <= start, 'overlapping regions')
        require(all(not (a <= self.stop < a+len(b)) for _, a, b, _ in self.regions), 'stop overlaps captured memory')


class Replay:
    def __init__(self, capsule, omit=None, writable=True):
        self.c = capsule
        self.regions = [r for r in capsule.regions if r[0] != omit]
        self.writable = [(capsule.self_+16, 2), (capsule.self_+0x214, 4)] if writable else []
        self.u = Uc(UC_ARCH_X86, UC_MODE_32)
        pages = {}
        for name, address, before, _ in self.regions:
            flags = UC_PROT_READ | (UC_PROT_EXEC if name == 'code' else 0)
            if name == 'package' and writable:
                flags |= UC_PROT_WRITE
            for page in range(address & ~4095, (address+len(before)+4095) & ~4095, 4096):
                pages[page] = pages.get(page, 0) | flags
        for page, flags in pages.items():
            self.u.mem_map(page, 4096, flags)
        for _, address, before, _ in self.regions:
            self.u.mem_write(address, before)
        for reg, value in zip(REG_IDS, capsule.before_regs):
            self.u.reg_write(reg, value)
        self.reads, self.writes, self.instructions = [], [], 0
        self.u.hook_add(UC_HOOK_MEM_READ | UC_HOOK_MEM_WRITE, self.access)
        self.u.hook_add(UC_HOOK_CODE, self.code)

    def access(self, uc, kind, address, size, value, _):
        if kind == UC_MEM_WRITE:
            require(any(a <= address and address+size <= a+n for a, n in self.writable),
                    f'uncaptured write {address:08x}+{size}')
            self.writes.append((address, size, value & ((1 << (size*8))-1)))
        else:
            require(any(a <= address and address+size <= a+len(b) for _, a, b, _ in self.regions),
                    f'uncaptured read {address:08x}+{size}')
            self.reads.append((address, size))

    def code(self, uc, address, size, _):
        require(self.c.entry <= address and address+size <= self.c.entry+CODE_SIZE,
                f'uncaptured instruction/import {address:08x}+{size}')
        self.instructions += 1

    def run(self):
        try:
            self.u.emu_start(self.c.entry, self.c.stop, count=100)
        except UcError as error:
            raise ValueError(f'uncaptured/protected access or invalid execution: {error}') from error
        require(self.u.reg_read(x.UC_X86_REG_EIP) == self.c.stop, 'instruction budget exhausted')
        actual_regs = tuple(self.u.reg_read(reg) for reg in REG_IDS)
        for name, actual, expected in zip(REG_NAMES, actual_regs, self.c.after_regs):
            require(actual == expected, f'{name}: replay {actual:08x} != live {expected:08x}')
        for name, address, _, expected in self.regions:
            require(bytes(self.u.mem_read(address, len(expected))) == expected, f'live output mismatch: {name}')
        expected_writes = [(self.c.self_+16, 2, 0), (self.c.self_+0x214, 4, self.c.seed_before)]
        require(self.writes == expected_writes, 'unexpected write sequence')
        return {'instructions': self.instructions, 'reads': self.reads, 'writes': self.writes,
                'registers': dict(zip(REG_NAMES, actual_regs))}


def replay(install, directory):
    identity = json.loads((directory / 'capsule-image.json').read_text())
    require(identity['schema'] == 1, 'unknown identity schema')
    for name, path in [('source', install / 'riseofnations.exe'),
                       ('staged_source', directory / 'riseofnations.exe'),
                       ('traced', directory / 'riseofnations_trace.exe'),
                       ('tracer', directory / 'rontrace.dll')]:
        require(digest(path) == identity['sha256'][name], f'image identity mismatch: {name}')
    c = Capsule((directory / 'capsule.bin').read_bytes())
    records = read_trace(directory / 'rontrace.log')
    receipts = [r for r in records if r[:2] == (5, 120)]
    require(len(receipts) == 1 and receipts[0][2:] == (0, c.self_, c.frame, 1224, 1224, c.frame), 'missing or inconsistent live capture receipt')
    require(not any(r[:2] == (5, 121) for r in records), 'capsule hook failure')
    copies = [r for r in records if r[:2] == (5, 122)]
    size = struct.unpack_from('<H', c.before, 16)[0]
    require(len(copies) == 1 and copies[0][2:] == (0, size, 0, 0, 0, c.frame), 'live source package was not preserved')
    require(c.code == image_bytes(install / 'riseofnations.exe', c.entry, 24), 'captured code differs from pinned original')
    first, second = Replay(c).run(), Replay(c).run()
    require(first == second, 'fresh replay differs')
    for omit in ('singleton', 'game_seed', 'return'):
        try:
            Replay(c, omit=omit).run()
        except ValueError:
            pass
        else:
            raise ValueError(f'missing {omit} did not fail')
    return {'schema': 1, 'source_sha256': identity['sha256']['source'],
            'capsule_sha256': digest(directory / 'capsule.bin'), 'frame': c.frame, 'fresh_replays': 2,
            'captured_bytes': sum(len(b) for _, _, b, _ in c.regions),
            'missing_dependency_controls': 3, 'result': first}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('action', choices=['bind', 'replay'])
    ap.add_argument('install', type=Path)
    ap.add_argument('directory', type=Path)
    args = ap.parse_args()
    if args.action == 'bind':
        bind(args.install, args.directory)
    else:
        print(json.dumps(replay(args.install, args.directory), indent=2))


if __name__ == '__main__':
    main()
