#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Explore a validated retained payload with explicit optional runtime models.

Prints packet-derived JSON: redirect it outside git. Every missing-data attempt
is discarded before a fresh runner receives another captured page fragment.
No live process, native DLL execution, implicit data zeros, or fidelity claim.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct
import time
from unicorn import UC_HOOK_CODE, UC_HOOK_MEM_INVALID, UC_MEM_READ, x86_const as x
from bounded_call import Region
from memory_inventory import decode as decode_inventory
from memory_payload import validate
from modeled_allocator import AllocatorCall
from modeled_memory_fill import MemoryFillCall
from modeled_memory_copy import MemoryCopyCall
from modeled_lifetime import LifetimeCall
from replay_capsule import require, REG_IDS
from restore_context import decode_context
from resume_frontier import regions_for, registers, ENTRY, RETURN
from search_census import records
from thread_context_probe import ThreadCall

IMAGE_SHA256 = '30478a44b577cb11ebcbbbf53d3e93ba02fd2aacf3bdefa6552c9b6449625079'
GDT, ARENA_SIZE = 0x71000000, 1024*1024


def subtract_regions(left, right, regions):
    cuts = [(left, right)]
    for r in regions:
        next_cuts = []
        for lo, hi in cuts:
            start, end = r.address, r.address+len(r.data)
            if start >= hi or end <= lo:
                next_cuts.append((lo, hi))
            else:
                if lo < start:
                    next_cuts.append((lo, start))
                if end < hi:
                    next_cuts.append((end, hi))
        cuts = next_cuts
    return cuts


class CapturedPages:
    def __init__(self, stream, spans, rows):
        self.stream, self.spans, self.rows = stream, spans, rows

    def span(self, address, size):
        return next((s for s in self.spans if int(s['base'], 16) <= address and
                     address+size <= int(s['base'], 16)+s['bytes']), None)

    def read(self, address, size):
        s = self.span(address, size)
        if s is None:
            return None
        self.stream.seek(s['file_offset']+address-int(s['base'], 16))
        data = self.stream.read(size)
        require(len(data) == size, 'short captured read')
        return data

    def expand(self, address, size, regions):
        s = self.span(address, size)
        if s is None:
            return []
        lo = max(address & ~4095, int(s['base'], 16))
        hi = min((address+size+4095) & ~4095, int(s['base'], 16)+s['bytes'])
        protection = self.rows[s['inventory_index']][5] & 255
        return [Region(f'payload_{left:x}', left, self.read(left, right-left),
                       writable=protection in (4, 8))
                for left, right in subtract_regions(lo, hi, regions)]


def original_code(raw):
    require(hashlib.sha256(raw).hexdigest() == IMAGE_SHA256, 'unsupported original image')
    pe = struct.unpack_from('<I', raw, 60)[0]
    opt = pe+24
    base = struct.unpack_from('<I', raw, opt+28)[0]
    count = struct.unpack_from('<H', raw, pe+6)[0]
    opt_size = struct.unpack_from('<H', raw, pe+20)[0]
    result = []
    for i in range(count):
        h = opt+opt_size+40*i
        _, rva, size, offset = struct.unpack_from('<4I', raw, h+8)
        if not struct.unpack_from('<I', raw, h+36)[0] & 0x20000000:
            continue
        address = base+rva
        # The stop sentinel authorizes stopping, not executing that byte.
        stop = [Region('stop', RETURN, bytes(1))]
        for left, right in subtract_regions(address, address+size, stop):
            result.append(Region(f'original_code_{left:x}', left,
                                 raw[offset+left-address:offset+right-address], executable=True))
    return result


def mutable_callee_arguments(regions):
    """This pinned thiscall consumes six words (RET 0x18 at 0x68371e).

    Only those input parameter slots become writable. The return address and
    the extra captured caller-stack tail keep their original read-only policy.
    """
    result = []
    for r in regions:
        if r.name != 'arguments':
            result.append(r)
            continue
        require(len(r.data) == 52 and not r.writable and not r.scratch and not r.executable,
                'unexpected delegation argument layout')
        result.extend([Region('callee_return', r.address, r.data[:4]),
                       Region('callee_arguments', r.address+4, r.data[4:28], writable=True),
                       Region('caller_tail', r.address+28, r.data[28:])])
    require(any(r.name == 'callee_arguments' for r in result), 'missing delegation arguments')
    return result


