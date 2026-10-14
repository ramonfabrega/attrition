#!/usr/bin/env python3
"""lanewait.py — the commander's wait on its lanes: one exit, on the first
thing a lane did that a commander acts on.

    python3 tools/lanewait.py <lane> [<lane> ...] [--quiet-after MIN] [--interval S]

Run it backgrounded after a spawn and end the turn; its exit re-invokes the
session. A lane is named as `ccc list` names it. The exits:

    0  LANDED <lane> <branch> <old>..<new>   origin's ref of the lane's branch moved
                                             and the row is out of `working`
    3  ENDED <lane> <state> — <detail>       the row left `working` and stayed out;
                                             the row's own status line follows
    4  QUIET <lanes> <minutes>               nothing in --quiet-after minutes
    5  DEEP <lane> <ctx>k                    a lane's next turn would carry --deep k
    6  STALLED <lane> <minutes>              a working row idle --stalled-after minutes
                                             with no child process but its servers:
                                             a gate's or a capture's exit notice was
                                             lost (parked 1871, 178 minutes); one line
                                             to the lane says what to read
                                             tokens or more (lore's `ctx`), once a lane
    7  UNCLEARED <lane> <minutes>            the lane's newest transcript ends on a
                                             `/clear` with no turn after it for
                                             --uncleared-after minutes: the armed
                                             clear's --then prompt never ran
                                             (parked 2127, twice in one tranche);
                                             one message carrying the prompt wakes it
    2  usage, a lane the roster does not hold when the wait starts, or a
       lane whose tree holds a second row (parked 1671: a lane's next session
       is `ccc clear <its row> --then`, never a spawn onto its tree)

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

The DEEP exit (the twenty-ninth pass, parked 1637 and 1638): the frame
asked a lane to arm its own clear at 300 k, and five sessions over 300 k
in two tranches took none — a lane cannot read its own context, and the
classifier refuses a session's arm of its own clear (DECISIONS 61). So
the waiter reads `lore agents --json`'s `ctx` (the context the lane's
next turn carries) every --deep-every polls and exits once per lane that
crosses --deep; the commander answers with one line to the lane (write
the journal as a draft, commit, end the turn) and arms `ccc clear <lane>
--then "continue item <N> from the journal"`, which fires at the lane's
next idle on a committed tree. `--deep 0` turns it off.

The UNCLEARED exit (the thirty-second pass, parked 2127): twice in one
tranche a cross-session clear fired, the lane's new session opened on the
`/clear` record, and the `--then` prompt never followed — the row read
idle with model `-` and sat twenty minutes until a message carrying the
prompt woke it. The lane's newest transcript says so: its last record is
the `/clear` command and nothing comes after. The waiter reads the
transcript's tail every --deep-every polls, beside `ctx`, and exits once
per lane whose `/clear` has stood --uncleared-after minutes with no turn
past it; the commander answers with the prompt as one message.
`--uncleared-after 0` turns it off.
"""
import os
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


def event(watch, states, tips, minutes, quiet_after, notes=None, details=None):
    """The first event among the lanes, as (exit code, line), or None.

    `watch` maps a lane to {'branch', 'tip', 'still'} and is updated in
    place: `still` counts consecutive polls out of `working`. `states`
    maps a lane to its roster state, None when the roster has no row;
    `tips` maps a lane to origin's ref of its branch, None when origin
    has none yet. A push outranks an ended row: a lane that pushed and
    stopped has landed. A push while the row works is noted in `notes`
    (when given) and the tip remembered, so the landing is the later
    stop, not the push. `details` maps a lane to the row's own status
    line; an ended row's goes on the exit line, because five of the race
    on 1648's twenty landings ended `blocked` asking for the commander's
    queue lines (parked 1671), and the one message that unblocks such a
    row is written in the turn that reads the line.
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
            detail = (details or {}).get(lane)
            tail = f" — {detail}" if detail else ''
            return 3, f"ENDED {lane} {state or 'gone'}{tail}"
    if minutes >= quiet_after:
        return 4, f"QUIET {' '.join(watch)} {int(minutes)}"
    return None


def deep(ctxs, k_tokens, reported):
    """The first watched lane whose next turn carries `k_tokens` or more
    and is not in `reported`, as (exit code, line), or None. `ctxs` maps a
    lane to lore's `ctx` (None when lore has no transcript for it)."""
    if not k_tokens:
        return None
    for lane, ctx in ctxs.items():
        if ctx is not None and ctx >= k_tokens and lane not in reported:
            reported.add(lane)
            return 5, f"DEEP {lane} {ctx // 1000}k"
    return None


