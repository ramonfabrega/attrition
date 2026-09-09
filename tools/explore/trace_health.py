#!/usr/bin/env python3
"""Read-only framing/loss census, not a proof that a trace is complete.

Usage: python3 tools/explore/trace_health.py LOG_DIRECTORY
Examines rontrace-run*.log; prints metadata only, never game records.
"""
import collections
import json
from pathlib import Path
import struct
import sys
import time

start = time.perf_counter()
rows = []
for path in sorted(Path(sys.argv[1]).glob('rontrace-run*.log')):
    with path.open('rb') as stream:
        header = stream.read(32)
        if len(header) != 32 or header[:4] != b'RONT':
            raise ValueError(f'{path.name}: missing RONT header')
        info = collections.Counter()
        records = 0
        tail = 0
        while block := stream.read(32 * 32768):
            tail = len(block) % 32
            for record in struct.iter_unpack('<8I', block[:len(block) - tail]):
                records += 1
                if record[0] == 5:
                    info[record[1]] += record[2] if record[1] == 14 else 1
        rows.append(dict(file=path.name, bytes=path.stat().st_size,
                         records=records, version=struct.unpack_from('<I', header, 4)[0],
                         tail=tail, dropped=info[14], hook_mismatch=info[2],
                         protect_fail=info[5], detach=info[7]))
assert rows, 'no trace files matched'
print(json.dumps(dict(files=len(rows),
    totals={key: sum(row[key] for row in rows) for key in
            ['bytes', 'records', 'dropped', 'hook_mismatch', 'protect_fail']},
    versions=dict(collections.Counter(row['version'] for row in rows)),
    without_detach=sum(not row['detach'] for row in rows),
    anomalies=[row for row in rows if any(row[key] for key in
               ['tail', 'dropped', 'hook_mismatch', 'protect_fail'])],
    elapsed_s=time.perf_counter() - start), indent=2))
