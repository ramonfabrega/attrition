#!/usr/bin/env python3
"""gates.py — a steering pass's table of a tranche's booking gates, from their logs.

    python3 tools/gates.py                       # $CLAUDE_JOB_DIR/tmp/gate-*.log
    python3 tools/gates.py ~/.claude/jobs/<commander>/tmp --since gate-a5d76b6b.log
    python3 tools/gates.py gate-*.log --json

One row a log: whole or scoped (and whether the scope widened to every
timed test), the minutes between the log's birth and its last write, the
`Gate steps:`, `Admission wait:` and `Host load:` receipts, the release
step's minutes where the gate printed `Gate minutes:`, and the verdict —
green, red with the failed tests named, or `no receipt` for a log that
ends without the gate's last lines (a killed gate). The totals under the
table are what a pass's measurement table carries. Three passes wrote
this probe by hand (the fortieth, forty-first and forty-second); the
third use graduates it (CLAUDE.md, "Conventions").
"""
import argparse
import datetime
import json
import os
import re
import sys
from pathlib import Path

NUM = r'([\d.]+|n/a)'


def _num(s):
    return None if s in (None, 'n/a') else float(s)


def row(path):
    path = Path(path)
    text = path.read_text(errors='replace')
    lines = text.splitlines()
    st = path.stat()
    born = getattr(st, 'st_birthtime', None)
    minutes = round((st.st_mtime - born) / 60, 1) if born and st.st_mtime >= born else None
    out = dict(name=path.name.removesuffix('.log'), path=str(path),
               started=datetime.datetime.fromtimestamp(born or st.st_mtime).strftime('%H:%M'),
               ended=datetime.datetime.fromtimestamp(st.st_mtime).strftime('%H:%M'), minutes=minutes,
               scope='whole', steps=None, release_min=None, admission_s=None, leases=None, longest_s=None,
               load_start=None, load_end=None, cores=None, verdict='no receipt', failed=[])
    for line in lines[:3]:
        m = re.match(r'Booking gate scoped to \S+: (.*)', line)
        if m:
            rest = m.group(1)
            left = re.match(r'(\d+) timed tests left', rest)
            out['scope'] = ('scoped, widened' if rest.startswith('every timed test')
                            else f'scoped, {left.group(1)} left' if left else 'scoped')
        elif line.startswith('Lane suite:'):
            out['scope'] = 'lane'
    # The gate runs the tools' own tests, whose fixtures print receipt
    # lines of their own; the gate's receipt is the LAST of each, and its
    # failed tests are the lines under the last `Test summary:`.
    last = lambda pattern: (re.findall(pattern, text, re.M) or [None])[-1]  # noqa: E731
    steps = last(r'^Gate steps: (\d+ of \d+) ran(.*)$')
    if steps:
        out['steps'] = steps[0]
    m = last(r'^Gate minutes: .*?\brelease ([\d.]+)')
    if m:
        out['release_min'] = float(m)
    m = last(r'^Admission wait: ([\d.]+) s over (\d+) leases, longest ([\d.]+) s')
    if m:
        out['admission_s'], out['leases'], out['longest_s'] = float(m[0]), int(m[1]), float(m[2])
    m = last(rf'^Host load: {NUM} at start, {NUM} at end, on (\d+) cores')
    if m:
        out['load_start'], out['load_end'], out['cores'] = _num(m[0]), _num(m[1]), int(m[2])
    summaries = [i for i, l in enumerate(lines) if l.startswith('Test summary: ')]
    summary = lines[summaries[-1]] if summaries else ''
    tail = lines[summaries[-1] + 1:] if summaries else []
    for line in tail:
        m = re.match(r'\s+failed: (\S+)', line)
        if not m:
            break
        out['failed'].append(m.group(1))
    exits = re.findall(r'^exit (\d+)$', text, re.M)
    if steps:
        red = ('not reached' in steps[1] or out['failed'] or ' FAILED ' in summary or (exits and exits[-1] != '0'))
        out['verdict'] = ('red: ' + ', '.join(out['failed']) if out['failed'] else 'red') if red else 'green'
    return out