SERVERS = ('bun ', 'node ', 'npx ')


def uncleared(tails, now, after_minutes, reported):
    """The first lane whose newest transcript ends on a `/clear` that has
    stood `after_minutes` with no record after it, once each — as (7,
    `UNCLEARED <lane> <minutes>`), or None. `tails` maps a lane to
    `(cleared_at, last_at)` from `transcript_tail`: the time of the last
    `/clear` command record, and of the last record of any kind; a lane
    with no `/clear`, or a record after it, is clear. `now` and the stamps
    are epoch seconds."""
    if not after_minutes:
        return None
    for lane in sorted(tails):
        tail = tails.get(lane)
        if not tail or lane in reported:
            continue
        cleared_at, last_at = tail
        if cleared_at is None or (last_at is not None and last_at > cleared_at):
            continue
        minutes = (now - cleared_at) / 60
        if minutes >= after_minutes:
            reported.add(lane)
            return 7, f"UNCLEARED {lane} {int(minutes)}"
    return None


def transcript_tail(lines):
    """`(cleared_at, last_at)` over a transcript's JSONL lines, epoch
    seconds or None: when the last `/clear` command record was written,
    and when the last record carrying a message of any kind was. The
    `/clear` record is a user record whose content opens on the
    `<command-name>/clear</command-name>` tag the harness writes for a
    local command; the records before the first message (the title, the
    bridge, a snapshot) carry no timestamp and are not read."""
    from datetime import datetime
    cleared_at = last_at = None
    for line in lines:
        try:
            d = json.loads(line)
        except ValueError:
            continue
        stamp = d.get('timestamp')
        if not stamp or d.get('type') not in ('user', 'assistant'):
            continue
        try:
            at = datetime.fromisoformat(stamp.replace('Z', '+00:00')).timestamp()
        except ValueError:
            continue
        content = (d.get('message') or {}).get('content')
        text = content if isinstance(content, str) else json.dumps(content or '')
        if d.get('type') == 'user' and '<command-name>/clear</command-name>' in text:
            cleared_at = at
        else:
            last_at = at
    return cleared_at, last_at


def newest_transcript(cwd, session_id=None):
    """The lane's newest transcript: the harness writes
    `~/.claude/projects/<cwd with / as ->/<session id>.jsonl`; the
    session's own file when ccc names the id, else the newest by mtime."""
    if not cwd:
        return None
    well = os.path.join(os.path.expanduser('~/.claude/projects'), cwd.replace('/', '-'))
    if session_id:
        own = os.path.join(well, f'{session_id}.jsonl')
        if os.path.isfile(own):
            return own
    try:
        files = [os.path.join(well, f) for f in os.listdir(well) if f.endswith('.jsonl')]
    except OSError:
        return None
    return max(files, key=os.path.getmtime) if files else None


def tails_of(lanes, where, sessions):
    out = {}
    for lane in lanes:
        path = newest_transcript(where.get(lane), sessions.get(lane))
        if not path:
            out[lane] = None
            continue
        try:
            with open(path, errors='replace') as f:
                out[lane] = transcript_tail(f)
        except OSError:
            out[lane] = None
    return out


def is_server(command):
    """A child every session carries from its start — an MCP server —
    and not a wait: the lazy-tools and hail servers here are `bun …`."""
    return command.startswith(SERVERS)


