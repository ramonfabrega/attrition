#!/usr/bin/env python3
"""lanewait.py — the commander's wait on its lanes: one exit, on the first
thing a lane did that a commander acts on.

    python3 tools/lanewait.py <lane> [<lane> ...] [--quiet-after MIN] [--interval S]
                              [--arm-after-push <lane> ...] [--then TEMPLATE]
                              [--stalled-after MIN] [--held-arm-after MIN]

Run it backgrounded after a spawn and end the turn; its exit re-invokes the
session. A lane is named as `ccc list` names it. The exits:

    0  LANDED <lane> <branch> <old>..<new>   origin's ref of the lane's branch moved
                                             and the row is out of `working`, its
                                             session unchanged and no clear armed
                                             (a push under a clear is a DRAFT on
                                             stderr; item 3368, Loop 3458)
    3  ENDED <lane> <state> — <detail>       the row left `working` and stayed out;
                                             the row's own status line follows
    4  QUIET <lanes> <minutes>               nothing in --quiet-after minutes
    5  DEEP <lane> <ctx>k                    a lane's next turn would carry --deep k
                                             tokens or more (lore's `ctx`), once a lane
    6  STALLED <lane> <minutes>              a working row idle --stalled-after minutes
                                             with no child process but its servers, or
                                             behind a child older than
                                             --stalled-child-after (named): a gate's or
                                             a capture's exit notice was lost (parked
                                             1871, 178 minutes); one line to the lane
                                             says what to read. Never a lane whose held
                                             draft the waiter armed on the stall, while
                                             the session it armed under stands (Loop
                                             3795, 3796): that stall's answer is the arm
    7  UNCLEARED <lane> <minutes>            the lane's newest transcript ends on a
                                             `/clear` with no turn after it for
                                             --uncleared-after minutes: the armed
                                             clear's --then prompt never ran
                                             (parked 2127, twice in one tranche);
                                             one message carrying the prompt wakes it
    2  usage, a lane the roster does not hold when the wait starts, or a
       lane whose tree holds a second row (parked 1671: a lane's next session
       is `ccc clear <its row> --then`, never a spawn onto its tree), or an
       --arm-after-push lane the wait does not watch

And the lines that are not exits — the wait goes on past each:

       WAITING <lane> <branch> <tip>; … — <ISO> pid <pid> [arm-after-push <lane> …]
                                             the first line, flushed before the first
                                             poll (stdout; Loop 3716): an empty log is
                                             a waiter that never ran
       PUSHED <lane> <branch> <old>..<new>   a push while the row works (stdout)
       DRAFT <lane> <branch> <old>..<new> — …  a push under a clear, never a
                                             landing (stderr; item 3368, Loop 3458)
       ARMED <lane> — <ccc's first line>     an --arm-after-push lane pushed and the
                                             waiter armed its clear (stdout; Loop 3526)
       ARM-REFUSED <lane>: <ccc's first line>  ccc refused that arm, or is missing,
                                             failed or timed out (stdout); the lane
                                             stopping is then an ENDED to arm by hand
       DRAFT <lane> <branch> <old>..<new> — held draft on a stalled row; arming its clear
                                             an --arm-after-push lane's held push on a
                                             row that reads `working`, idle
                                             --held-arm-after minutes with nothing but
                                             its servers under it (stderr; Loop 3795):
                                             armed in place of a STALLED exit, its
                                             ARMED or ARM-REFUSED following on stdout,
                                             and the wait goes on
       PUSHED <lane> … — its done line, '…'; a landing, not armed
                                             an --arm-after-push lane's push sent with
                                             its done line (stdout; Loop 3749): the
                                             row stopping is its LANDED
       GATING <lane> <state> — <command> (<n> min); held, read again at its exit
                                             a row out of `working` with a gate running
                                             under its session (stderr; Loop 3753):
                                             never an ENDED while the gate lives
       HELD <lane> <old>..<new> — …          an --arm-after-push lane's push held for
                                             its done line when the wait exits on
                                             anything else (stdout, before the exit
                                             line; Loop 3765): the next
                                             --arm-after-push wait on it arms it

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

A push under a clear is a draft, not a landing (item 3368, the
forty-sixth pass): att-3365 crossed DEEP, pushed its draft journal as the
commander's answer asks, and ended its turn for the armed `ccc clear`;
the row left `working` for the seconds the clear took, and the waiter
exited `LANDED att-3365 … none..2caf3aee` with the row back at work under
a new session — a commander turn that learned nothing, and one that would
merge a draft read at face value; att-3374 the same morning. A clear
gives the row a new session uuid (`ccc list --json`'s
`session.sessionId`) and opens the new transcript on the `/clear`
record. So a push seen on the poll its session's uuid moved, or while
the lane's newest transcript ends on a `/clear`, and a push noted while
working whose session has since moved, is printed on stderr as `DRAFT
<lane> <branch> <old>..<new> — …`, its ref recorded, and the wait goes
on; LANDED fires only for a push whose session is the one the previous
poll saw. The uuid is compared poll to poll, never to the wait's start:
a lane cleared at DEEP with nothing pushed lands under its new session.

And the armed clear itself is asked before a LANDED (Loop 3458, the
fiftieth pass): att-3446 pushed its draft on a DEEP answer and the waiter
printed `LANDED att-3446`, its poll falling after the old turn ended and
before the clear fired, where neither tell above can see. So a landing is
first put to `ccc clear <lane> --status --json`; an armed clear makes it
`DRAFT <lane> … — pushed under an armed clear` on stderr, and the wait
goes on. A ccc that is missing, fails or answers unparseably is no clear,
and the landing stands as before.

And the DEEP answer's arm goes in after the lane's push (Loop 3526, the
fifty-second pass): `ccc clear <lane> --then …` refuses a tree with
uncommitted changes, and the commander's arm in the DEEP answer's own
turn was refused twice in one tranche (items 3523, 3524) with the lane
mid-edit — its line already said "armed", and the arm went in by hand
after the draft push. Without an arm none of the tells above can see the
draft, which reads PUSHED and then LANDED. So the commander re-arms the
wait with `--arm-after-push <lane>`: that lane's next push is the draft
on a committed tree; the ref is recorded (never a PUSHED or a LANDED),
`ccc clear <lane> --then "<--then with {lane} and {item}>"` runs once,
and `ARMED` or `ARM-REFUSED` says how it went. A clear already armed, or
one that fired on the push's poll, is not armed again. The flag lives in
the process: a re-arm after `ARMED` leaves the lane off it.

And the push the arm waits for can be the landing itself (Loop 3749,
the sixty-first pass): lane-3744 crossed DEEP while its lane gate ran,
and its next push, `none..10e53239`, was the finished landing the merge
took, sent with its done line `3744 landed and pushed` — the waiter
armed a clear on a landed lane, and the clear's new session announced
the landing a second time. Two of the twenty-five DEEP-answer pushes on
the commander's transcripts were landings (3591, 3744). What tells them
apart is the lane's own word: a worker pushes and then sends its
commander one line, and every done line and every draft note those
lanes sent reads one way or the other on its summary alone. So an
--arm-after-push lane's push is held while its row works, and the
lane's transcript is asked each poll for a `SendMessage` since the
push; a done line makes the push a landing (said as `PUSHED … — its
done line`, never armed, and LANDED when the row stops), and the row
stopping with none is the draft, armed as before.

And a lane gate is not an end (Loop 3753, the sixty-second pass):
lane-3747 committed its journal, started its lane gate backgrounded and
ended its turn; the row read `done` with the gate alive and nothing on
origin, and two arms printed `ENDED lane-3747 done` 69 s after the
commit and 60 s after WAITING, the wait on lane-3748 lost with it (3759
the same, 63 s after its journal commit). So a row that would be ENDED
with a `release_gate.py`, a `cargo test` or a `check.sh test` among its
session's descendants is GATING, said once on stderr, and read again
when the gate has gone or the branch pushed.

And a held push outlives its wait (Loop 3765, the sixty-second pass):
lane-3763's draft push was held for its done line while the row worked,
and the next poll's `DEEP lane-3731` exited with nothing said of it; the
row went idle with no clear armed, and the re-armed wait started on the
moved ref and could not see the push. So any exit says `HELD` first, and
every --arm-after-push wait records the ref the lane started on beside
--deep-state (`lanewait-arm.json`); the next --arm-after-push wait on a
lane whose ref has moved past that base, its session unchanged, seeds
the push as held and settles it at its start — armed when the row is
idle with no done line since the base was recorded, a landing with one,
held while the row works. A push no wait recorded a base for is never
armed at start.

And a held draft on a stalled row is armed (Loop 3795, the sixty-third
pass): lane-3788 answered its `DEEP 339k`, pushed its draft (73765697)
and ended its turn, but its row read `working` throughout — the lane's
own word, at `tempo=idle` — so `arm_pushes` held the push: lanewait-28
exited `HELD lane-3788 none..73765697` on the other lane's DEEP,
lanewait-29 seeded it held, held it again, and exited `STALLED
lane-3788 30`, and the commander armed the clear by hand; 3793's draft
the same tranche was armed by hand at its push rather than wait. The
held arm waited for the row to stop, and a row that reads `working`
never does. So an --arm-after-push lane whose push is held, on a row
idle --held-arm-after minutes (10; at 0, --stalled-after's) with no
child but its servers, is armed as if it had stopped — `DRAFT … — held
draft on a stalled row; arming its clear`, then `ARMED` or
`ARM-REFUSED` — and the wait goes on. A done line still makes it a
landing, a clear already armed is not armed again, and a gate or a
capture under the session, of any age, is a wait. And the stall is
not named while the session the arm went in under stands, so the
commander's nudge never reaches it (Loop 3796: a hand-armed clear
fired, and the nudge the STALLED asked for reached the fresh session,
which idled on it); a refused arm leaves the stall to the commander,
and it exits STALLED as before.

And the first line is the waiter's own (Loop 3716, the sixtieth pass):
fourteen of the sixtieth tranche's thirty-four logs and nine of the
sixty-first's sixteen were empty, a waiter replaced, a quiet one and a
dead one alike. `WAITING` names each lane, its branch and its pushed
tip, the time and the pid, flushed before the first poll.
"""
import os
import argparse
import json
import re
import subprocess
import sys
import time