def rows(paths):
    return [row(p) for p in paths]


def totals(rows):
    waits = [r['admission_s'] for r in rows if r['admission_s'] is not None]
    longest = [r['longest_s'] for r in rows if r['longest_s'] is not None]
    minutes = [r['minutes'] for r in rows if r['minutes'] is not None]
    return dict(gates=len(rows), whole=sum(r['scope'] == 'whole' for r in rows),
                scoped=sum(r['scope'].startswith('scoped') for r in rows),
                widened=sum(r['scope'] == 'scoped, widened' for r in rows),
                lane=sum(r['scope'] == 'lane' for r in rows),
                red=sum(r['verdict'].startswith('red') for r in rows),
                no_receipt=sum(r['verdict'] == 'no receipt' for r in rows),
                minutes=round(sum(minutes), 1), admission_s=round(sum(waits), 1),
                longest_s=max(longest) if longest else None,
                past_cores=sum(1 for r in rows if r['load_end'] is not None and r['cores'] and r['load_end'] > r['cores']))


def render(rows, totals):
    head = f"{'gate':<16} {'start':>5} {'min':>5} {'scope':<17} {'steps':<7} {'rel':>5} {'adm s':>7} {'long':>6} {'load':>11} verdict"
    out = [head]
    for r in rows:
        load = (f"{r['load_start']:.1f}→{r['load_end']:.1f}" if r['load_start'] is not None and r['load_end'] is not None
                else '-')
        out.append(f"{r['name']:<16} {r['started']:>5} {str(r['minutes'] if r['minutes'] is not None else '-'):>5} "
                   f"{r['scope']:<17} {str(r['steps'] or '-'):<7} {str(r['release_min'] if r['release_min'] is not None else '-'):>5} "
                   f"{str(r['admission_s'] if r['admission_s'] is not None else '-'):>7} "
                   f"{str(r['longest_s'] if r['longest_s'] is not None else '-'):>6} {load:>11} {r['verdict']}")
    t = totals
    out.append(f"{t['gates']} gates: {t['whole']} whole, {t['scoped']} scoped ({t['widened']} widened), {t['red']} red"
               + (f", {t['no_receipt']} without a receipt" if t['no_receipt'] else '')
               + (f", {t['lane']} lane" if t['lane'] else '') + f"; {t['minutes']} minutes in all")
    out.append(f"admission {t['admission_s']} s summed, longest {t['longest_s']} s; {t['past_cores']} ended past the cores")
    return '\n'.join(out) + '\n'


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('paths', nargs='*', help='gate logs, or a directory of gate-*.log')
    ap.add_argument('--since', help='keep only logs written after this one (a name or a path)')
    ap.add_argument('--json', action='store_true')
    args = ap.parse_args(argv)
    paths = args.paths or [os.path.join(os.environ.get('CLAUDE_JOB_DIR', '.'), 'tmp')]
    logs = []
    for p in paths:
        p = Path(p)
        logs += sorted(p.glob('gate-*.log')) if p.is_dir() else [p]
    if args.since:
        since = Path(args.since)
        if not since.is_file():
            since = next((l for l in logs if l.name == since.name), None)
        if since is None or not since.is_file():
            print(f'gates.py: --since {args.since} is not a log here', file=sys.stderr)
            return 2
        logs = [l for l in logs if l.stat().st_mtime > since.stat().st_mtime]
    logs.sort(key=lambda l: l.stat().st_mtime)
    table = rows(logs)
    total = totals(table)
    if args.json:
        print(json.dumps(dict(rows=table, totals=total), indent=2))
    else:
        sys.stdout.write(render(table, total))
    return 0


if __name__ == '__main__':
    sys.exit(main())
