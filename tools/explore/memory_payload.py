#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Stream-validate an optional broad candidate packet. Never inspect a live process.

A validated packet is not an atomic snapshot or a complete replay dependency set.
Original-derived payloads and reports stay outside git.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from memory_inventory import decode as decode_inventory, validate as validate_inventory
from restore_context import decode_context
from replay_capsule import require
from search_census import records

CAP, CHUNK, MAX_MS = 1024**3, 1024**2, 5000


def plan(inventory, context, cap=CAP):
    rows = inventory['rows']
    observer = next(r for r in rows if r[0] <= inventory['observer'] < r[0]+r[3])
    require(observer[4] == 0x1000 and observer[6] == 0x1000000 and
            observer[1] != inventory['main'], 'invalid observer allocation')
    low, high = context['tib'][2], context['tib'][1]
    spans = []
    total = 0
    for index, (base, allocation, _, size, state, protect, kind) in enumerate(rows):
        size = min(size, inventory['end']-base)
        if (state != 0x1000 or protect & 0x100 or protect & 0xff not in (2, 4, 8)
                or allocation == observer[1] or (base < high and base+size > low)
                or not (kind == 0x20000 or (kind == 0x1000000 and allocation == inventory['main']))):
            continue
        total += size
        require(total <= cap, 'payload candidate exceeds cap')
        spans.append((index, base, size))
    require(spans, 'empty payload plan')
    return spans, total, observer[1]


def anchors(context, world):
    return [(context['unit'], context['unit_data']), (context['origin'], context['table']),
            (0xcab3ac, struct.pack('<I', context['origin'])),
            (0xcae5fc, struct.pack('<I', context['center'])), (0xc06188, struct.pack('<I', world))]


def decode(stream, inventory_raw, prefix_raw, context):
    require(context['version'] == 2, 'payload needs full unit context')
    inventory = decode_inventory(inventory_raw, prefix_raw)
    require((inventory['frame'], inventory['unit']) == (context['frame'], context['unit']),
            'payload inventory/context identity differs')
    spans, total, observer = plan(inventory, context)
    digest = hashlib.sha256()
    position = 0

    def read(size):
        nonlocal position
        data = stream.read(size)
        require(len(data) == size, 'truncated payload')
        digest.update(data); position += size
        return data

    h = struct.unpack('<16I', read(64))
    world = h[13]
    expected = (0x31504d52, 1, inventory['frame'], inventory['unit'], len(spans), total,
                CAP, MAX_MS, CHUNK, len(inventory_raw), context['tib'][2], context['tib'][1],
                observer, world, 0, 0)
    require(h == expected, 'payload header differs from policy/context')
    require(read(len(inventory_raw)) == inventory_raw, 'payload inventory binding differs')
    watched = anchors(context, world)
    seen = [0]*len(watched)
    described = []
    for index, base, size in spans:
        require(struct.unpack('<3I', read(12)) == (index, base, size), 'payload range order/extent differs')
        offset = position
        range_digest = hashlib.sha256()
        for done in range(0, size, CHUNK):
            data = read(min(CHUNK, size-done)); address = base+done
            range_digest.update(data)
            for j, (anchor, expected_bytes) in enumerate(watched):
                left, right = max(address, anchor), min(address+len(data), anchor+len(expected_bytes))
                if left < right:
                    require(data[left-address:right-address] == expected_bytes[left-anchor:right-anchor],
                            f'payload anchor differs at {left:x}')
                    seen[j] += right-left
        described.append({'inventory_index': index, 'base': hex(base), 'bytes': size,
                          'file_offset': offset, 'sha256': range_digest.hexdigest()})
    require(seen == [len(data) for _, data in watched], 'payload omits an anchor')
    footer = struct.unpack('<8I', read(32))
    require(footer[:4] == (0x31444e45, 1, len(spans), total) and 0 <= footer[4] < MAX_MS and
            footer[5:] == (world, sum(seen), 0), 'invalid payload completion footer')
    require(stream.read(1) == b'', 'extra payload bytes')
    return {'frame': inventory['frame'], 'unit': inventory['unit'], 'payload_bytes': total,
            'file_bytes': position, 'range_count': len(spans), 'world_pointer': hex(world),
            'footer_elapsed_ms': footer[4], 'anchor_bytes': sum(seen), 'sha256': digest.hexdigest(),
            'spans': described, 'atomic_snapshot': False, 'replay_closure_established': False}


def check_receipts(rows, report):
    successes = [(i, r) for i, r in enumerate(rows) if r[:2] == (5, 167)]
    require(len(successes) == 1 and not any(r[:2] == (5, 168) for r in rows),
            'missing/duplicate payload success or failure receipt')
    finish, receipt = successes[0]
    require(receipt[2:6] == (0, report['unit'], report['payload_bytes'], report['range_count']) and
            receipt[7] == report['frame'] and report['footer_elapsed_ms'] <= receipt[6] < MAX_MS,
            'payload receipt differs')
    inventories = [i for i, r in enumerate(rows) if r[:2] == (5, 165)]
    require(len(inventories) == 1, 'missing/duplicate inventory receipt')
    begin = inventories[0]
    following = next((i for i, r in enumerate(rows[begin+1:], begin+1) if r[:2] == (7, 0)), len(rows))
    require(begin < finish < following < len(rows), 'payload outside delegation event')
    return receipt[6]


def validate(install, directory):
    validate_inventory(install, directory)
    prefix = (directory/'restore-prefix.bin').read_bytes()
    context = decode_context((directory/'restore-context.bin').read_bytes(), prefix)
    with (directory/'memory-payload.bin').open('rb') as stream:
        report = decode(stream, (directory/'memory-inventory.bin').read_bytes(), prefix, context)
    report['receipt_elapsed_ms'] = check_receipts(list(records(directory/'rontrace.log')), report)
    return report


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path); ap.add_argument('directory', type=Path)
    args = ap.parse_args(); print(json.dumps(validate(args.install, args.directory), indent=2))


if __name__ == '__main__':
    main()