WORKING = 'working'
# Polls a row must stay out of `working` before it is called ended: a
# background task's exit re-invokes a session, and one poll can fall in
# the gap between the turn that ended and the turn that follows.
STILL = 2


def ends_on_clear(tail):
    """Whether a `transcript_tail` result ends on a `/clear` with no
    record after it — the new session the clear opened, before its first
    turn."""
    if not tail:
        return False
    cleared_at, last_at = tail
    return cleared_at is not None and (last_at is None or last_at <= cleared_at)


def event(watch, states, tips, minutes, quiet_after, notes=None, details=None, sessions=None,
          cleared=None, drafts=None, armed=None, gating=None, gating_notes=None):
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

    `sessions` maps a lane to its row's session uuid now, and `seen`'s
    `session` holds the one the previous poll saw; `cleared` holds the
    lanes whose newest transcript ends on a `/clear`. A push on the poll
    the uuid moved, or from a `cleared` lane, and a pending push whose
    session has since moved, is a draft under a clear (item 3368): it
    goes in `drafts` (when given), its tip is remembered, and it never
    lands. An unknown uuid, either side, is read as unchanged.

    `armed`, when given, is asked of a lane just before it would land
    (`armed_clear`, Loop 3458): True makes the landing a draft under an
    armed clear, recorded and watched past.

    `gating` maps a lane to `(minutes, command)` of a gate running under
    its session (`gate_under`, Loop 3753): a row that would be ENDED with
    one is held instead, `GATING <lane> <state> — …` going in
    `gating_notes` once per gate, and the row must stay out STILL polls
    past the gate's exit before it is ENDED — the exit's notice is what
    re-invokes the session.
    """
    sessions = sessions or {}
    gating = gating or {}
    cleared = cleared or set()
    moved_session = {}
    for lane, seen in watch.items():
        now, then = sessions.get(lane), seen.get('session')
        moved_session[lane] = (then[:8], now[:8]) if now and then and now != then else None
        if now:
            seen['session'] = now
        if moved_session[lane] and seen.get('pushed'):
            line = seen.pop('pushed')
            if drafts is not None:
                drafts.append(f"DRAFT {line} — pushed before a clear (session "
                              "{}..{}); watching on".format(*moved_session[lane]))
    for lane, seen in watch.items():
        tip = tips.get(lane)
        if tip and tip != seen['tip']:
            old = seen['tip'][:8] if seen['tip'] else 'none'
            moved = f"{lane} {seen['branch']} {old}..{tip[:8]}"
            if moved_session[lane] or lane in cleared:
                seen['tip'] = tip
                seen.pop('pushed', None)
                if drafts is not None:
                    why = ('session {}..{}'.format(*moved_session[lane]) if moved_session[lane]
                           else 'transcript ends on /clear')
                    drafts.append(f"DRAFT {moved} — pushed under a clear ({why}); watching on")
                continue
            if states.get(lane) == WORKING:
                seen['tip'] = tip
                seen['pushed'] = moved
                if notes is not None:
                    notes.append(f"PUSHED {moved}")
                continue
            if armed is not None and armed(lane):
                seen['tip'] = tip
                seen.pop('pushed', None)
                if drafts is not None:
                    drafts.append(f"DRAFT {moved} — pushed under an armed clear; watching on")
                continue
            return 0, f"LANDED {moved}"
    for lane, seen in watch.items():
        state = states.get(lane)
        if lane not in gating:
            seen.pop('gating', None)
        if state == WORKING:
            seen['still'] = 0
            continue
        seen['still'] += 1
        if seen['still'] >= STILL:
            if seen.get('pushed'):
                if armed is not None and armed(lane):
                    line = seen.pop('pushed')
                    seen['still'] = 0
                    if drafts is not None:
                        drafts.append(f"DRAFT {line} — pushed under an armed clear; watching on")
                    continue
                return 0, f"LANDED {seen['pushed']}"
            gate = gating.get(lane)
            if gate:
                if not seen.get('gating'):
                    seen['gating'] = True
                    age = f"{int(gate[0])} min" if gate[0] is not None else 'age unknown'
                    if gating_notes is not None:
                        gating_notes.append(f"GATING {lane} {state or 'gone'} — {gate[1]} ({age}); "
                                            "held, read again at its exit")
                seen['still'] = 0
                continue
            seen.pop('gating', None)
            detail = (details or {}).get(lane)
            tail = f" — {detail}" if detail else ''
            return 3, f"ENDED {lane} {state or 'gone'}{tail}"
    if minutes >= quiet_after:
        return 4, f"QUIET {' '.join(watch)} {int(minutes)}"
    return None


DEEP_STEP = 40_000


def deep(ctxs, k_tokens, reported, answered=None, step=DEEP_STEP):
    """The first watched lane whose next turn carries `k_tokens` or more
    and is not in `reported`, as (exit code, line), or None. `ctxs` maps a
    lane to lore's `ctx` (None when lore has no transcript for it).

    `answered` maps a lane to the context its last DEEP was answered at,
    and outlives the process (`--deep-state`; Loop 2186): `reported` lived
    in the process alone, so a DEEP answered while the lane's gate ran
    fired again at the next re-arm — 2107 at 303k, then 310k three minutes
    later — and the commander raised `--deep` for every lane to quiet one.
    A lane answered at k is quiet until it has grown `step` past k; the
    exit moves the answer."""
    if not k_tokens:
        return None
    answered = {} if answered is None else answered
    for lane, ctx in ctxs.items():
        if ctx is None or ctx < k_tokens or lane in reported:
            continue
        last = answered.get(lane)
        if last is not None and ctx < last + step:
            continue
        reported.add(lane)
        answered[lane] = ctx
        return 5, f"DEEP {lane} {ctx // 1000}k"
    return None


def deep_state_default():
    """Where the DEEP answers persist between arms: the job's scratch
    directory when the harness names one, else nowhere."""
    job = os.environ.get('CLAUDE_JOB_DIR')
    return os.path.join(job, 'tmp', 'lanewait-deep.json') if job else None


def load_answered(path):
    """`{lane: ctx}` from the state file, `{}` when there is none or it
    does not parse — a lost file re-fires a DEEP, never hides one."""
    if not path or not os.path.exists(path):
        return {}
    try:
        with open(path) as f:
            data = json.load(f)
        return {str(k): int(v) for k, v in data.items()}
    except (OSError, ValueError, AttributeError):
        return {}


def save_answered(path, answered):
    if not path:
        return
    os.makedirs(os.path.dirname(path) or '.', exist_ok=True)
    with open(path, 'w') as f:
        json.dump(answered, f)


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


def stalled(watch, states, idle_minutes, children, after, reported, child_after=0):
    """The first watched lane that is `working`, idle for `after` minutes
    or more, with no child process but its servers, not yet in `reported`
    — as (6, `STALLED <lane> <minutes>`), or None. Parked 1871: 1738's lane
    gate exited and its notice never reached the session, which sat
    `working` at `tempo=idle` for 178 minutes; the row never left
    `working`, so no other exit could fire. A lane that waits on a gate or
    a capture has that process as a child.

    A child is a command line, or `(minutes, command)` with its age (Loop
    2820, the thirty-eighth pass: 2793 ended its turn with a subagent's
    background `cargo test` as its only child and sat idle 1 h 50, the
    child four hours, and a child of any age read as a wait). With
    `child_after` above zero, a lane idle past `after` whose every
    non-server child has outlived `child_after` is named with the oldest
    child; a child of unknown age, or one younger, is a wait."""
    for lane in watch:
        if states.get(lane) != WORKING or lane in reported:
            continue
        idle = idle_minutes.get(lane)
        if idle is None or idle < after:
            continue
        waits = [(c if isinstance(c, tuple) else (None, c)) for c in children.get(lane, [])]
        waits = [(age, cmd) for age, cmd in waits if not is_server(cmd)]
        if waits:
            if not child_after or any(age is None or age < child_after for age, _ in waits):
                continue
            age, cmd = max(waits, key=lambda w: w[0])
            reported.add(lane)
            return 6, f"STALLED {lane} {int(idle)} (child {int(age)} min: {cmd})"
        reported.add(lane)
        return 6, f"STALLED {lane} {int(idle)}"
    return None


def etime_minutes(etime):
    """Minutes from `ps`'s `etime` (`[[dd-]hh:]mm:ss`), or None."""
    try:
        days, _, rest = etime.rpartition('-')
        parts = [int(x) for x in rest.split(':')]
        if len(parts) == 2:
            h, m, s = 0, *parts
        elif len(parts) == 3:
            h, m, s = parts
        else:
            return None
        return (int(days or 0) * 24 + h) * 60 + m + s / 60
    except ValueError:
        return None


