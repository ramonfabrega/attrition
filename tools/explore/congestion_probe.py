#!/usr/bin/env python3
"""Reproduce and validate the opt-in two-group congestion experiment.

schedule prints rontrace.cmd; report validates one trace; compare validates two
and requires the observed RNG/order/search projections to agree. Reports and
traces are original-derived and must stay outside git. No full-state parity
claim follows from these projections.
"""
import argparse
import hashlib
import json
from pathlib import Path
from search_census import records, require, scan


def schedule():
    return '\n'.join(['37 !ffwd 2'] + [
        f'{150+i} !add hoplite who=0 {14+i%4},{166+i//4}' for i in range(32)
    ] + ['1400 !quit']) + '\n'


def scenario(rows):
    frames = [(r[1], r[2]) for r in rows if r[0] == 2]
    require([f for f, _ in frames] == list(range(1401)), 'incomplete/duplicate scenario frames')
    info = [r for r in rows if r[0] == 5]
    require(not any(r[1] in (10, 144) for r in info), 'scenario command/read/identity failure')
    commands = [r for r in info if r[1] == 9]
    expected = [37] + list(range(150, 182)) + [1400]
    require(len(commands) == len(expected), 'missing/extra scenario command')
    for i, (r, f) in enumerate(zip(commands, expected)):
        require(r[2:] == (f, i, 0, 1, 0, f), 'wrong/rejected scenario command')
    roster = [r for r in info if r[1] == 140]
    require(len(roster) == 32 and len({r[3] for r in roster}) == 32, 'missing/duplicate spawned captain')
    require([r[2] for r in roster] == [i % 2 for i in range(32)] and
            all(r[7] == 210 for r in roster), 'wrong roster group/frame')
    summaries = [r for r in info if r[1] == 141]
    require(len(summaries) == 1 and summaries[0][2:4] == (16, 16) and
            summaries[0][7] == 210, 'missing/incorrect roster summary')
    orders = [r for r in info if r[1] == 142]
    targets = [r for r in info if r[1] == 143]
    order_frames = [220+200*i+j for i in range(5) for j in range(2)]
    require(len(orders) == len(targets) == 10, 'missing/extra order receipt')
    for i, (order, target, f) in enumerate(zip(orders, targets, order_frames)):
        require(order[2:4] == (i % 2, 16) and order[6:] == (f, f) and
                0 <= order[4] < order[5] <= 512, 'wrong/rejected order receipt')
        direction = 1 if (i % 2 + i//2) % 2 else -1
        require(target[2:] == (i % 2, summaries[0][4]+direction*1536,
                              summaries[0][5], 0, 0, f), 'wrong target receipt')
    return {'frames': frames, 'roster': roster, 'summary': summaries, 'orders': orders, 'targets': targets}


def digest(value):
    return hashlib.sha256(json.dumps(value, separators=(',', ':')).encode()).hexdigest()


def report(path):
    rows = list(records(path))
    census = scan(iter(rows))
    observed = scenario(rows)
    calls, returns = [], []
    for r in rows:
        if r[:2] == (7, 0):
            calls.append(r)
        elif r[:2] == (8, 0):
            call = calls.pop()
            returns.append((r[7], call[4], call[5], r[2]))
    owners, shapes = {}, []
    for e in census['events']:
        # Preserve repeated ownership, but addresses vary across processes.
        owner = owners.setdefault(e['unit'], len(owners))
        shapes.append((e['sequence'], e['frame'], owner, e['limit'], e['saving'],
                       [(bool(p), n, bool(root)) for p, n, root in e['containers']],
                       [(bool(p), cap, n) for p, cap, n in e['pools']]))
    projections = {k: digest(v) for k, v in observed.items()}
    projections.update(astar_returns=digest(returns), metadata=digest(shapes))
    return {'status': census['status'], 'frames': len(observed['frames']),
            'spawned_captains': len(observed['roster']), 'issued_orders': len(observed['orders']),
            'astar_returns': census['astar_returns'], 'suspension_returns': census['suspension_returns'],
            'metadata_events': len(shapes), 'first_suspension_frame': shapes[0][1] if shapes else None,
            'max_container_lengths': [max((e['containers'][i][1] for e in census['events']), default=0)
                                      for i in range(5)], 'histogram': census['histogram'],
            'projections': projections}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest='mode', required=True)
    sub.add_parser('schedule')
    one = sub.add_parser('report'); one.add_argument('trace', type=Path)
    two = sub.add_parser('compare'); two.add_argument('trace', type=Path); two.add_argument('repeat', type=Path)
    args = ap.parse_args()
    if args.mode == 'schedule':
        print(schedule(), end=''); return
    result = report(args.trace)
    if args.mode == 'compare':
        repeat = report(args.repeat)
        require(result == repeat, 'repeat differs in observed scenario/search projections')
        result['repeat_matches'] = True
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
