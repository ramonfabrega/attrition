#!/usr/bin/env python3
"""lanewait.py — the commander's wait on its lanes: one exit, on the first
thing a lane did that a commander acts on.

    python3 tools/lanewait.py <lane> [<lane> ...] [--quiet-after MIN] [--interval S]

Run it backgrounded after a spawn and end the turn; its exit re-invokes the
session. A lane is named as `ccc list` names it. The exits:

    0  LANDED <lane> <branch> <old>..<new>   origin's ref of the lane's branch moved
                                             and the row is out of `working`
    3  ENDED <lane> <state>                  the row left `working` and stayed out
    4  QUIET <lanes> <minutes>               nothing in --quiet-after minutes
    2  usage, or a lane the roster does not hold when the wait starts

A push while the row still works is printed as `PUSHED <lane> <branch>
<old>..<new>` and watched on (the twenty-fifth pass): the twenty-fifth
tranche's workers pushed mid-work — before a gate, after a re-pin, at an
update — and the waiter exited LANDED forty-one times for twenty landings,
each a commander turn that learned nothing. A lane that pushed and then
stopped working has landed, and the row says so within two polls; a lane
that pushed and works on is still working.

Why this and not an idle notice (the twenty-fourth pass, the sequential arm's
record): `notify_when_idle` fires when a turn ends, and a lane whose waits are
backgrounded ends a turn at every capture and every gate — three notices for
one idle in two minutes, none of them a landing. A lane that pushed has
landed; a row that left `working` with nothing pushed is the silent landing
of parked 1412 or a lane that needs a hand, and either is the commander's to
read. The local tip is not watched: a worker commits before it gates, and a
wake on that commit is a turn that learns nothing.
"""
import argparse
import json
import subprocess
import sys
import time

WORKING = 'working'
# Polls a row must stay out of `working` before it is called ended: a
# background task's exit re-invokes a session, and one poll can fall in
# the gap between the turn that ended and the turn that follows.
STILL = 2


def event(watch, states, tips, minutes, quiet_after, notes=None):
    """The first event among the lanes, as (exit code, line), or None.

    `watch` maps a lane to {'branch', 'tip', 'still'} and is updated in
    place: `still` counts consecutive polls out of `working`. `states`
    maps a lane to its roster state, None when the roster has no row;
    `tips` maps a lane to origin's ref of its branch, None when origin
    has none yet. A push outranks an ended row: a lane that pushed and
    stopped has landed. A push while the row works is noted in `notes`
    (when given) and the tip remembered, so the landing is the later
    stop, not the push.
    """
    for lane, seen in watch.items():
        tip = tips.get(lane)
        if tip and tip != seen['tip']:
            old = seen['tip'][:8] if seen['tip'] else 'none'
            moved = f"{lane} {seen['branch']} {old}..{tip[:8]}"
            if states.get(lane) == WORKING:
                seen['tip'] = tip
                seen['pushed'] = moved
                if notes is not None:
                    notes.append(f"PUSHED {moved}")
                continue
            return 0, f"LANDED {moved}"
    for lane, seen in watch.items():
        state = states.get(lane)
        if state == WORKING:
            seen['still'] = 0
            continue
        seen['still'] += 1
        if seen['still'] >= STILL:
            if seen.get('pushed'):
                return 0, f"LANDED {seen['pushed']}"
            return 3, f"ENDED {lane} {state or 'gone'}"
    if minutes >= quiet_after:
        return 4, f"QUIET {' '.join(watch)} {int(minutes)}"
    return None


def roster():
    """Each named session's (state, cwd, branch), by name; None when ccc did not answer."""
    out = subprocess.run(['ccc', 'list', '--json'], capture_output=True, text=True, check=False).stdout
    try:
        rows = json.loads(out)
    except ValueError:
        return None
    found = {}
    for row in rows:
        session = row.get('session') or {}
        name = session.get('name')
        # A name's newest row wins: `--replace` leaves the stopped one listed.
        if name and (name not in found or session.get('startedAt', '') > found[name][3]):
            found[name] = (session.get('state'), session.get('cwd'),
                           (row.get('worktree') or {}).get('branch'), session.get('startedAt', ''))
    return found


def origin_tip(cwd, branch):
    out = subprocess.run(['git', 'rev-parse', '--verify', '-q', f'refs/remotes/origin/{branch}'],
                         cwd=cwd, capture_output=True, text=True, check=False)
    return out.stdout.strip() or None


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    parser.add_argument('lanes', nargs='+')
    parser.add_argument('--quiet-after', type=float, default=100, metavar='MIN')
    parser.add_argument('--interval', type=float, default=30, metavar='S')
    args = parser.parse_args(argv)

    rows = roster()
    if rows is None:
        print('lanewait: ccc list did not answer', file=sys.stderr)
        return 2
    missing = [lane for lane in args.lanes if lane not in rows or not rows[lane][2]]
    if missing:
        print(f"lanewait: no roster row with a worktree branch for {', '.join(missing)}", file=sys.stderr)
        return 2
    watch, where = {}, {}
    for lane in args.lanes:
        _, cwd, branch, _ = rows[lane]
        where[lane] = cwd
        watch[lane] = {'branch': branch, 'tip': origin_tip(cwd, branch), 'still': 0}

    started = time.monotonic()
    while True:
        time.sleep(args.interval)
        rows = roster()
        if rows is None:
            continue
        states = {lane: (rows[lane][0] if lane in rows else None) for lane in watch}
        tips = {lane: origin_tip(where[lane], seen['branch']) for lane, seen in watch.items()}
        notes = []
        found = event(watch, states, tips, (time.monotonic() - started) / 60, args.quiet_after, notes)
        for note in notes:
            print(note, flush=True)
        if found:
            print(found[1], flush=True)
            return found[0]


if __name__ == '__main__':
    sys.exit(main())
