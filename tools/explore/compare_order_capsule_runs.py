#!/usr/bin/env python3
"""Compare a natural order-capsule run with a normal command-probe control."""
import argparse
import json
from pathlib import Path
from compare_live_runs import check


def compare(control, capsule):
    if not __debug__:
        raise ValueError('the reused scenario checker requires Python assertions enabled')
    normal, frames, rng = check(control)
    probe, other, other_rng = check(capsule)
    if frames != other or rng != other_rng:
        raise ValueError('logged frame bodies or frame/RNG records differ')
    if any(r[0] == 5 and r[1] in (120, 121, 122, 124, 125) for r in normal):
        raise ValueError('control contains capsule or suppression markers')
    receipts = [r for r in probe if r[:2] == (5, 124)]
    if not (len(receipts) == 1 and receipts[0][2] == 0 and
            receipts[0][5:7] == (240, 240) and 0 <= receipts[0][4] <= 35 and
            receipts[0][4] == receipts[0][7] and
            not any(r[0] == 5 and r[1] in (120, 121, 122, 125) for r in probe)):
        raise ValueError('missing or inconsistent natural capsule evidence')
    return {'equal_logged_frame_bodies': len(frames), 'equal_frame_rng_records': len(rng),
            'natural_call_frame': receipts[0][4]}


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('control', type=Path); ap.add_argument('capsule', type=Path)
    args = ap.parse_args()
    print(json.dumps(compare(args.control, args.capsule), indent=2))