def children_of(pids):
    """Each pid's children as `(minutes alive, command line)`, from one `ps`."""
    out = subprocess.run(['ps', '-axo', 'pid=,ppid=,etime=,command='], capture_output=True, text=True,
                         check=False).stdout
    found = {pid: [] for pid in pids}
    for line in out.splitlines():
        parts = line.split(None, 3)
        if len(parts) < 4:
            continue
        try:
            ppid = int(parts[1])
        except ValueError:
            continue
        if ppid in found:
            found[ppid].append((etime_minutes(parts[2]), parts[3]))
    return found


def process_table():
    """Every process as `{pid: (ppid, minutes alive, command line)}`, from one `ps`."""
    out = subprocess.run(['ps', '-axo', 'pid=,ppid=,etime=,command='], capture_output=True, text=True,
                         check=False).stdout
    table = {}
    for line in out.splitlines():
        parts = line.split(None, 3)
        if len(parts) < 4:
            continue
        try:
            table[int(parts[0])] = (int(parts[1]), etime_minutes(parts[2]), parts[3])
        except ValueError:
            continue
    return table


# A gate a lane runs in the background and ends its turn on (Loop 3753).
# The process itself, by its program and first argument: a shell whose
# command text only names one (a `grep`, an `ls`) is not a gate.
GATE = re.compile(r'^\S*[Pp]ython[\d.]*(\s+-\S+)*\s+\S*release_gate\.py\b'
                  r'|^\S*cargo(\s+\+\S+)?\s+test\b'
                  r'|^\S*(zsh|bash|sh)\s+\S*check\.sh\s+test\b')


