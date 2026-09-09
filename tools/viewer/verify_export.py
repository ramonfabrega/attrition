#!/usr/bin/env python3
"""Check exported coordinates against the existing harness's summary counts."""
import json
import re
import sys
from pathlib import Path


def verify(data):
    if data.get('schema') != 1 or not data.get('frames'):
        raise ValueError('unsupported or empty export')
    focus = data.get("focusIndex")
    if focus is not None and (type(focus) is not int or not 0 <= focus < len(data["frames"])):
        raise ValueError("failure focus outside exported window")
    compared = mismatches = 0
    previous = None
    for frame in data['frames']:
        def require(condition, message):
            if not condition:
                raise ValueError(f"source frame {frame['index']}: {message}")
        require(previous is None or frame['index'] == previous + 1, 'nonconsecutive source indices')
        previous = frame['index']
        if 'differences' in frame:
            rows = frame['differences']
            require(all(type(d['weight']) is int and d['weight'] > 0 for d in rows), 'invalid issue weight')
            require(sum(d['weight'] for d in rows) == frame['issues'], 'field rows lose reported issues')
            require(all(isinstance(d[k], str) for d in rows for k in ('entity', 'field', 'original', 'rust', 'assessment')), 'field values must remain lossless strings')
        units = frame['units']
        require(len({u['id'] for u in units}) == len(units), 'duplicate unit identity')
        paired = [u for u in units if u['scope'] and u['original'] is not None and u['rust'] is not None]
        different = sum(u['original'] != u['rust'] for u in paired)
        require(len(paired) == frame['compared'], 'coordinate rows disagree with harness comparison count')
        require(different == frame['positionMismatches'], 'coordinates disagree with harness mismatch count')
        compared += len(paired)
        mismatches += different
    return len(data['frames']), compared, mismatches


def read(path):
    match = re.search(r'<script type="application/json" id="replay-data">(.*?)</script>', Path(path).read_text(), re.S)
    if match is None:
        raise ValueError('embedded replay data missing')
    return json.loads(match[1])


if __name__ == '__main__':
    try:
        frames, compared, mismatches = verify(read(sys.argv[1]))
        print(f'{frames} frames; {compared} paired positions; {mismatches} disagreements: counts match harness')
    except (ValueError, KeyError, IndexError, TypeError) as error:
        sys.exit(str(error))
