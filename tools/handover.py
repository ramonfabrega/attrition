#!/usr/bin/env python3
"""handover.py — one driver row hands the integration tree to the other.

`docs/DECISIONS.md` entries 130 and 131 §8 (Ramon, 2026-11-13: spawn and
reap). Two rows drive this tree, `commander` and `steer`, one at a time.
Clearing the other row and typing a prompt into it failed twice: a row that
rested on its context woke cold on a notification, and ccc declines to type
a prompt onto a row that is already empty. So a handover spawns the other
row fresh, with its opener as the spawn's prompt, and the new row's first
act reaps the old one. No pane is typed into and no row rests.

  handover.py to <commander|steer> --opener-file <file>
      The handing row's last act. Refuses a dirty or unpushed tree and a
      background command still running under this row (the reap would kill
      it). Reaps every other driver row of this tree, spawns the new one,
      and prints its ref. The turn ends after it.
  handover.py reap <ref>
      The new row's first act: waits for <ref> to go idle, stops it and
      forgets it. Never `ccc rm`: that verb reaches the tree.
  handover.py check
      Exit 0 when at most one driver row of this tree has a process, 1
      otherwise. `tools/gate_run.sh` runs it before a gate.
"""
import argparse
import json
import os
import subprocess
import sys
import time

DRIVERS = {
    'commander': ('claude-opus-5-5', 'high'),
    'steer': ('claude-fable-5-2', 'high'),
}
IDLE_WAIT_S = 180
STOP_WAIT_S = 90


def run(argv, stdin=None):
    return subprocess.run(argv, input=stdin, capture_output=True, text=True)


def tree():
    return run(['git', 'rev-parse', '--show-toplevel']).stdout.strip()


def roster():
    """ccc's rows, every one. A row is `{job, session, worktree, archived}`."""
    out = run(['ccc', 'list', '--json'])
    if out.returncode != 0:
        raise SystemExit(f'handover: `ccc list --json` exited {out.returncode}: {out.stderr.strip()}')
    rows = json.loads(out.stdout)
    return rows if isinstance(rows, list) else rows.get('rows', [])


def driver_rows(rows, cwd):
    """The rows that can wake in this tree: a driver's name, this tree's
    path, not folded away, and a process behind them."""
    held = []
    for row in rows:
        session = row.get('session') or {}
        if session.get('name') not in DRIVERS:
            continue
        if os.path.realpath(session.get('cwd') or '') != os.path.realpath(cwd):
            continue
        if row.get('archived') or session.get('state') == 'stopped':
            continue
        held.append(row)
    return held


def describe(row):
    session = row['session']
    return f"{session.get('name')} {session.get('id')} ({session.get('state')})"


def check_line(rows, cwd):
    """`(ok, line)`: at most one driver row may have a process."""
    held = driver_rows(rows, cwd)
    if len(held) <= 1:
        who = describe(held[0]) if held else 'none listed'
        return True, f'Rows: one driver row at most: {who}'
    names = ', '.join(describe(row) for row in held)
    return False, (f'Rows: {len(held)} driver rows hold this tree: {names} — '
                   'reap the one that handed over: python3 tools/handover.py reap <ref> '
                   '(docs/DECISIONS.md entry 130)')


def own_ref():
    job = os.environ.get('CLAUDE_JOB_DIR', '').rstrip('/')
    return os.path.basename(job) if job else None


SHELLS = ('zsh', 'bash', 'sh')


def strays(pid, table, me):
    """The tool commands still running under the row `pid`: what a reap of
    the row would kill. `table` is `{pid: (ppid, command)}`. A tool call's
    command runs under a shell the row started, so a stray is a shell that
    is the row's direct child and not this script's own ancestor. The
    row's other children are the harness's own — its MCP servers, its
    `caffeinate` — and die with it by design."""
    lineage = set()
    walk = me
    while walk in table and walk not in lineage:
        lineage.add(walk)
        walk = table[walk][0]
    found = []
    for child, (parent, command) in table.items():
        if parent != pid or child in lineage:
            continue
        words = command.split()
        if words and os.path.basename(words[0]).lstrip('-') in SHELLS:
            found.append((child, command))
    return sorted(found)


def process_table():
    out = run(['ps', '-Ao', 'pid=,ppid=,command=']).stdout
    table = {}
    for line in out.splitlines():
        parts = line.split(None, 2)
        if len(parts) == 3:
            table[int(parts[0])] = (int(parts[1]), parts[2])
    return table


def said(out):
    text = (out.stdout or out.stderr).strip()
    try:
        return json.loads(text).get('said', text)[:300]
    except (ValueError, AttributeError):
        return text[:300]