def stalled(watch, states, idle_minutes, children, after, reported):
    """The first watched lane that is `working`, idle for `after` minutes
    or more, with no child process but its servers, not yet in `reported`
    — as (6, `STALLED <lane> <minutes>`), or None. Parked 1871: 1738's lane
    gate exited and its notice never reached the session, which sat
    `working` at `tempo=idle` for 178 minutes; the row never left
    `working`, so no other exit could fire. A lane that waits on a gate or
    a capture has that process as a child."""
    for lane in watch:
        if states.get(lane) != WORKING or lane in reported:
            continue
        idle = idle_minutes.get(lane)
        if idle is None or idle < after:
            continue
        if any(not is_server(c) for c in children.get(lane, [])):
            continue
        reported.add(lane)
        return 6, f"STALLED {lane} {int(idle)}"
    return None


def children_of(pids):
    """Each pid's child command lines, from one `ps`."""
    out = subprocess.run(['ps', '-axo', 'pid=,ppid=,command='], capture_output=True, text=True,
                         check=False).stdout
    found = {pid: [] for pid in pids}
    for line in out.splitlines():
        parts = line.split(None, 2)
        if len(parts) < 3:
            continue
        try:
            ppid = int(parts[1])
        except ValueError:
            continue
        if ppid in found:
            found[ppid].append(parts[2])
    return found


def idle_since(stamp, now):
    """Minutes from a row's `updatedAt` (ccc's ISO instant, `Z`) to `now`,
    or None when it does not parse."""
    from datetime import datetime, timezone
    try:
        then = datetime.fromisoformat(stamp.replace('Z', '+00:00'))
    except (ValueError, AttributeError):
        return None
    if then.tzinfo is None:
        then = then.replace(tzinfo=timezone.utc)
    return (now - then).total_seconds() / 60


def contexts(lanes):
    """Each lane's `ctx` from `lore agents --json`; None for a lane lore
    does not list, and {} when lore did not answer."""
    out = subprocess.run(['lore', 'agents', '--json'], capture_output=True, text=True,
                         check=False).stdout
    try:
        rows = json.loads(out)
    except ValueError:
        return {}
    rows = rows if isinstance(rows, list) else rows.get('agents') or rows.get('rows') or []
    by_name = {r.get('name'): r.get('ctx') for r in rows if isinstance(r, dict)}
    return {lane: by_name.get(lane) for lane in lanes}


def state_of(session, job=None):
    """A row's state as the waiter reads it. A row whose turn is running is
    working whatever its state says: a lane that wrote `blocked — awaiting
    …` as its own status and never refreshed it read as ended for an hour
    while its pane showed it writing code (parked 1572), and three re-arms
    exited on it at once. The field that means "the turn is running" is
    the job's `tempo` (ccc's word, 2026-10-07: `session.status` is only
    "something is attached" — a Monitor or a background bash keeps a
    resting commander `busy` indefinitely, and ccc's own clear gate had
    to switch fields); `state` is the lane's own word, the fallback."""
    state = session.get('state')
    if state != WORKING and (job or {}).get('tempo') == WORKING:
        return WORKING
    return state

def second_rows(trees, lanes):
    """Each watched lane whose tree another row stands on, as (lane, other,
    tree). `trees` maps every roster name to its cwd, archived and stopped
    rows included — the race on 1648 put `att-1648-sm-2` beside a stopped
    `att-1648-sm` on one tree (parked 1671). The two permanent rows share
    the integration tree by design and are never a lane."""
    found = []
    for lane in lanes:
        tree = trees.get(lane)
        for other, cwd in sorted(trees.items()):
            if other != lane and tree and cwd == tree:
                found.append((lane, other, tree))
    return found


