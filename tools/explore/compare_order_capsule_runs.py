#!/usr/bin/env python3
"""Compare a natural order-capsule run with a normal command-probe control."""
import argparse
import json
from pathlib import Path


from compare_natural_capsule_runs import compare


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('control', type=Path); ap.add_argument('capsule', type=Path)
    args = ap.parse_args()
    print(json.dumps(compare(args.control, args.capsule), indent=2))