def gate_under(pid, table):
    """The shallowest descendant of `pid` in a `process_table` whose
    command line is a gate — a `release_gate.py`, a `cargo test`, a
    `check.sh test` run as its own process — as `(minutes, command)`, or
    None. A background task runs as a shell under the session and the
    gate under the shell, so the shell's text is never matched; a
    descendant of the lane's session is the lane's own."""
    if pid is None:
        return None
    kids = {}
    for child, (ppid, _, _) in table.items():
        kids.setdefault(ppid, []).append(child)
    level, seen = sorted(kids.get(pid, [])), {pid}
    while level:
        for child in level:
            _, minutes, command = table[child]
            if GATE.search(command):
                return minutes, command
        seen.update(level)
        level = sorted(c for parent in level for c in kids.get(parent, []) if c not in seen)
    return None


def held_lines(watch):
    """A `HELD` line for each --arm-after-push lane whose push is held for
    its done line (Loop 3765): said before any exit, so the commander's
    turn knows a draft went unarmed."""
    return [f"HELD {lane} {seen['held'][0]}..{seen['tip'][:8] if seen['tip'] else 'none'} — held for its done "
            f"line, not armed; the next --arm-after-push {lane} wait arms it"
            for lane, seen in watch.items() if seen.get('held')]


def held_ripe(watch, pending, states, idle_minutes, children, after):
    """The --arm-after-push lanes whose push is held on a row that reads
    `working`, idle `after` minutes or more with no child process but its
    servers — `stalled`'s test, at the held arm's own threshold (Loop
    3795): lane-3788's row read `working` at `tempo=idle` with its draft
    pushed, so `arm_pushes` held the push until `STALLED` exited, and the
    commander armed the clear by hand. A child of any age is a wait here,
    as it is to `stalled` with --stalled-child-after off: a gate or a
    capture still running is the lane's, and its exit is the next turn."""
    if not after:
        return set()
    ripe = set()
    for lane in sorted(pending):
        seen = watch.get(lane)
        if not seen or not seen.get('held') or states.get(lane) != WORKING:
            continue
        idle = idle_minutes.get(lane)
        if idle is None or idle < after:
            continue
        if any(not is_server(c[1] if isinstance(c, tuple) else c) for c in children.get(lane, [])):
            continue
        ripe.add(lane)
    return ripe


