#!/usr/bin/env python3
"""Stream an A* return census; experimental mode requires complete metadata receipts.

Reports contain original-derived observations and belong outside the repo.
--archive accepts pre-existing proxy traces without census instrumentation.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import struct


def require(ok, message):
    if not ok:
        raise ValueError(message)


def records(path, archive=False):
    with path.open('rb') as f:
        header = f.read(32)
        require(len(header) == 32, 'missing trace header')
        h = struct.unpack('<8I', header)
        require(h[0] == 0x544e4f52 and h[1] in ((1, 2) if archive else (2,)) and
                h[2] == 0x400000, 'unsupported trace identity')
        while block := f.read(32*4096):
            require(len(block) % 32 == 0, 'partial trace record')
            yield from struct.iter_unpack('<8I', block)


def scan(rows, experimental=True):
    stack, counts, events = [], Counter(), []
    armed, proxies, pending, total, capped = 0, [], None, 0, False
    frame_count, last_frame = 0, None
    for r in rows:
        kind, tag = r[:2]
        if kind == 2:
            frame_count += 1
            last_frame = r[1]
        if kind == 5:
            require(tag not in (2, 5, 14), f'trace health error {tag}')
            if tag == 12:
                proxies.append(r[2])
            if experimental and tag == 136:
                require(r[2:7] == (0, 1, 64, 0, 0), 'census refused or wrong version')
                armed += 1
            if experimental and tag == 133:
                raise ValueError('census memory read failed')
            if experimental and tag == 135:
                require(total == 65 and not capped and r[2:7] == (65, 64, 0, 0, 0), 'invalid cap receipt')
                capped = True
            if experimental and 130 <= tag <= 134:
                require(pending is not None and r[2] == total and r[7] == pending['frame'], 'orphan census record')
                if tag == 130:
                    require('unit' not in pending and r[6] == 0, 'duplicate census header')
                    pending.update(unit=r[3], limit=r[4], saving=r[5])
                elif tag in (131, 132):
                    require('unit' in pending, 'metadata before census header')
                    name, maximum = ('containers', 5) if tag == 131 else ('pools', 7)
                    require(r[3] == len(pending[name]) and r[3] < maximum, 'missing/duplicate census slot')
                    pending[name].append(list(r[4:7]))
                elif tag == 134:
                    require(r[3:7] == (5, 7, 0, 0) and len(pending['containers']) == 5 and len(pending['pools']) == 7,
                            'incomplete census completion')
                    require(pending['unit'] != 0 and pending['containers'][0][0] != 0, 'suspension has no owner/open container')
                    events.append(pending)
                    pending = None
        if kind == 7 and tag == 0:
            require(len(stack) < 32, 'excessive A* nesting')
            stack.append(r)
        if kind == 8 and tag == 0:
            require(stack, 'unmatched A* return')
            call = stack.pop()
            require(call[7] == r[7], 'A* crosses recorded frames')
            if experimental:
                require(r[7] == last_frame, 'A* return has no matching observed frame')
            result = r[2] if r[2] < 2**31 else r[2]-2**32
            counts[(call[4], call[5], result)] += 1
            if result == -1:
                total += 1
                if experimental:
                    require(pending is None, 'missing census completion')
                    if total <= 64:
                        pending = {'sequence': total, 'frame': r[7], 'step': call[4], 'anti': call[5], 'containers': [], 'pools': []}
    require(not stack, 'unfinished A* call')
    require(pending is None, 'unfinished census')
    if experimental:
        require(armed == 1 and proxies == [0x283770], 'missing/duplicate or nonminimal census instrumentation')
        require(len(events) == min(total, 64) and capped == (total > 64), 'missing census records')
    return {'frame_records': frame_count, 'last_frame': last_frame, 'astar_returns': sum(counts.values()), 'suspension_returns': total,
            'histogram': [{'step': a, 'anti': b, 'result': c, 'count': n} for (a, b, c), n in sorted(counts.items())],
            'status': 'suspension_returns_without_metadata' if not experimental and total else
                      'capped' if capped else 'witnesses' if events else
                      'no_game_frames' if experimental and frame_count == 0 else 'no_suspension_witness', 'events': events}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('trace', type=Path)
    ap.add_argument('--archive', action='store_true')
    args = ap.parse_args()
    print(json.dumps(scan(records(args.trace, args.archive), not args.archive), indent=2))


if __name__ == '__main__':
    main()