def free_arena(rows, regions, address=None):
    candidates = [address] if address is not None else [r[0] for r in rows if r[0] >= 0x10000000]
    for start in candidates:
        if start % 16 or not any(r[4] == 0x10000 and r[0] <= start and
                                 start+ARENA_SIZE <= r[0]+r[3] for r in rows):
            continue
        occupied = [*regions, Region('gdt', GDT, bytes(24)), Region('stop', RETURN, bytes(1))]
        if subtract_regions(start, start+ARENA_SIZE, occupied) == [(start, start+ARENA_SIZE)]:
            return start
    raise ValueError('no nonoverlapping captured-free arena')


def import_target(pages, slot):
    word = pages.read(slot, 4)
    require(word is not None, 'import slot absent from packet')
    target = struct.unpack('<I', word)[0]
    require(any(r[0] <= target < r[0]+r[3] and r[4] == 0x1000 and
                not r[5] & 0x100 and r[5] & 255 in (0x10, 0x20, 0x40, 0x80) and r[6] == 0x1000000
                for r in pages.rows), 'import target is not an executable image range')
    return target


def declare_borrowed(pages, regions, borrowed):
    require(len(borrowed) <= 64 and all(0 < n <= 1024*1024 for _, n in borrowed) and
            sum(n for _, n in borrowed) <= 1024*1024,
            'borrowed object cap exceeded')
    added, checked = [], []
    for address, size in borrowed:
        require(all(address+size <= a or a+n <= address for a, n in checked), 'overlapping borrowed objects')
        checked.append((address, size))
        span = pages.span(address, size)
        require(span is not None, 'borrowed object outside captured candidate')
        require(pages.rows[span['inventory_index']][5] & 255 in (4, 8), 'borrowed object is read-only')
        data = pages.read(address, size)
        existing = next((r for r in regions if r.address == address and len(r.data) == size), None)
        if existing is not None:
            require(existing.writable and not existing.scratch and not existing.executable and
                    existing.data == data, 'borrowed object differs from declared input')
            continue
        require(subtract_regions(address, address+size, [*regions, *added]) == [(address, address+size)],
                'borrowed object overlaps declared region')
        added.append(Region(f'borrowed_{address:x}', address, data, writable=True))
    return added


def parse_borrow(value):
    address, size = value.split(':')
    return int(address, 0), int(size, 0)


def native_return(directory):
    armed, depth = False, 0
    for row in records(directory/'rontrace.log'):
        if row[:2] == (5, 167):
            armed = True
        elif armed and row[:2] == (7, 0):
            depth += 1
        elif armed and row[:2] == (8, 0):
            require(depth > 0, 'unmatched native A* return')
            depth -= 1
            if not depth:
                return row[2]
    raise ValueError('native A* return absent')


def observe_unit_path(runner, unit):
    # Owned PDB: Unit.path +0xb8, Stack<PathData> 16 bytes; PathData 16 bytes.
    # This is a bounded post-state observation, not a native comparison.
    runner.access(runner.uc, UC_MEM_READ, unit, 0x158, 0, None)
    unit_data = bytes(runner.uc.mem_read(unit, 0x158))
    pointer, capacity, length = struct.unpack_from('<3I', unit_data, 0xb8)
    require(capacity <= 4096 and length <= capacity, 'unexpected final path bounds')
    data = b''
    if capacity:
        runner.access(runner.uc, UC_MEM_READ, pointer, capacity*16, 0, None)
        data = bytes(runner.uc.mem_read(pointer, capacity*16))
    return dict(unit_address=hex(unit), unit_bytes=unit_data.hex(), path_address=hex(pointer),
                capacity=capacity, length=length, path_slot_bytes=data.hex(),
                path_sha256=hashlib.sha256(data).hexdigest(), native_compared=False)


def fingerprint(runner, error):
    """State of this model, including backing bytes; not a native-state claim."""
    memory = hashlib.sha256()
    for r in runner.regions:
        if not r.executable:
            memory.update(struct.pack('<II', r.address, len(r.data)))
            memory.update(bytes(runner.uc.mem_read(r.address, len(r.data))))
    models = {name: getattr(runner, name, []) for name in ('allocations', 'fills', 'copies', 'frees', 'retired')}
    return dict(error=error, instructions=runner.instructions,
                registers=[runner.uc.reg_read(reg) for reg in REG_IDS],
                declared_memory_sha256=memory.hexdigest(),
                initialized_sha256=hashlib.sha256(json.dumps(sorted(runner.initialized)).encode()).hexdigest(),
                model_state_sha256=hashlib.sha256(json.dumps(models, sort_keys=True).encode()).hexdigest(),
                writes_sha256=hashlib.sha256(json.dumps(runner.writes).encode()).hexdigest())