def load_bases(path):
    """`{lane: {'base', 'at', 'session'}}` from the arm state file, `{}`
    when there is none or it does not parse."""
    if not path or not os.path.exists(path):
        return {}
    try:
        with open(path) as f:
            data = json.load(f)
        return {str(k): v for k, v in data.items() if isinstance(v, dict)}
    except (OSError, ValueError, AttributeError):
        return {}


def save_bases(path, bases):
    if not path:
        return
    os.makedirs(os.path.dirname(path) or '.', exist_ok=True)
    with open(path, 'w') as f:
        json.dump(bases, f)


def seed_held(watch, pending, bases, now):
    """The base each --arm-after-push lane's wait recorded (Loop 3765),
    read at a wait's start and updated in place. A pending lane whose ref
    has moved past the base an earlier wait recorded, its session
    unchanged, has pushed since that wait began: the push is seeded as
    held from the base, for `arm_pushes` to arm or to call a landing.
    A pending lane with no base, or a session that moved (its clear
    fired), records the ref it starts on; a watched lane without the flag
    forgets its base. Returns the seeded lanes."""
    seeded = []
    for lane, seen in watch.items():
        if lane not in pending:
            bases.pop(lane, None)
            continue
        rec = bases.get(lane)
        if rec and rec.get('session') and seen.get('session') and rec['session'] != seen['session']:
            rec = None
        if rec is None:
            bases[lane] = {'base': seen['tip'], 'at': now, 'session': seen.get('session')}
        elif rec.get('base') != seen['tip']:
            seen['held'] = ((rec.get('base') or '')[:8] or 'none', rec.get('at'))
            seeded.append(lane)
    return seeded


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


def rows_for(lanes, wait, interval, roster_fn=None, sleep_fn=time.sleep):
    """The roster once every lane in `lanes` has a worktree branch, or after
    `wait` seconds: `(rows, missing)`, `rows` None when ccc did not answer.
    Loop 2620 (the thirty-sixth pass): armed in the spawn's own turn, the
    waiter exited 2 at once — the new row had not yet shown its branch — and
    a wait that dies at arm leaves the commander waking on nothing, so a
    missing branch is asked again every `interval` seconds for `wait`."""
    roster_fn = roster_fn or roster
    waited = 0.0
    while True:
        rows = roster_fn()
        if rows is None:
            return None, list(lanes)
        missing = [lane for lane in lanes if lane not in rows or not rows[lane][2]]
        if not missing or waited >= wait:
            return rows, missing
        step = min(interval, wait - waited)
        sleep_fn(step)
        waited += step


def armed_clear(lane, run=subprocess.run):
    """Whether `ccc clear <lane> --status --json` says a clear is armed on
    the lane and not yet fired (ccc 0.1.40: `{ref, armed, …}`). A missing
    ccc, a non-zero exit or an answer that does not parse is no clear, so
    the waiter lands as it did before the question was asked (Loop 3458)."""
    try:
        out = run(['ccc', 'clear', lane, '--status', '--json'], capture_output=True, text=True,
                  check=False, timeout=30)
    except (OSError, subprocess.SubprocessError):
        return False
    if out.returncode != 0:
        return False
    try:
        answer = json.loads(out.stdout)
    except ValueError:
        return False
    return isinstance(answer, dict) and answer.get('armed') is True


THEN = 'continue item {item} from the journal'


def item_of(lane):
    """The item a lane works: the first run of digits in its name
    (`att-3523` and `att-3523-2` are 3523), else the name itself."""
    import re
    found = re.search(r'\d+', lane)
    return found.group(0) if found else lane


def then_prompt(template, lane):
    """The `--then` prompt for a lane: `{lane}` and `{item}` substituted."""
    return template.replace('{lane}', lane).replace('{item}', item_of(lane))