def roster():
    """Each named session's (state, cwd, branch, startedAt, detail), by name,
    and every row's tree under `roster.trees`; None when ccc did not answer."""
    out = subprocess.run(['ccc', 'list', '--json'], capture_output=True, text=True, check=False).stdout
    try:
        rows = json.loads(out)
    except ValueError:
        return None
    found, trees, rests, sessions = {}, {}, {}, {}
    for row in rows:
        session = row.get('session') or {}
        name = session.get('name')
        if not name:
            continue
        trees[name] = session.get('cwd')
        # A name's newest row wins: `--replace` leaves the stopped one listed.
        if name not in found or session.get('startedAt', '') > found[name][3]:
            found[name] = (state_of(session, row.get('job')), session.get('cwd'),
                           (row.get('worktree') or {}).get('branch'), session.get('startedAt', ''),
                           session.get('detail') or (row.get('job') or {}).get('detail'))
            job = row.get('job') or {}
            rests[name] = (session.get('pid'), job.get('tempo'), job.get('updatedAt'))
            sessions[name] = session.get('sessionId')
    roster.trees = trees
    roster.rests = rests
    roster.sessions = sessions
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
    parser.add_argument('--deep', type=int, default=300, metavar='K',
                        help='exit DEEP once per lane whose next turn carries K k tokens or more; 0 off')
    parser.add_argument('--deep-every', type=int, default=10, metavar='POLLS',
                        help='read lore agents every this many polls')
    parser.add_argument('--stalled-after', type=float, default=30, metavar='MIN',
                        help='exit STALLED once per working lane idle this long with nothing running; 0 off')
    parser.add_argument('--uncleared-after', type=float, default=5, metavar='MIN',
                        help="exit UNCLEARED once per lane whose newest transcript ends on a /clear "
                             "this long with no turn after it (parked 2127); 0 off")
    args = parser.parse_args(argv)

    rows = roster()
    if rows is None:
        print('lanewait: ccc list did not answer', file=sys.stderr)
        return 2
    missing = [lane for lane in args.lanes if lane not in rows or not rows[lane][2]]
    if missing:
        print(f"lanewait: no roster row with a worktree branch for {', '.join(missing)}", file=sys.stderr)
        return 2
    doubled = second_rows(roster.trees, args.lanes)
    if doubled:
        for lane, other, tree in doubled:
            print(f"lanewait: a second row on {lane}'s tree: {other} ({tree}); a lane's next "
                  f"session is `ccc clear {lane} --then`, never a spawn onto its tree (parked 1671)",
                  file=sys.stderr)
        return 2
    watch, where = {}, {}
    for lane in args.lanes:
        _, cwd, branch, _, _ = rows[lane]
        where[lane] = cwd
        watch[lane] = {'branch': branch, 'tip': origin_tip(cwd, branch), 'still': 0}

    started = time.monotonic()
    polls = 0
    reported = set()
    stalled_reported = set()
    uncleared_reported = set()
    while True:
        time.sleep(args.interval)
        polls += 1
        if args.deep and polls % max(args.deep_every, 1) == 0:
            found = deep(contexts(list(watch)), args.deep * 1000, reported)
            if found:
                print(found[1], flush=True)
                return found[0]
        rows = roster()
        if rows is None:
            continue
        if args.uncleared_after and polls % max(args.deep_every, 1) == 0:
            found = uncleared(tails_of(list(watch), where, getattr(roster, 'sessions', {})),
                              time.time(), args.uncleared_after, uncleared_reported)
            if found:
                print(found[1], flush=True)
                return found[0]
        states = {lane: (rows[lane][0] if lane in rows else None) for lane in watch}
        details = {lane: (rows[lane][4] if lane in rows else None) for lane in watch}
        if args.stalled_after:
            from datetime import datetime, timezone
            now = datetime.now(timezone.utc)
            rests = getattr(roster, 'rests', {})
            idle = {lane: (idle_since(rests[lane][2], now) if lane in rests and rests[lane][1] == 'idle'
                           and rests[lane][2] else None) for lane in watch}
            pids = {lane: rests[lane][0] for lane in watch if lane in rests and rests[lane][0]}
            kids = children_of(list(pids.values()))
            children = {lane: kids.get(pid, []) for lane, pid in pids.items()}
            found = stalled(list(watch), states, idle, children, args.stalled_after, stalled_reported)
            if found:
                print(found[1], flush=True)
                return found[0]
        tips = {lane: origin_tip(where[lane], seen['branch']) for lane, seen in watch.items()}
        notes = []
        found = event(watch, states, tips, (time.monotonic() - started) / 60, args.quiet_after, notes,
                      details)
        for note in notes:
            print(note, flush=True)
        if found:
            print(found[1], flush=True)
            return found[0]


if __name__ == '__main__':
    sys.exit(main())
