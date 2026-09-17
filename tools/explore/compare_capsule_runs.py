#!/usr/bin/env python3
"""Compare the capsule-copy run with a normal command-probe control."""
import argparse
import json
from pathlib import Path
from compare_live_runs import check


def main():
    if not __debug__:
        raise ValueError("the reused scenario checker requires Python assertions enabled")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('control', type=Path)
    ap.add_argument('capsule', type=Path)
    args = ap.parse_args()
    normal, frames, rng = check(args.control)
    probe, other, other_rng = check(args.capsule)
    if frames != other or rng != other_rng:
        raise ValueError('logged frame bodies or frame/RNG records differ')
    if any(r[0] == 5 and r[1] in (120, 121, 122) for r in normal):
        raise ValueError('control contains capsule or suppression markers')
    receipts = [r for r in probe if r[:2] == (5, 120)]
    copies = [r for r in probe if r[:2] == (5, 122)]
    issued = [r for r in probe if r[:2] == (5, 100)]
    if not (len(receipts) == len(copies) == len(issued) == 1
            and receipts[0][2] == copies[0][2] == 0
            and receipts[0][5:7] == (1224, 1224)
            and copies[0][3] == issued[0][4] and copies[0][4] == 0
            and copies[0][7] == receipts[0][7] == 20
            and not any(r[:2] == (5, 121) for r in probe)):
        raise ValueError('missing or inconsistent capsule/copy evidence')
    print(json.dumps({'equal_logged_frame_bodies': len(frames),
                      'equal_frame_rng_records': len(rng),
                      'live_package_preserved': True}, indent=2))


if __name__ == '__main__':
    main()