def arm_clear(lane, prompt, run=subprocess.run):
    """`ccc clear <lane> --then <prompt>`, as (armed, ccc's first line). A
    missing ccc, a non-zero exit and a timeout are a refusal; never raises
    (Loop 3526)."""
    try:
        out = run(['ccc', 'clear', lane, '--then', prompt], capture_output=True, text=True,
                  check=False, timeout=60)
    except (OSError, subprocess.SubprocessError) as e:
        return False, f"{type(e).__name__}: {e}".splitlines()[0]
    text = '\n'.join(part for part in (out.stdout or '', out.stderr or '') if part.strip())
    first = next((line.strip() for line in text.splitlines() if line.strip()), '')
    if out.returncode != 0:
        return False, first or f"exit {out.returncode}"
    return True, first or 'armed'


# A worker's done line names the landing; a DEEP draft's note names the
# draft. Measured on the forty-three lines the twenty-five DEEP lanes of
# the fifty-third to sixty-first tranches sent their commander (Loop 3749).
DONE = re.compile(r'\b(landed|landing|done line|done report|ready to merge)\b'
                  r'|^\s*(att-)?(item )?\d+:? done\b|^\s*done\b', re.I)
NOT_DONE = re.compile(r'\b(drafts?|drafted|handover|handoff|handed over|deep exit|unlanded|not landed|'
                      r'not cleanly landed|no landing|not a landing|yet|before landing|status|asks?|ack|partial|'
                      r'nearly|run\d+\w* landed|gate running|first commit|landing the)\b', re.I)
# A line to the lane's own subagent or to another lane is never its done line.
NOT_TO = re.compile(r'^(a[0-9a-f]{16}|(att|lane)-\S+)$')


def is_done_line(summary, message=''):
    """Whether a `SendMessage` reads as a landing's done line and not a
    draft's note or a status: its summary when it has one, else its
    message's first line. Over the lines this repo's lanes had sent by
    2026-10-28, every line it calls done is a landing's; a landing it
    misses is armed as before."""
    text = (summary or '').strip() or ((message or '').strip().splitlines() or [''])[0][:200]
    return bool(DONE.search(text)) and not NOT_DONE.search(text)


def done_line(lines, since):
    """The summary of the first done line a transcript's JSONL lines
    carry at or after `since` (epoch seconds), or None."""
    from datetime import datetime
    for line in lines:
        if 'SendMessage' not in line:
            continue
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get('type') != 'assistant':
            continue
        try:
            at = datetime.fromisoformat((d.get('timestamp') or '').replace('Z', '+00:00')).timestamp()
        except ValueError:
            continue
        if at < since:
            continue
        for part in (d.get('message') or {}).get('content') or []:
            if not isinstance(part, dict) or part.get('type') != 'tool_use' or part.get('name') != 'SendMessage':
                continue
            given = part.get('input') or {}
            if NOT_TO.match(str(given.get('to') or '')):
                continue
            summary, message = given.get('summary') or '', given.get('message') or ''
            if is_done_line(summary, message):
                return summary.strip() or message.strip().splitlines()[0][:200]
    return None


def done_line_of(lane, since, where, sessions):
    """`done_line` over the lane's newest transcript; None when there is
    none or it cannot be read."""
    path = newest_transcript(where.get(lane), sessions.get(lane))
    if not path:
        return None
    try:
        with open(path, errors='replace') as f:
            return done_line(f, since)
    except OSError:
        return None


def arm_pushes(watch, tips, sessions, pending, template, arm=arm_clear, already_armed=armed_clear,
               states=None, landed=None, since=None, stalled=()):
    """Arm each pending lane's clear on its first push (Loop 3526), as
    `(drafts, lines)`: the DRAFT lines for stderr and the ARMED /
    ARM-REFUSED lines for stdout. Called before `event` on every poll.

    A lane in `pending` (the `--arm-after-push` lanes not yet armed) whose
    origin ref moved has pushed its DEEP answer's draft: the ref is
    recorded so `event` never reads it as a PUSHED or a LANDED, and
    `arm(lane, prompt)` runs once — the lane leaves `pending` whatever
    the answer. A clear already armed is not armed again, and a push on
    the poll the lane's session moved is the clear's own (it has fired):
    both leave the lane, and the second leaves the push to `event`, which
    reads it as the draft it is.

    With `states` and `landed` (Loop 3749), the push may be the landing:
    `landed(lane, since)` is the lane's done line since the push (`since`,
    epoch seconds, is the previous poll) or None. A done line makes the
    push a landing — a PUSHED line, `pending` left, and the push left to
    `event` as a push noted while working, so the row stopping lands it.
    With none, a push while the row works is held — its ref recorded, its
    range and poll kept in `seen['held']`, nothing said — and the row
    stopping with still no done line is the draft, armed as before.

    `stalled` names the lanes whose held push sits on a row that reads
    `working` but is idle with nothing running (`held_ripe`, Loop 3795):
    the push is armed as if the row had stopped, its DRAFT saying why.
    """
    drafts, lines = [], []
    for lane in sorted(pending):
        seen = watch.get(lane)
        tip = tips.get(lane)
        held = seen.get('held') if seen is not None else None
        if seen is None or (not held and (not tip or tip == seen['tip'])):
            continue
        now, then = sessions.get(lane), seen.get('session')
        if now and then and now != then:
            pending.discard(lane)
            if held:
                seen.pop('held')
                drafts.append(f"DRAFT {lane} {seen['branch']} {held[0]}..{seen['tip'][:8]} — held for its "
                              f"done line, and its clear fired (session {then[:8]}..{now[:8]}); watching on")
            lines.append(f"ARMED {lane} — its clear has fired (session {then[:8]}..{now[:8]}); "
                         "not armed again")
            continue
        old = held[0] if held else (seen['tip'][:8] if seen['tip'] else 'none')
        pushed_at = held[1] if held else since
        if tip and tip != seen['tip']:
            seen['tip'] = tip
        seen.pop('pushed', None)
        moved = f"{lane} {seen['branch']} {old}..{seen['tip'][:8]}"
        if landed is not None:
            said = landed(lane, pushed_at if pushed_at is not None else 0)
            if said:
                pending.discard(lane)
                seen.pop('held', None)
                seen['pushed'] = moved
                lines.append(f"PUSHED {moved} — its done line, {said!r}; a landing, not armed")
                continue
        if states is not None and states.get(lane) == WORKING and not (held and lane in stalled):
            seen['held'] = (old, pushed_at)
            continue
        pending.discard(lane)
        seen.pop('held', None)
        why = 'held draft on a stalled row' if held and lane in stalled else "the DEEP answer's push"
        drafts.append(f"DRAFT {moved} — {why}; arming its clear")
        if already_armed(lane):
            lines.append(f"ARMED {lane} — a clear was already armed; not armed again")
            continue
        ok, said = arm(lane, then_prompt(template, lane))
        lines.append(f"ARMED {lane} — {said}" if ok else f"ARM-REFUSED {lane}: {said}")
    return drafts, lines


