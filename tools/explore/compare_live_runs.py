#!/usr/bin/env python3
"""Assert the bounded live rendering experiment against a normal control run.

compare_live_runs.py CONTROL_DIRECTORY HIDDEN_DIRECTORY
Both directories contain gamelog.txt and rontrace.log. Reads only; originals
and whole-state values remain outside the repository. This checks the logged
UNITS=3 frame bodies, not every byte of the game's memory.
"""
from pathlib import Path
import argparse
import json
import re
import struct


def read_trace(path):
    raw = path.read_bytes()
    assert len(raw) >= 32 and len(raw) % 32 == 0, f'incomplete trace: {path}'
    header = struct.unpack('<8I', raw[:32])
    assert header[:3] == (0x544e4f52, 2, 0x400000), 'unexpected trace header'
    records = list(struct.iter_unpack('<8I', raw[32:]))
    assert not any(r[0] == 5 and r[1] in (2, 5, 14) for r in records), 'trace health error'
    return records


def read_frames(path):
    parts = re.split(r'(?m)^\s*BEGIN FRAME (\d+)\s*$', path.read_text())
    pairs = [(int(parts[i]), parts[i + 1].strip()) for i in range(1, len(parts) - 1, 2)]
    assert len(dict(pairs)) == len(pairs), 'duplicate frame labels'
    frames = dict(pairs)
    assert set(frames) == set(range(18, 36)) | {37}, 'unexpected dump window'
    return frames


def check(root):
    records = read_trace(root / 'rontrace.log')
    frames = read_frames(root / 'gamelog.txt')
    receipts = [r for r in records if r[:2] == (5, 100)]
    assert len(receipts) == 1 and receipts[0][2] == 0, 'command issuance failed'
    r = receipts[0]
    assert r[7] == 20 and r[4] - r[3] == 27, 'unexpected issuance frame/size'
    before = [r for r in records if r[:2] == (5, 101) and r[2] == 20]
    assert len(before) == 1
    assert (r[5], r[6]) == (before[0][3] + 2560, before[0][4]), 'coordinate decoding error'
    assert f'process_move_to {r[5]} {r[6]} 2 0 0 0 0' in frames[21], 'missing processed command'
    rng = [r for r in records if r[0] == 2]
    assert [r[1] for r in rng] == list(range(37)), 'incomplete frame trace'
    return records, frames, rng


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('control', type=Path)
    ap.add_argument('hidden', type=Path)
    args = ap.parse_args()
    _, control, rng_control = check(args.control)
    records, hidden, rng_hidden = check(args.hidden)
    for frame in control:
        assert control[frame] == hidden[frame], f'logged state differs at frame {frame}'
    assert rng_control == rng_hidden, 'frame RNG records differ'
    markers = [r for r in records if r[:2] == (5, 120)]
    assert [(r[2], r[3], r[4]) for r in markers] == [(18, 0, 3), (35, 3, 0)], 'suppression not restored'
    rendered = [r[7] for r in records if r[:2] == (7, 9)]
    assert any(f < 18 for f in rendered) and any(f >= 35 for f in rendered), 'render witness absent'
    assert not any(18 <= f < 35 for f in rendered), 'render ran during suppression'
    print(json.dumps({'equal_logged_frame_bodies': len(control),
                      'equal_frame_rng_records': len(rng_control),
                      'suppressed_sim_frames': 17, 'scene_render_calls_in_window': 0,
                      'render_calls_outside_window': len(rendered)}, indent=2))


if __name__ == '__main__':
    main()
