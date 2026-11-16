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
  handover.py to commander --opener-file <file> --ungated
      The same, from a row past its ceiling whose tip is committed and not
      gated: the pushed check is waived and the new row is told to gate
      that tip first. Refused under the ceiling.
  handover.py wake
      What a morning wake's prompt asks of whatever stands in the row: a
      handover to a fresh `commander` whose opener is the queue's own
      `**Opener:` paragraph. A session's cron fires into the row as it then
      stands — another model, a conversation — so the wake is a spawn.
  handover.py check [--scoped] [--tip <sha>]
      `tools/gate_run.sh` runs it before a gate, and it prints the `Rows:`
      and `Driver:` lines the log carries. Exit 1 when two driver rows of
      this tree have a process; 3 when a scoped gate is asked for more than
      BACKSTOP + SEATS landings after the last pass; 4 when the calling
      row's next turn carries more than CEILING_K.

The driver's two lines (`docs/DECISIONS.md` entry 135). The tranche of
2026-11-14 and 15 booked fifty landings without a pass against a backstop
of twenty, and its driver went from 81 k to 536 k without naming its 400 k
line: both were prose. A landing is a first-parent merge after the commit
that added the newest pass record (`PASS_RECORDS`); `tools/brief.py`
refuses a brief at BACKSTOP, and a scoped gate is refused once the count
passes what the seats could still hold. The depth is `tools/ctx.py`'s.
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

# Twenty landings without a pass is a boundary (DECISIONS 126 §4); the four
# seats (three lanes and the chain's builder, DECISIONS 129 §1) are what can
# still be out when the count reaches it.
BACKSTOP = 20
SEATS = 4
# The driver clears when its next turn would carry more than 400 k
# (DECISIONS 131 §6). Its context grew 38 k an hour on 2026-11-15 and a
# builder ran 24 to 104 minutes, so fifty thousand above the line is one
# builder and its booking gate; past it no gate starts.
CLEAR_AT_K = 400
CEILING_K = 450
PASS_RECORDS = 'docs/audit/*-fable-pass-*.md'
WAIVER = 'ATTRITION_BACKSTOP_WAIVED'


def run(argv, stdin=None):
    return subprocess.run(argv, input=stdin, capture_output=True, text=True)


def tree():
    return run(['git', 'rev-parse', '--show-toplevel']).stdout.strip()


def roster(fresh=False):
    """ccc's rows, every one. A row is `{job, session, worktree, archived}`.
    `ccc list` is answered by the running app's roster, a tick old; `fresh`
    polls once more first, for a read that follows a write of ours."""
    out = run(['ccc', 'list', '--json'] + (['--fresh'] if fresh else []))
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


def landings_since_pass(cwd=None, tip='HEAD'):
    """`(count, sha)`: the first-parent merges on `tip` after the commit
    that added the newest pass record, and that commit. `(None, None)`
    where no record was ever added. A record amended later is not a new
    pass; a merge is a worker's branch, since the driver merges nothing
    else."""
    out = subprocess.run(['git', 'log', '--first-parent', '--diff-filter=A', '--format=%H', '-1', tip,
                          '--', PASS_RECORDS], cwd=cwd, capture_output=True, text=True)
    sha = out.stdout.strip()
    if out.returncode != 0 or not sha:
        return None, None
    out = subprocess.run(['git', 'rev-list', '--first-parent', '--merges', '--count', f'{sha}..{tip}'],
                         cwd=cwd, capture_output=True, text=True)
    if out.returncode != 0:
        return None, None
    return int(out.stdout.strip()), sha


def brief_line(count, waiver=None):
    """`(ok, line)` for a lane's brief: none is composed at the backstop."""
    if count is None or count < BACKSTOP or waiver:
        return True, ''
    return False, (f'{count} landings since the last pass, and the backstop is {BACKSTOP}: no lane is '
                   'refilled. Book what is out, gate whole, and hand over: python3 tools/handover.py '
                   'to steer --opener-file <file> (docs/LOOP.md, "Supply calls the pass")')


def landings_line(count, scoped, waiver=None):
    """`(ok, line)` for a gate: a scoped one is refused once the count is
    past the backstop and the seats; the drain's whole gate never is."""
    if count is None:
        return True, 'landings since the last pass not read (no pass record in this history)'
    said = f'{count} landings since the last pass (backstop {BACKSTOP})'
    if count < BACKSTOP:
        return True, said
    if waiver:
        return True, f'{said}, waived: {waiver}'
    if scoped and count > BACKSTOP + SEATS:
        return False, (f'{said}: past the backstop and the {SEATS} seats it could find out. No scoped '
                       'gate: gate whole (no --scope) and hand over with python3 tools/handover.py to steer')
    return True, f'{said}: backstop reached — refill nothing, book what is out, gate whole, hand to steer'


