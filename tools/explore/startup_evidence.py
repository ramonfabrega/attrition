#!/usr/bin/env python3
"""Summarize retained startup witnesses, including incomplete/failed traces.

Diagnostic only: never validates capture success or infers the cause of a hang.
Reads at most one record at a time and retains bounded exception addresses.
"""
import argparse
import json
from pathlib import Path
import struct


def inspect(path):
    result = {'diagnostic_only': True, 'header_supported': False,
              'records': 0, 'trailing_bytes': 0, 'driver_events': {},
              'startup_events': [], 'startup_event_count': 0,
              'frames_observed': 0, 'last_frame': None, 'fault_addresses': [], 'fault_contexts': []}
    with path.open('rb') as stream:
        while data := stream.read(32):
            if len(data) != 32:
                result['trailing_bytes'] = len(data)
                break
            row = struct.unpack('<8I', data)
            if result['records'] == 0:
                result['header_supported'] = row[:3] == (0x544e4f52, 2, 0x400000)
                if not result['header_supported']:
                    result['records'] = 1
                    return result
            elif row[0] == 5 and 170 <= row[1] <= 177:
                key = str(row[1])
                result['driver_events'][key] = result['driver_events'].get(key, 0) + 1
                if row[1] == 176 and len(result['fault_addresses']) < 8:
                    result['fault_addresses'].append(f'0x{row[2]:08x}')
                    result['fault_contexts'].append({'ip': row[2], 'sp': row[3],
                                                     'bp': row[4], 'access_address': row[5]})
            elif row[:2] == (5, 180):
                result['startup_event_count'] += 1
                if len(result['startup_events']) < 16:
                    result['startup_events'].append({'site': row[2], 'phase': row[3],
                                                     'version': row[4], 'flags': row[5], 'result': row[6]})
            elif row[0] == 2:
                result['frames_observed'] += 1
                result['last_frame'] = row[1]
            result['records'] += 1
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('trace', type=Path)
    print(json.dumps(inspect(parser.parse_args().trace), indent=2))


if __name__ == '__main__':
    main()