def explore(install, directory, *, services='none', budget=1000000, rounds=128,
            arena=None, perturb=False, borrowed=(), mutable_arguments=False, repeat_final=0, observe_path=False):
    require(services in ('none', 'malloc', 'malloc+memset', 'malloc+memset+memcpy', 'malloc+memset+memcpy+free'), 'unknown service policy')
    require(1 <= budget <= 1000000 and 1 <= rounds <= 128, 'replay limit outside policy')
    require(0 <= repeat_final <= 16, 'repeat count outside policy')
    require(not borrowed or services.endswith('+free'), 'borrowing requires explicit free policy')
    started = time.monotonic()
    report = validate(install, directory)
    prefix = (directory/'restore-prefix.bin').read_bytes()
    context = decode_context((directory/'restore-context.bin').read_bytes(), prefix)
    rows = decode_inventory((directory/'memory-inventory.bin').read_bytes(), prefix)['rows']
    image = install/'riseofnations.exe'
    code = original_code(image.read_bytes())
    regions = [r for r in regions_for(image, context, (directory/'search-graph.bin').read_bytes(), True)
               if not r.executable] + code
    if mutable_arguments:
        regions = mutable_callee_arguments(regions)
    kwargs = dict(fs_address=context['teb'], gdt_address=GDT, budget=budget)
    history = []
    expected = native_return(directory)
    with (directory/'memory-payload.bin').open('rb') as stream:
        pages = CapturedPages(stream, report['spans'], rows)
        for r in regions:
            if not r.executable and not r.scratch:
                captured = pages.read(r.address, len(r.data))
                require(captured is None or captured == r.data, 'baseline/payload overlap differs: '+r.name)
        borrowed_regions = declare_borrowed(pages, regions, borrowed)
        regions.extend(borrowed_regions)
        kind = ThreadCall
        if services != 'none':
            kwargs.update(malloc_target=import_target(pages, 0xac54f0),
                          arena_address=free_arena(rows, regions, arena),
                          arena_size=ARENA_SIZE, max_allocations=1024)
            kind = AllocatorCall
        if services in ('malloc+memset', 'malloc+memset+memcpy', 'malloc+memset+memcpy+free'):
            kwargs['memset_target'] = import_target(pages, 0xac5424)
            kind = MemoryFillCall
        if services in ('malloc+memset+memcpy', 'malloc+memset+memcpy+free'):
            kwargs['memcpy_target'] = import_target(pages, 0xac5428)
            kind = MemoryCopyCall
        if services.endswith('+free'):
            kwargs.update(free_target=import_target(pages, 0xac5500), borrowed=borrowed)
            kind = LifetimeCall
        for attempt in range(rounds):
            runner = kind(regions, RETURN, **kwargs)
            if perturb:
                for i in range(8):
                    runner.uc.reg_write(getattr(x, f'UC_X86_REG_XMM{i}'), (i+1)*(2**120+17))
                    runner.uc.reg_write(getattr(x, f'UC_X86_REG_FP{i}'), (1 << 63, 0x4000+i))
                runner.uc.reg_write(x.UC_X86_REG_MXCSR, 0x3f80)
                runner.uc.reg_write(x.UC_X86_REG_FPCW, 0x27f)
                runner.initial_context = runner.uc.context_save()
            astar, faults = [], []
            runner.uc.hook_add(UC_HOOK_CODE, lambda u, a, n, d:
                               astar.append(u.reg_read(x.UC_X86_REG_EAX)) if a == 0x68335e else None)
            runner.uc.hook_add(UC_HOOK_MEM_INVALID, lambda u, k, a, n, v, d:
                               faults.append((k, a, n)) or False)
            raw_error = None
            try:
                runner.run(ENTRY, registers(context))
                last = dict(returned=True, instructions=runner.instructions,
                            eax=runner.uc.reg_read(x.UC_X86_REG_EAX))
                break
            except ValueError as error:
                pc, reason = runner.uc.reg_read(x.UC_X86_REG_EIP), str(error)
                raw_error = reason
                last = dict(returned=False, pc=hex(pc), instructions=runner.instructions, reason=reason)
                missing = re.fullmatch(r'undeclared access ([0-9a-f]+)\+(\d+)', reason)
                if missing:
                    address, size = int(missing[1], 16), int(missing[2])
                elif len(faults) == 1 and 'UNMAPPED' in reason:
                    address, size = faults[0][1:]
                else:
                    break
                event = dict(pc=hex(pc), instructions=runner.instructions, address=hex(address), bytes=size)
                history.append(event)
                added = pages.expand(address, size, regions)
                if not added:
                    last.update(address=hex(address), bytes=size,
                                reason='outside captured candidate' if pages.span(address, size) is None
                                else 'fault overlaps declared bytes')
                    break
                if attempt+1 == rounds:
                    last['reason'] = 'dependency round cap reached'
                    break
                regions.extend(added)
        last.update(astar_returns=astar, native_astar_return=expected,
                    astar_return_matches_native=(astar == [expected]) if astar else None,
                    allocations=getattr(runner, 'allocations', []), fills=getattr(runner, 'fills', []),
                    copies=getattr(runner, 'copies', []), frees=getattr(runner, 'frees', []),
                    writes_count=len(runner.writes),
                    writes_sha256=hashlib.sha256(json.dumps(runner.writes).encode()).hexdigest())
        expected_state, expected_returns = fingerprint(runner, raw_error), list(astar)
        for _ in range(repeat_final):
            astar.clear(); faults.clear()
            repeat_error = None
            try:
                runner.run(ENTRY, registers(context))
            except ValueError as error:
                repeat_error = str(error)
            require(fingerprint(runner, repeat_error) == expected_state and astar == expected_returns,
                    'reused final context differs')
        last['final_fingerprint'] = expected_state
        if observe_path and last['returned']:
            last['unit_path_observation'] = observe_unit_path(runner, context['unit'])
        stack = runner.uc.reg_read(x.UC_X86_REG_ESP)
        last['stack_words'] = []
        for address in range(stack, stack+16, 4):
            r = runner.region(address, 4)
            available = r is not None and (not r.scratch or
                        all(a in runner.initialized for a in range(address, address+4)))
            last['stack_words'].append(struct.unpack('<I', runner.uc.mem_read(address, 4))[0]
                                       if available else None)
        return dict(payload_sha256=report['sha256'], image_sha256=IMAGE_SHA256,
                    services=services, repeated_final_runs=repeat_final, mutable_callee_arguments=mutable_arguments, limits=dict(instructions=budget, rounds=rounds,
                    allocations=1024, arena_bytes=ARENA_SIZE, fill_bytes=1024*1024, fills=1024, copy_bytes=1024*1024, copies=1024, frees=1024, borrowed_objects=64, borrowed_bytes=1024*1024),
                    arena_address=kwargs.get('arena_address'),
                    borrowed_objects=[dict(address=a, size=n) for a, n in borrowed],
                    borrowed_bytes=sum(n for _, n in borrowed),
                    added_borrowed_bytes=sum(len(r.data) for r in borrowed_regions),
                    baseline_overlap_conflicts=[], added_bytes=sum(len(r.data) for r in regions
                    if r.name.startswith('payload_')), dependency_rounds=history, last=last,
                    seconds=time.monotonic()-started, extended_state_perturbed=perturb,
                    fxsave_imported=False, fidelity_claim=False)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path)
    ap.add_argument('directory', type=Path)
    ap.add_argument('--services', choices=('none', 'malloc', 'malloc+memset', 'malloc+memset+memcpy', 'malloc+memset+memcpy+free'), default='none')
    ap.add_argument('--budget', type=int, default=1000000)
    ap.add_argument('--rounds', type=int, default=128)
    ap.add_argument('--arena', type=lambda s: int(s, 0))
    ap.add_argument('--observe-unit-path', action='store_true',
                    help='after return, retain the complete unit and all path slots; no native equivalence claim')
    ap.add_argument('--repeat-final', type=int, default=0,
                    help='0..16 resets of the final region set; compare GPR, memory, initialization and model state')
    ap.add_argument('--mutable-arguments', action='store_true',
                    help='allow stores to six callee-owned parameter words; return and caller tail remain read-only')
    ap.add_argument('--borrow', action='append', default=[], type=parse_borrow, metavar='ADDRESS:BYTES',
                    help='explicit modeled object extent from captured writable data; not a native heap assertion')
    ap.add_argument('--perturb-extended-state', action='store_true')
    args = ap.parse_args()
    print(json.dumps(explore(args.install, args.directory, services=args.services,
                            budget=args.budget, rounds=args.rounds, arena=args.arena,
                            perturb=args.perturb_extended_state, borrowed=args.borrow,
                            mutable_arguments=args.mutable_arguments, repeat_final=args.repeat_final, observe_path=args.observe_unit_path), indent=2))


if __name__ == '__main__':
    main()