def depth_line(ctx):
    """`(ok, line)` for the row that asks for a gate, on the context its
    next turn carries (`tools/ctx.py`)."""
    if ctx is None:
        return True, 'context not read'
    k = ctx // 1000
    if k > CEILING_K:
        return False, (f'context {k} k, past the {CEILING_K} k ceiling. No gate from this row: commit, '
                       'write the handoff, and python3 tools/handover.py to commander --opener-file '
                       '<file> --ungated; the fresh row gates this tip')
    if k >= CLEAR_AT_K:
        return True, (f'context {k} k, past the {CLEAR_AT_K} k line: cut no builder, bank on this gate '
                      f'and clear ({CEILING_K} k refuses the gate)')
    return True, f'context {k} k'


def own_ctx():
    """This row's context, or None: `tools/ctx.py`'s reading of `lore
    agents` for `CLAUDE_CODE_SESSION_ID`."""
    session = os.environ.get('CLAUDE_CODE_SESSION_ID')
    if not session:
        return None
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import ctx
    env = dict(os.environ)
    home = os.path.expanduser('~')
    env['PATH'] = f"{home}/.local/bin:{home}/.bun/bin:/opt/homebrew/bin:" + env.get('PATH', '')
    try:
        out = subprocess.run(['lore', 'agents', '--json'], capture_output=True, text=True, env=env, timeout=60)
        return ctx.ctx_of(ctx.rows_of(json.loads(out.stdout)), session)
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return None


def queue_opener(text):
    """The queue's `**Opener:` paragraph, to its blank line: one, or a
    ValueError that says how many."""
    paragraphs = [p.strip() for p in text.split('\n\n') if p.lstrip().startswith('**Opener:')]
    if len(paragraphs) != 1:
        raise ValueError(f'the queue holds {len(paragraphs)} `**Opener:` paragraphs; a wake needs one')
    return paragraphs[0]


def ungated_opener(sha, opener):
    return (f'The tip `{sha}` is committed and neither gated nor pushed: the row that handed over was '
            f'past its {CEILING_K} k ceiling. Gate it first (`zsh tools/gate_run.sh {sha} <log> --scope '
            '<the last gated tip>`), and push and fast-forward main on its `gate exit 0`. '
            f'Then:\n\n{opener}')


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
    # A stale roster still lists the row `forget` just removed, and the
    # archive that follows then fails on a row that is gone (the reap of
    # e9aff36b, 2026-11-13: "forget kept the row; ccc archive: exit 1").
    rows = roster(fresh=True)
    row = next((r for r in rows if (r.get('session') or {}).get('id') == ref), None)
    if row is not None and (row.get('session') or {}).get('state') != 'stopped':
        print(f'handover: {ref} still has a process after stop and forget', file=sys.stderr)
        return 1
    if row is not None:
        out = run(['ccc', 'archive', ref])
        print(f'handover: forget kept the row; ccc archive {ref}: exit {out.returncode}')
    return 0


def hand_to(name, opener_file, ungated=False, opener=None):
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
    if ungated:
        ok, line = depth_line(own_ctx())
        if ok:
            raise SystemExit(f'handover: --ungated is for a row past its {CEILING_K} k ceiling; this one reads: {line}')
    elif head != pushed:
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
    if opener is None:
        opener = open(opener_file).read().strip()
    if ungated and head != pushed:
        opener = ungated_opener(head[:9], opener)
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
    to.add_argument('--ungated', action='store_true')
    sub.add_parser('reap').add_argument('ref')
    sub.add_parser('wake')
    check = sub.add_parser('check')
    check.add_argument('--scoped', action='store_true')
    check.add_argument('--tip', default='HEAD')
    args = parser.parse_args(argv)
    if args.verb == 'check':
        ok, line = check_line(roster(), tree())
        print(line)
        count, _ = landings_since_pass(tip=args.tip)
        landed, landings = landings_line(count, args.scoped, os.environ.get(WAIVER))
        shallow, depth = depth_line(own_ctx())
        print(f'Driver: {landings} · {depth}')
        return 1 if not ok else 3 if not landed else 4 if not shallow else 0
    if args.verb == 'reap':
        return reap(args.ref)
    if args.verb == 'wake':
        try:
            opener = queue_opener(open(os.path.join(tree(), 'docs/QUEUE.md')).read())
        except ValueError as e:
            raise SystemExit(f'handover: {e}')
        return hand_to('commander', None, opener=opener)
    return hand_to(args.name, args.opener_file, ungated=args.ungated)


if __name__ == '__main__':
    sys.exit(main())