def reap(ref):
    """Stop the row and drop it from the roster, keeping the tree."""
    deadline = time.time() + IDLE_WAIT_S
    while True:
        rows = roster()
        row = next((r for r in rows if (r.get('session') or {}).get('id') == ref), None)
        if row is None:
            print(f'handover: {ref} is not on the roster; nothing to reap')
            return 0
        if (row.get('job') or {}).get('tempo') != 'active' or time.time() > deadline:
            break
        time.sleep(5)
    out = run(['ccc', 'stop', ref, '--json'])
    print(f'handover: ccc stop {ref}: exit {out.returncode} {said(out)}')
    # A stop is answered before the process is gone, and `forget` refuses a
    # row that still has one: wait for the roster to say stopped.
    deadline = time.time() + STOP_WAIT_S
    while time.time() < deadline:
        row = next((r for r in roster() if (r.get('session') or {}).get('id') == ref), None)
        if row is None or (row.get('session') or {}).get('state') == 'stopped':
            break
        time.sleep(3)
    out = run(['ccc', 'forget', ref, '--json'])
    print(f'handover: ccc forget {ref}: exit {out.returncode} {said(out)}')
    rows = roster()
    row = next((r for r in rows if (r.get('session') or {}).get('id') == ref), None)
    if row is not None and (row.get('session') or {}).get('state') != 'stopped':
        print(f'handover: {ref} still has a process after stop and forget', file=sys.stderr)
        return 1
    if row is not None:
        out = run(['ccc', 'archive', ref])
        print(f'handover: forget kept the row; ccc archive {ref}: exit {out.returncode}')
    return 0


def hand_to(name, opener_file):
    cwd = tree()
    me = own_ref()
    if me is None:
        raise SystemExit('handover: CLAUDE_JOB_DIR is not set; `to` is a row\'s own act')
    dirty = run(['git', 'status', '--porcelain']).stdout.strip()
    if dirty:
        raise SystemExit(f'handover: the tree is dirty; bank first:\n{dirty}')
    branch = run(['git', 'rev-parse', '--abbrev-ref', 'HEAD']).stdout.strip()
    run(['git', 'fetch', '-q', 'origin', branch])
    head = run(['git', 'rev-parse', 'HEAD']).stdout.strip()
    pushed = run(['git', 'rev-parse', f'origin/{branch}']).stdout.strip()
    if head != pushed:
        raise SystemExit(f'handover: HEAD {head[:9]} is not origin/{branch} {pushed[:9]}; push the gated tip first')
    rows = roster()
    mine = next((r for r in rows if (r.get('session') or {}).get('id') == me), None)
    if mine is None:
        raise SystemExit(f'handover: this row ({me}) is not on ccc\'s roster')
    left = strays(mine['session'].get('pid'), process_table(), os.getpid())
    if left:
        lines = '\n'.join(f'  {pid} {command[:160]}' for pid, command in left)
        raise SystemExit('handover: a command is still running under this row, and the reap would kill it:\n' + lines)
    for row in driver_rows(rows, cwd):
        ref = row['session']['id']
        if ref != me and reap(ref) != 0:
            raise SystemExit(f'handover: could not reap {describe(row)}')
    model, effort = DRIVERS[name]
    opener = open(opener_file).read().strip()
    prompt = (f'Your first act, before anything else: `python3 tools/handover.py reap {me}` '
              f'(the row that handed over to you; docs/DECISIONS.md entries 130 and 131 §8). '
              f'Then:\n\n{opener}\n')
    out = run(['ccc', 'spawn', '--name', name, '--cwd', cwd, '--model', model, '--effort', effort,
               '--json', '-'], stdin=prompt)
    if out.returncode != 0:
        raise SystemExit(f'handover: ccc spawn exited {out.returncode}: {(out.stderr or out.stdout).strip()[:600]}')
    answer = json.loads(out.stdout)
    if answer.get('worktree'):
        raise SystemExit(f'handover: the spawn cut a worktree, which a driver row never has: {out.stdout[:400]}')
    print(f"handover: spawned {name} on {model}: {json.dumps(answer)[:400]}")
    print('handover: this row is reaped by the new one; end the turn now, asking nothing')
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sub = parser.add_subparsers(dest='verb', required=True)
    to = sub.add_parser('to')
    to.add_argument('name', choices=sorted(DRIVERS))
    to.add_argument('--opener-file', required=True)
    sub.add_parser('reap').add_argument('ref')
    sub.add_parser('check')
    args = parser.parse_args(argv)
    if args.verb == 'check':
        ok, line = check_line(roster(), tree())
        print(line)
        return 0 if ok else 1
    if args.verb == 'reap':
        return reap(args.ref)
    return hand_to(args.name, args.opener_file)


if __name__ == '__main__':
    sys.exit(main())