def origin_tip(cwd, branch):
    """The branch's pushed tip as the lane's tree sees it, or None — also
    when the tree is gone: a waiter started on a lane the commander had
    already reaped died on `FileNotFoundError` from `subprocess` (the
    fifty-ninth tranche's `lanewait-30.log`, lane 3690), and a dead
    waiter names nothing."""
    if not cwd or not os.path.isdir(cwd):
        return None
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
    parser.add_argument('--deep-state', default=deep_state_default(), metavar='FILE',
                        help='where a DEEP answer persists between arms, so an answered lane is quiet '
                             'until it grows --deep-step (Loop 2186); default $CLAUDE_JOB_DIR/tmp/lanewait-deep.json')
    parser.add_argument('--deep-step', type=int, default=DEEP_STEP // 1000, metavar='K',
                        help='k tokens a DEEP lane must grow past its last answer to be named again')
    parser.add_argument('--deep-every', type=int, default=10, metavar='POLLS',
                        help='read lore agents every this many polls')
    parser.add_argument('--stalled-after', type=float, default=30, metavar='MIN',
                        help='exit STALLED once per working lane idle this long with nothing running; 0 off')
    parser.add_argument('--held-arm-after', type=float, default=10, metavar='MIN',
                        help='arm an --arm-after-push lane\'s held draft once its working row has idled this '
                             'long with nothing running, instead of a STALLED exit (Loop 3795); 0 takes '
                             '--stalled-after\'s threshold')
    parser.add_argument('--stalled-child-after', type=float, default=90, metavar='MIN',
                        help='a stalled lane whose every child has run this long is named with the child '
                             '(Loop 2820: a subagent\'s four-hour background test read as a wait); 0 off')
    parser.add_argument('--uncleared-after', type=float, default=5, metavar='MIN',
                        help="exit UNCLEARED once per lane whose newest transcript ends on a /clear "
                             "this long with no turn after it (parked 2127); 0 off")
    parser.add_argument('--roster-wait', type=float, default=60, metavar='S',
                        help='seconds to wait for a named lane to show its worktree branch before '
                             'refusing (Loop 2620: a wait armed in the spawn\'s own turn died at arm)')
    parser.add_argument('--arm-after-push', action='append', default=[], metavar='LANE',
                        help="a watched lane answered at DEEP: its next push is the draft, and the waiter "
                             "arms `ccc clear LANE --then` itself on it, once (Loop 3526); repeatable")
    parser.add_argument('--then', default=THEN, metavar='TEMPLATE',
                        help="the --arm-after-push clear's prompt; {lane} and {item} (the lane's number) "
                             "are substituted; default %(default)r")
    parser.add_argument('--arm-state', default=None, metavar='FILE',
                        help="where each --arm-after-push lane's base persists between waits (Loop 3765); "
                             "default lanewait-arm.json beside --deep-state, nowhere without one")
    args = parser.parse_args(argv)
    arm_state = args.arm_state if args.arm_state is not None else (
        os.path.join(os.path.dirname(args.deep_state), 'lanewait-arm.json') if args.deep_state else None)
    strays = [lane for lane in args.arm_after_push if lane not in args.lanes]
    if strays:
        print(f"lanewait: --arm-after-push names a lane not watched: {', '.join(strays)}", file=sys.stderr)
        return 2

    rows, missing = rows_for(args.lanes, args.roster_wait, 5)
    if rows is None:
        print('lanewait: ccc list did not answer', file=sys.stderr)
        return 2
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
        watch[lane] = {'branch': branch, 'tip': origin_tip(cwd, branch), 'still': 0,
                       'session': getattr(roster, 'sessions', {}).get(lane)}

    from datetime import datetime, timezone
    begun = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
    named = '; '.join(f"{lane} {seen['branch']} {seen['tip'][:8] if seen['tip'] else 'none'}"
                      for lane, seen in watch.items())
    arming = f" arm-after-push {' '.join(args.arm_after_push)}" if args.arm_after_push else ''
    arm_pending = set(args.arm_after_push)
    bases = load_bases(arm_state)
    seeded = seed_held(watch, arm_pending, bases, time.time())
    save_bases(arm_state, bases)
    print(f"WAITING {named} — {begun} pid {os.getpid()}{arming}", flush=True)

    def settle(states, tips, sessions, since, stalled=()):
        arm_drafts, arm_lines = arm_pushes(
            watch, tips, sessions, arm_pending, args.then, arm=arm_clear, already_armed=armed_clear,
            states=states, since=since, landed=lambda lane, at: done_line_of(lane, at, where, sessions),
            stalled=stalled)
        for draft in arm_drafts:
            print(draft, file=sys.stderr, flush=True)
        for line in arm_lines:
            print(line, flush=True)
        gone = [lane for lane in bases if lane in watch and lane not in arm_pending]
        for lane in gone:
            bases.pop(lane)
        if gone:
            save_bases(arm_state, bases)
        return {lane for lane in stalled for line in arm_lines if line.startswith(f"ARMED {lane} ")}

    def finish(found):
        for line in held_lines(watch):
            print(line, flush=True)
        print(found[1], flush=True)
        return found[0]

    if seeded:
        settle({lane: rows[lane][0] for lane in watch}, {lane: seen['tip'] for lane, seen in watch.items()},
               {lane: seen.get('session') for lane, seen in watch.items()}, None)

    started = time.monotonic()
    polled_at = time.time()
    polls = 0
    reported = set()
    stalled_reported = set()
    uncleared_reported = set()
    held_after = args.held_arm_after or args.stalled_after
    # Lanes whose held draft the waiter armed on a stall, by the session it
    # armed under: never STALLED while that session stands (Loop 3796).
    held_armed = {}
    while True:
        time.sleep(args.interval)
        polls += 1
        if args.deep and polls % max(args.deep_every, 1) == 0:
            answered = load_answered(args.deep_state)
            found = deep(contexts(list(watch)), args.deep * 1000, reported, answered, args.deep_step * 1000)
            if found:
                save_answered(args.deep_state, answered)
                return finish(found)
        rows = roster()
        if rows is None:
            continue
        if args.uncleared_after and polls % max(args.deep_every, 1) == 0:
            found = uncleared(tails_of(list(watch), where, getattr(roster, 'sessions', {})),
                              time.time(), args.uncleared_after, uncleared_reported)
            if found:
                return finish(found)
        states = {lane: (rows[lane][0] if lane in rows else None) for lane in watch}
        details = {lane: (rows[lane][4] if lane in rows else None) for lane in watch}
        since, polled_at = polled_at, time.time()
        tips = {lane: origin_tip(where[lane], seen['branch']) for lane, seen in watch.items()}
        sessions = {lane: getattr(roster, 'sessions', {}).get(lane) for lane in watch}
        if arm_pending:
            settle(states, tips, sessions, since)
        holding = held_after and any(watch[lane].get('held') for lane in arm_pending)
        if args.stalled_after or holding:
            from datetime import datetime, timezone
            now = datetime.now(timezone.utc)
            rests = getattr(roster, 'rests', {})
            idle = {lane: (idle_since(rests[lane][2], now) if lane in rests and rests[lane][1] == 'idle'
                           and rests[lane][2] else None) for lane in watch}
            pids = {lane: rests[lane][0] for lane in watch if lane in rests and rests[lane][0]}
            kids = children_of(list(pids.values()))
            children = {lane: kids.get(pid, []) for lane, pid in pids.items()}
            ripe = held_ripe(watch, arm_pending, states, idle, children, held_after) if holding else set()
            if ripe:
                for lane in settle(states, tips, sessions, since, stalled=ripe):
                    held_armed[lane] = sessions.get(lane)
            if args.stalled_after:
                live = [lane for lane in watch
                        if lane not in held_armed or (sessions.get(lane) and held_armed[lane]
                                                      and sessions[lane] != held_armed[lane])]
                found = stalled(live, states, idle, children, args.stalled_after, stalled_reported,
                                args.stalled_child_after)
                if found:
                    return finish(found)
        rests = getattr(roster, 'rests', {})
        out = [lane for lane in watch if states.get(lane) != WORKING and (rests.get(lane) or (None,))[0]]
        table = process_table() if out else {}
        gating = {lane: gate for lane in out if (gate := gate_under(rests[lane][0], table))}
        # The transcript is read only for a lane whose ref moved: the tell
        # is asked of a push, never of every poll.
        pushed = [lane for lane, seen in watch.items() if tips.get(lane) and tips[lane] != seen['tip']]
        tails = tails_of(pushed, where, sessions) if pushed else {}
        cleared = {lane for lane, tail in tails.items() if ends_on_clear(tail)}
        notes, drafts, gate_notes = [], [], []
        found = event(watch, states, tips, (time.monotonic() - started) / 60, args.quiet_after, notes,
                      details, sessions, cleared, drafts, armed_clear, gating, gate_notes)
        for note in notes:
            print(note, flush=True)
        for draft in drafts + gate_notes:
            print(draft, file=sys.stderr, flush=True)
        if found:
            return finish(found)


if __name__ == '__main__':
    sys.exit(main())
