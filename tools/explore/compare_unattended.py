#!/usr/bin/env python3
"""Compare all logged frame bodies and frame/seed pairs in two acquisitions.

Checks both lifecycle receipts and all expected logging frames, including the
closing record. This compares observed projections, not complete native state.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct

from autostart_receipt import receipt_file
from unattended_capture import verify_game


def bodies(path):
    result, frame, digest = {}, None, None
    with path.open('rb') as stream:
        for line in stream:
            line = line.rstrip(b'\r\n')
            match = re.fullmatch(rb'\s*BEGIN FRAME (\d+)', line)
            if match:
                if frame is not None: result[frame] = digest.hexdigest()
                frame = int(match[1])
                if frame in result: raise ValueError(f'repeated logged frame {frame}')
                digest = hashlib.sha256()
            elif frame is not None and line.startswith(b' '):
                # Retain every indented record/field; omit only ambient chatter
                # at column zero, following tools/gamelog/samegame.py.
                digest.update(line + b'\n')
        if frame is not None: result[frame] = digest.hexdigest()
    return result


def seeds(path):
    result = {}
    with path.open('rb') as stream:
        while block := stream.read(65536):
            if len(block) % 32: raise ValueError('truncated trace')
            for row in struct.iter_unpack('<8I', block):
                if row[0] == 2:
                    if row[1] in result: raise ValueError('repeated seed frame')
                    result[row[1]] = row[2]
    return result


def compare(left, right):
    configs = [json.loads((p/'receipt.json').read_text()) for p in (left,right)]
    for report in configs:
        if not report.get('success') or not report.get('settings_restored'):
            raise ValueError('acquisition did not succeed and restore settings')
    keys = ('map_style','end_frame','seed_requested')
    if any(configs[0][k] != configs[1][k] for k in keys):
        raise ValueError('different map, endpoint, or requested seed')
    style, end, seed = (configs[0][k] for k in keys)
    expected = set(range(18, min(36,end))) | {end+1}
    frame_bodies, frame_seeds = [], []
    for path, report in zip((left,right),configs):
        receipt_file(path/'rontrace.log',end,report['exit_code'])
        verify_game(path/'gamelog.txt',style,end,seed)
        frame_bodies.append(bodies(path/'gamelog.txt'))
        frame_seeds.append(seeds(path/'rontrace.log'))
    if any(set(b) != expected for b in frame_bodies):
        raise ValueError('logged frame coverage differs from configured window/closing record')
    for label, pair in (('logged body',frame_bodies),('frame seed',frame_seeds)):
        for frame in sorted(pair[0]):
            if pair[0][frame] != pair[1].get(frame):
                raise ValueError(f'first differing {label}: frame {frame}')
    return {'map_style':style, 'end_frame':end, 'logged_bodies_equal':len(expected),
            'closing_record_compared':True, 'frame_seed_pairs_equal':end+1,
            'complete_state_parity':False}


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('left',type=Path)
    ap.add_argument('right',type=Path)
    args=ap.parse_args()
    print(json.dumps(compare(args.left,args.right),indent=2))


if __name__=='__main__': main()
