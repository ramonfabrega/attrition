#!/usr/bin/env python3
"""Validate a no-click driver's lifecycle; process exit is NOT capture success.

This checks transport/lifecycle only, not simulation fidelity or map identity.
"""
import argparse
import json
from pathlib import Path
import struct


def receipt(data, end_frame, exit_code):
    if len(data) < 32 or len(data) % 32:
        raise ValueError('missing or truncated trace')
    return receipt_records(struct.iter_unpack('<8I', data), end_frame, exit_code)


def receipt_file(path, end_frame, exit_code):
    def records():
        with path.open('rb') as stream:
            while chunk := stream.read(65536):
                if len(chunk) % 32:
                    raise ValueError('truncated trace')
                yield from struct.iter_unpack('<8I', chunk)
    return receipt_records(records(), end_frame, exit_code)


def receipt_records(records, end_frame, exit_code):
    if not 0 <= end_frame <= 24000:
        raise ValueError('invalid endpoint')
    rows = iter(records)
    header = next(rows, ())
    if header[:3] != (0x544E4F52, 2, 0x400000):
        raise ValueError('missing or unsupported trace header')
    events, frame_count = [], 0
    for row in rows:
        # Match the finalized Rust reader's transport checks. A complete
        # lifecycle cannot make a lossy or internally inconsistent trace valid.
        if row[0] == 5 and row[1] == 14 and row[2] != 0:
            raise ValueError('trace reports dropped records')
        if row[0] == 5 and 170 <= row[1] <= 177:
            events.append(row)
            if len(events) > 6:
                raise ValueError('extra lifecycle or fault records')
        elif row[0] == 2:
            if row[1] != row[7]:
                raise ValueError('trace FRAME fields disagree')
            if row[1] != frame_count or frame_count > end_frame:
                raise ValueError('missing, repeated, or unexpected simulation frames')
            frame_count += 1
    if exit_code != 0:
        raise ValueError(f'process failed: {exit_code}')
    if any(r[1] in (174, 176, 177) for r in events):
        raise ValueError('driver refused or native fault recorded')
    if [r[1] for r in events] != [175, 170, 171, 172, 173, 170]:
        raise ValueError('incomplete, repeated, or reordered lifecycle')
    if events[0][2] != 1 or [(r[2], r[3]) for r in events if r[1] == 170] != [(1, 1), (21, 2)]:
        raise ValueError('unexpected install/menu result')
    setup, start, returned = events[2:5]
    if not setup[2] or setup[3] != 0 or start[2] != setup[2] or returned[2] != setup[2]:
        raise ValueError('setup instance/mode mismatch')
    if frame_count != end_frame + 1:
        raise ValueError('missing, repeated, or unexpected simulation frames')
    return {'lifecycle_verified': True, 'frames': frame_count, 'end_frame': end_frame,
            'map_verified': False, 'fidelity_verified': False}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('trace', type=Path)
    ap.add_argument('--end-frame', type=int, required=True)
    ap.add_argument('--exit-code', type=int, required=True)
    args = ap.parse_args()
    print(json.dumps(receipt_file(args.trace, args.end_frame, args.exit_code), indent=2))


if __name__ == '__main__':
    main()
