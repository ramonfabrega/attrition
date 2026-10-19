#!/usr/bin/env python3
"""Run the full local gate with an explicit install, including its data survey.

**Two gates, one script** (1567, `docs/DECISIONS.md` 63 amended). With no
flag this is the commander's booking gate: six steps, the whole release
suite, nothing forgiven. With `--lane` it is a worker's, and since the
twenty-seventh tranche it no longer runs the whole suite — every landing
ran it three times on one box (the worker's own run, its lane gate, the
booking gate), and a third word lane bought no wall clock for that reason.

The lane gate's release step runs:

  * every test of every crate that is **not timed over budget** — the
    timings file (`TIMINGS`, every rondata release test alone, one at a
    time) names each rondata test that took over `LANE_BUDGET_SECONDS`, and
    those are `--skip`ped by exact name; a test the file does not name (the
    worker's new widening among them) runs. So the sim suite, the paperwork
    guards and the data layer run whole, and rondata's walks under a second;
  * **the word's own tests**, `--tests FILTER ...`, which the brief names
    and the lane gate refuses to run without: a timed test that contains a
    filter is kept, however slow, and a filter that matched no test that
    ran turns the gate red — a typo is not a pass;
  * and **never** the two coverage pins or the rest of the slow walks
    (`coverage::every_key_the_dump_prints_is_read_or_pinned`, 118 s alone;
    `..._every_parsed_field_is_compared_by_the_instrument_or_pinned`, 80 s):
    those, and the whole suite, are the booking gate's. A worker whose
    landing reads a new key passes `--tests coverage::` and runs them.

The worker's own full run, measured on the tree after its last `ccc
update`, is still owed before the gate (the brief's "measure" rows); the
lane gate is the confirmation of the word and the paperwork, not the
search for a re-pin. Suites a landing: three to two.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import time
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from test_runtime import child_environment, allocator_policy
from resource_budget import CPU as HOST_CPU_SLOTS, MEMORY_MIB as HOST_MEMORY_MIB, OFF_MARKER

RUNNING = re.compile(r'^\s*Running (?:unittests )?\S+ \((\S+)\)$')
DOC_TESTS = re.compile(r'^\s*Doc-tests (\S+)$')
RESULT = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;')
FAILED_TEST = re.compile(r'^test (\S+) \.\.\. FAILED$')


def phase_budget(command):
    """Small already-built checks can coexist; possible compilations drain slots."""
    if any(str(a).endswith('offline_tests.py') for a in command) or 'fmt' in command:
        return 2, 1024
    if any(str(a).endswith('guard.sh') for a in command):
        return 2, 4096  # the reflex's reservation (3 GiB measured); its rebuild ceiling is phase_limit's
    return 8, 32768


def phase_limit(command):
    """The sampled RSS ceiling where it is not the reservation: the guard's
    debug rebuild after an edit (DECISIONS 75)."""
    if any(str(a).endswith('guard.sh') for a in command):
        return 16384
    return None


def run_logged(command, *, cwd, env, check, log=None):
    """`subprocess.run`, or — with `log` — the same command with its output
    copied line by line to that file as well as to stdout. No shell pipe
    stands between the command and its exit status: a `| tee` launders
    `memcap.sh`'s 137 into a 0, and the release command's exit code is the
    gate's verdict."""
    dispatcher = any(str(a).endswith('tools/isolated_tests.py') for a in command)
    if env.get('RON_GATE_RESOURCE_POLICY') == '1' and not dispatcher:
        env = env.copy()
        if any(str(a).endswith('guard.sh') for a in command):
            env['RON_GUARD_ADMITTED'] = '1'
        cpu, memory = phase_budget(command)
        limit = phase_limit(command)
        command = [sys.executable, str(ROOT / 'tools/resource_budget.py'),
                   '--cpu', str(cpu), '--memory-mib', str(memory)] + \
            (['--limit-mib', str(limit)] if limit is not None else []) + ['--'] + command
    if log is None:
        return subprocess.run(command, cwd=cwd, env=env, check=check)
    with open(log, 'wb') as sink, subprocess.Popen(
            command, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT) as child:
        for line in child.stdout:
            sys.stdout.buffer.write(line)
            sys.stdout.buffer.flush()
            sink.write(line)
    completed = subprocess.CompletedProcess(command, child.returncode)
    if check:
        completed.check_returncode()
    return completed


def summarize_tests(log, *, release_completed):
    """Every `test result:` line in a release log, keyed by the binary that
    printed it, plus the name of every test that failed. `--no-fail-fast`
    makes cargo run every binary; this is what makes the run readable
    afterwards — a red rondata by design on a word-moving item, and the
    sim suite behind it, are two lines instead of one truncated run."""
    binaries = []
    current = None
    failed_tests = []
    for raw in Path(log).read_bytes().splitlines():
        line = raw.decode('utf-8', 'replace')
        if match := RUNNING.match(line):
            name = Path(match.group(1)).name
            current = name.rsplit('-', 1)[0] if '-' in name else name
        elif match := DOC_TESTS.match(line):
            current = f'doc-tests {match.group(1)}'
        elif match := FAILED_TEST.match(line):
            failed_tests.append(f'{current}::{match.group(1)}')
        elif match := RESULT.match(line):
            binaries.append({'name': current or '?', 'result': match.group(1),
                             'passed': int(match.group(2)), 'failed': int(match.group(3))})
    return {'schema': 1, 'release_completed': release_completed,
            'binaries': binaries, 'failed_tests': failed_tests,
            'binaries_failed': sum(row['failed'] > 0 for row in binaries)}


def summarize_requests(directory, *, release_completed):
    fixtures = {}
    requests = 0
    for path in sorted(Path(directory).glob('*.tsv')):
        with path.open() as stream:
            for line in stream:
                columns = line.rstrip('\n').split('\t')
                if not line.endswith('\n') or len(columns) != 3 or columns[0] not in ('0', '1'):
                    raise ValueError(f'malformed fixture audit: {path}')
                name, test = (bytes.fromhex(value).decode('utf-8') for value in columns[1:])
                item = fixtures.setdefault(name, {'present_requests': 0, 'missing_requests': 0, 'tests': set()})
                item['present_requests' if columns[0] == '1' else 'missing_requests'] += 1
                item['tests'].add(test)
                requests += 1
    rows = [{'name': name, **item, 'tests': sorted(item['tests'])} for name, item in sorted(fixtures.items())]
    return {'schema': 1, 'release_completed': release_completed,
            'observed_requests': requests, 'coverage_observed': bool(requests),
            'unique_fixtures': len(rows),
            'missing_fixtures': sum(row['missing_requests'] > 0 for row in rows),
            'complete_corpus_claim': False, 'fixtures': rows}


# **The commander's lines** (parked 969, the seventeenth pass): the tests
# that read `docs/QUEUE.md`'s handoff against a pinned constant, and the one
# that fails while an item is both booked and journalled. A worker re-pins
# the constant and may not write the line, so on a lane these are red by
# rule until the merge — and twenty of twenty worker gates of one tranche
# therefore exited 1, none reached `guard`, and four had a red of their own
# among the expected ones. `--lane` reads the release log and says which:
# red only on these, the gate goes on to `guard` and exits 0; red on
# anything else, it stops and names it. The commander's gate takes no flag
# and forgives nothing. `test_release_gate.py` pins that each name is a
# test that exists.
JOURNAL_QUOTE_GUARD = 'a_journal_quotes_its_lane_verdict_and_gate_minutes'
LOOP_LINE_GUARD = 'a_journal_s_loop_line_is_filed_or_says_none'
COMMANDERS_LINES = (
    'the_handoff_s_scoreboard_is_the_floors',
    'the_handoff_s_golden_line_is_the_pinned_word',
    'the_handoff_s_default_map_is_the_lower_word',
    'the_handoff_s_endpoint_is_the_pinned_counts',
    'an_item_number_is_minted_once_and_in_its_file_s_form',
    'the_handoff_s_second_pair_is_the_pinned_words',
    'the_handoff_s_third_pair_is_the_pinned_words',
    'the_handoff_s_third_map_is_the_pinned_word',
    'the_handoff_s_coverage_pair_is_the_pinned_word',
    'the_handoff_s_successor_pair_is_the_pinned_word',
    'the_handoff_s_peace_pair_is_the_pinned_word',
    'the_handoff_s_battery_word_is_the_pinned_word',
    'the_handoff_s_census_is_the_pinned_share',
    # The census's own pin (Loop 2367, the thirty-fourth pass): two sweep
    # lanes re-pinned `census::SIMULATION_BACKED` in the same landing and
    # seven of thirteen merges backed out on it. The shared pin is the
    # commander's at booking, with the `Census:` line; a lane reports the
    # count it measured.
    'the_census_s_simulation_share_is_pinned',
    # The journal's gate quote (Loop 3277, the forty-fourth pass): the
    # guard asks the lane's journal to quote `Lane verdict:` and `Gate
    # minutes:`, lines that exist only once this gate has run, so fourteen
    # of twenty first lane gates were red on it alone. A line the worker
    # cannot write before the gate is forgiven on a lane and held at the
    # booking gate; the verdict says what to add.
    JOURNAL_QUOTE_GUARD,
    # The journal's Loop line (Loop 3328, the forty-fifth pass): the guard
    # asks that a journal's "for the Loop" line be filed in the parked
    # file by number, and only the commander files it, at the merge — a
    # worker never writes the Loop section. 3314's lane gate exited 1
    # three times on it alone, 3318 filed its own line to get past it,
    # and 3322 wrote `none` and moved its note out of the heading.
    # Forgiven on a lane, held at the booking gate; the verdict says
    # whose line it is.
    LOOP_LINE_GUARD,
)
# **The lane gate's suite** (1567): the timed tests over this many seconds
# alone are the booking gate's. The file is the thirty-fifth pass's
# measure by `tools/suite_timings.py` (`docs/audit/2026-10-15-suite-timings.txt`,
# tree bb8dad3f: 849 rondata tests, 3,540 s in all, 323 under a second; 71
# were untimed by the file before it); the pass re-times it
# (`suite_timings.py --stale` says how far it has aged), and a newer
# measure is a new dated file this constant moves to. The memory column
# (`-suite-memory.txt`) is the tool's since this pass and lands with the
# next re-timing.
TIMINGS = ROOT / 'docs/audit/2026-10-15-suite-timings.txt'
LANE_BUDGET_SECONDS = 1.0
RAN_TEST = re.compile(r'^test (\S+) \.\.\. (?:ok|FAILED)$')


def lane_skips(timings, tests):
    """The timed tests a lane gate leaves to the booking gate: every name
    whose seconds alone exceed the budget and which no word filter in
    `tests` names. `timings` is the file's text: `seconds<TAB>name` rows,
    `#` comments."""
    skips = []
    for line in timings.splitlines():
        if not line.strip() or line.startswith('#'):
            continue
        seconds, name = line.split('\t')
        if float(seconds) > LANE_BUDGET_SECONDS and not any(f in name for f in tests):
            skips.append(name)
    return sorted(skips)


# Loop 2415 (the thirty-fifth pass): the lane gate leaves the timed
# `diff::` tests to the booking gate (1567), and both kill rules of one
# tranche tripped by that one route — 2384's 144 `aggression` pins and
# 2400's floor 17379 → 14461 were red only at the booking. A branch that
# touches sim code outside `sweep/` or a `rondata::diff` row list can move
# those pins, so its lane gate skips nothing; one that touches neither — a
# sweep batch, a document — cannot, and keeps the fast suite. The
# whole-suite run before the gate (Loop 2409) is then the gate's own.
# The thirty-fifth steer widened the rule from sim and the `diff` row lists
# to every crate (a parser in `rondata` moves every pin) and the workspace
# files, less the three things a pin cannot see: a sweep test, the census
# count and the blind residue (`blind::NEVER`, the functions the documents
# cite and no trace enters — a sweep that cites one moves it; Loop 3087:
# three scoped gates of the fortieth tranche ran every timed test for it
# alone). `--scope` (below) reads the same rule for the booking gate.
NEVER_WIDENS = ('crates/sim/src/sweep/', 'crates/rondata/src/census.rs', 'crates/rondata/src/blind.rs')


def lane_widens(paths):
    """The paths of a branch that can move the timed `diff::` pins."""
    return sorted(p for p in paths
                  if (p.startswith('crates/') or p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'))
                  and not p.startswith(NEVER_WIDENS))


def branch_paths(base, run=subprocess.run):
    """Every path the branch changed since it left `base`, or None when
    `base` is not a ref this tree knows."""
    try:
        fork = run(['git', 'merge-base', base, 'HEAD'], capture_output=True, text=True,
                   cwd=ROOT, check=True).stdout.strip()
        out = run(['git', 'diff', '--name-only', fork, 'HEAD'], capture_output=True, text=True,
                  cwd=ROOT, check=True).stdout
    except subprocess.CalledProcessError:
        return None
    return out.split()


def base_moved(base, run=subprocess.run):
    """(commits, overlap): the commits `base` has gained since this branch
    left it, and the paths both sides touched since — what a `ccc merge`
    will have to reconcile. (None, None) when `base` is not a ref here.
    Loop 3341 (the forty-fifth pass): 3335 announced its landing on a base
    four landings behind and 3338 on one two behind, and both merges backed
    out at `ccc merge` on the battery tables every promotion edits."""
    try:
        fork = run(['git', 'merge-base', base, 'HEAD'], capture_output=True, text=True,
                   cwd=ROOT, check=True).stdout.strip()
        commits = run(['git', 'log', '--format=%h %s', f'{fork}..{base}'], capture_output=True,
                      text=True, cwd=ROOT, check=True).stdout.splitlines()
        theirs = set(run(['git', 'diff', '--name-only', fork, base], capture_output=True, text=True,
                         cwd=ROOT, check=True).stdout.split())
        ours = set(run(['git', 'diff', '--name-only', fork, 'HEAD'], capture_output=True, text=True,
                       cwd=ROOT, check=True).stdout.split())
    except subprocess.CalledProcessError:
        return None, None
    return [c for c in commits if c.strip()], sorted(theirs & ours)


def base_line(base, commits, overlap):
    """The `Base moved:` line a lane gate prints last: how far the base has
    moved since the branch left it and what that means for the done line."""
    if commits is None:
        return f"Base moved: {base} is not a ref here, so nothing was compared"
    if not commits:
        return f"Base moved: no — {base} has not moved since this branch left it"
    first = commits[0].split(' ', 1)[0]
    last = commits[-1].split(' ', 1)[0]
    line = (f"Base moved: {len(commits)} commit(s) on {base} since this branch left it "
            f"({first}{'..' + last if len(commits) > 1 else ''}); ")
    if overlap:
        shown = ', '.join(overlap[:4]) + (', …' if len(overlap) > 4 else '')
        return line + (f"{len(overlap)} path(s) both sides touched ({shown}): take `ccc update <your ref>` "
                       "(`--keep-conflicts` on a conflict), gate again, and only then your done line — a "
                       "landing announced here backs out at `ccc merge` (Loop 3341)")
    return line + ("no path both sides touched: take `ccc update <your ref>` at this gate's exit and "
                   "land on this verdict if the update is clean, saying which base the gate ran on")


def unmatched_filters(log, tests):
    """The word filters no test that ran contains — read off the release
    log's `test <name> ... ok|FAILED` lines, so an ignored test is not a
    match and neither is a filter cargo never saw."""
    ran = [m.group(1) for raw in Path(log).read_bytes().splitlines()
           if (m := RAN_TEST.match(raw.decode('utf-8', 'replace')))]
    return [f for f in tests if not any(f in name for name in ran)]


# What `cargo test` exits with when a test failed; a kill (memcap's 137) or
# a build error is not a red test and is never forgiven.
CARGO_TESTS_FAILED = 101


def lane_verdict(failed_tests):
    """(forgiven, line): whether every red test is one of the commander's
    lines, and the sentence the gate prints either way."""
    own = [name for name in failed_tests
           if name.rsplit('::', 1)[-1] not in COMMANDERS_LINES]
    if failed_tests and not own:
        remedy = ('; quote this gate\'s `Lane verdict:` and `Gate minutes:` lines in your journal '
                  'before your done line — the booking gate holds the quote (3277)'
                  if any(name.endswith(JOURNAL_QUOTE_GUARD) for name in failed_tests) else '')
        remedy += ('; your journal\'s "for the Loop" line is unfiled, and the commander files it at the '
                   'merge — leave it as written, never file it yourself or rewrite it to `none` (3328)'
                   if any(name.endswith(LOOP_LINE_GUARD) for name in failed_tests) else '')
        return True, (f"Lane verdict: red only on the commander's lines "
                      f"({len(failed_tests)}): " + ', '.join(failed_tests) + remedy)
    return False, (f"Lane verdict: red on the worker's own ({len(own)}): "
                   + (', '.join(own) or 'no failed test named; the suite did not finish'))


def journal_of(item, root=ROOT):
    """The journal files of an item in the tree, `docs/journal/<date>-item-N.md`."""
    return sorted(Path(root, 'docs/journal').glob(f'*-item-{int(item)}.md'))


# The release suite's width and its ceiling (the thirty-fifth steer, Ramon's
# word): four threads under 20 GiB is a lane's, three of which may gate at
# once; six or eight under 40 GiB is the booking gate's, which runs alone.
# The box has 128 GiB; the widest single test binary measured 11.8 GiB at
# four threads, and whether that is one test or four is the timing file's
# memory column to say.
WIDTHS = (2, 3, 4, 6, 8)


def memory_cap_gib(test_threads):
    return 20 if test_threads <= 4 else 40


def gate(install, *, report_dir=None, require_fixtures=False, test_threads=2, run=run_logged,
         lane=False, tests=None, item=None, base='worktree-replan-pdb', scope=None, isolated=None,
         no_widen=False):
    # Parked 1713: two lanes `pkill -f release_gate.py`'d every gate on the
    # box to stop their own; a gate names its pid first so it can be killed
    # by it.
    print(f"Gate pid: {os.getpid()} — kill this gate by its pid, never by `pkill -f` on a "
          'name every lane shares (parked 1713)', flush=True)
    if type(test_threads) is not int or test_threads not in WIDTHS:
        raise ValueError(f'test threads must be one of {WIDTHS}')
    if scope is not None and lane:
        raise ValueError('--scope is the booking gate\'s; a lane gate has --tests and --base')
    if isolated is None:
        isolated = True
    tests = list(tests or [])
    if item is not None and not lane:
        raise ValueError('--item is a lane gate\'s: the booking gate reads the queue, not a journal')
    if lane and item is not None and not journal_of(item):
        # Parked 1699: a landing's chain died before its journal and the
        # gate ran and the done line went out without it.
        raise ValueError(f'no docs/journal/<date>-item-{item}.md in the tree: a landing writes '
                         'its journal before it gates (parked 1699)')
    if lane and not tests:
        raise ValueError("a lane gate names its word's tests: --tests FILTER [FILTER ...] "
                         '(the widening, the pinned walk); the full suite is the booking gate\'s')
    if tests and not lane:
        raise ValueError('--tests is a lane gate\'s; the booking gate runs the whole suite')
    if no_widen and not lane:
        raise ValueError('--no-widen is a lane gate\'s: the booking gate always widens on a path that can move a pin')
    if any(not f.strip() for f in tests):
        raise ValueError('an empty --tests filter would keep every slow test')
    skips, widened, touched = [], [], None
    if lane or scope is not None:
        touched = branch_paths(base if lane else scope)
        widened = lane_widens(touched or [])
        # DECISIONS 83 (the forty-fourth steer): every lane of one tranche
        # touched a sim path and every lane gate ran the whole timed suite
        # at four threads (6–26 minutes, median 18.9) while fifteen of the
        # twenty moved no pin. The AI lane's brief widens; a sweep's or a
        # chapter's says `--no-widen`, and the booking gate holds the pins.
        skips = [] if widened and not no_widen else lane_skips(TIMINGS.read_text(), tests)
    install = Path(install).resolve()
    if not (install / 'Data/rules.xml').is_file():
        raise ValueError(f'not an install: missing {install / "Data/rules.xml"}')
    if report_dir is None:
        report_dir = Path(tempfile.mkdtemp(prefix='attrition-gate-'))
    else:
        report_dir = Path(report_dir).resolve()
        report_dir.mkdir(parents=True, exist_ok=False)
    audit_dir = report_dir / 'fixture-requests'
    audit_dir.mkdir()
    env = child_environment(dispatcher=isolated)
    env.pop('RON_FIXTURE_AUDIT_DIR', None)
    env.pop('RON_TEST_MEMCAP_GIB', None)
    env['RUST_TEST_THREADS'] = '2'
    env['RON_INSTALL'] = str(install)
    env.pop('RON_GUARD_ADMITTED', None)
    env['RON_GATE_RESOURCE_POLICY'] = '1'
    env['CARGO_BUILD_JOBS'] = '8'
    # The whole gate — every third landing and the drain (DECISIONS 72) — runs
    # the on-disk caches' verifiers: each cached oracle frame and setup record
    # is compared with a fresh parse, so a schema hash that missed a parser's
    # file is caught on that schedule and not by a wrong floor (DECISIONS 75).
    whole = scope is None and not lane
    if whole:
        env['RON_VERIFY_ORACLE'] = '1'
        env['RON_VERIFY_SETUP_OBSERVATIONS'] = '1'
    policy = {'schema': 1, 'release_test_threads': test_threads,
              'test_allocator': allocator_policy(env),
              'verify_caches': whole,
              'memory_cap_gib': 32 if isolated else min(32, memory_cap_gib(test_threads)),
              'shared_admission': {'global_cpu_slots': HOST_CPU_SLOTS, 'global_reserved_mib': HOST_MEMORY_MIB,
                                   'cargo_build_jobs': 8, 'all_phases': True,
                                   'off_marker': str(OFF_MARKER), 'off': OFF_MARKER.exists()}}
    if isolated:
        policy['isolated'] = {'slots': 8, 'global_cpu_slots': HOST_CPU_SLOTS, 'global_reserved_mib': HOST_MEMORY_MIB}
    if scope is not None:
        # The thirty-fifth steer (Ramon's word): a booking gate scoped to the
        # merges since the last whole one runs the timed tests only when one
        # of them can move a pin; every third landing, and the drain, take
        # the whole gate with no --scope. Forgives nothing, as ever.
        policy['scope'] = {'base': scope, 'widened': widened, 'skipped': len(skips),
                           'timings': str(TIMINGS.relative_to(ROOT))}
    if lane:
        policy['lane'] = {'tests': tests, 'timings': str(TIMINGS.relative_to(ROOT)), 'no_widen': bool(no_widen),
                          'budget_seconds': LANE_BUDGET_SECONDS, 'skipped': len(skips),
                          'base': base, 'widened': widened}
    (report_dir / 'gate-policy.json').write_text(json.dumps(policy, indent=2) + '\n')
    # No game launch: rondata surveys the user's data files.
    #
    # clippy and fmt run **before** the release suite (parked 639, the
    # thirteenth pass): a worker whose suite is red only on the queue's
    # lines — the commander's half, by rule — stopped here before clippy
    # ever ran, reported "red only on your lines", and the commander's
    # booking gate then failed on a `collapsible_if` in that worker's own
    # code. The cheap checks go first so a paperwork-red gate has still
    # said everything it can about the code; and the steps line at the
    # end says which steps ran, so "red only on X" is read off the gate
    # and never inferred.
    commands = [
        [sys.executable, 'tools/offline_tests.py'],
        ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'],
        ['cargo', 'fmt', '--check'],
        ['cargo', 'run', '-p', 'rondata', '--', str(install)],
        ['zsh', 'tools/memcap.sh', str(min(32, memory_cap_gib(test_threads))), 'cargo', 'test', '--release',
         '--no-fail-fast', '--', f'--test-threads={test_threads}']
        + (['--exact'] + [arg for name in skips for arg in ('--skip', name)] if (lane or scope is not None) else []),
        # The guards ran in the release suite already; the reflex script
        # re-runs them on the release binary in seconds (the thirty-fifth
        # steer: a debug build a gate was three minutes, forty-four gates a
        # tranche).
        ['zsh', 'tools/guard.sh', '--built'],
    ]
    if isolated:
        commands[4] = [sys.executable, 'tools/isolated_tests.py', '--release',
                       '--output', str(report_dir / 'isolated'), '--test-threads', str(test_threads)] + [arg for name in skips for arg in ('--skip', name)]
    if scope is not None:
        print((f"Booking gate scoped to {scope}: " +
               (f"every timed test — a merge touches {len(widened)} path(s) that can move the pins ({widened[0]}"
                f"{', …' if len(widened) > 1 else ''})"
                if widened else
                f"{len(skips)} timed tests left to the next whole gate" +
                (f"; {scope} is not a ref here, so nothing widened" if touched is None else ''))), flush=True)
    if lane and widened and no_widen:
        print(f"Lane suite: {len(skips)} timed tests over {LANE_BUDGET_SECONDS:g} s alone left to the "
              f"booking gate by the brief's --no-widen, though the branch touches {len(widened)} path(s) "
              f"that can move a pin ({widened[0]}{', …' if len(widened) > 1 else ''}); the booking gate "
              "holds the pins (DECISIONS 83)", flush=True)
    elif lane and widened:
        print(f"Lane suite: every timed test — the branch touches {len(widened)} path(s) that can move "
              f"the diff:: pins ({widened[0]}{', …' if len(widened) > 1 else ''}; Loop 2415); "
              f"the word's: {', '.join(tests)}", flush=True)
    elif lane:
        print(f"Lane suite: {len(skips)} timed tests over {LANE_BUDGET_SECONDS:g} s alone left to the "
              f"booking gate ({TIMINGS.relative_to(ROOT)}); the word's: {', '.join(tests)}"
              + (f"; base {base} is not a ref here, so nothing widened" if touched is None else ''),
              flush=True)
    reached = []
    seconds = []
    load_at_start = host_load()
    try:
        _run_steps(commands, run=run, env=env, audit_dir=audit_dir, report_dir=report_dir,
                   require_fixtures=require_fixtures, reached=reached, lane=lane, word=tests,
                   seconds=seconds)
        if lane:
            # Loop 3341: the base is read after the steps, so the line is
            # the freshest a lane can have before its done line.
            commits, overlap = base_moved(base)
            print(base_line(base, commits, overlap), flush=True)
            if overlap:
                raise ValueError(f"the base moved under this branch and both sides touched "
                                 f"{', '.join(overlap)}: take `ccc update <your ref>` and gate again "
                                 "before your done line (Loop 3341)")
    finally:
        print(minutes_line([step_name(c) for c in commands], seconds), flush=True)
        print(steps_line(commands, reached), flush=True)
        print(admission_line(admission_summary(report_dir)), flush=True)
        print(load_line(load_at_start, host_load(), os.cpu_count() or 0), flush=True)
    return report_dir


def host_load():
    """The one-minute load average, or None where the platform has none."""
    try:
        return os.getloadavg()[0]
    except (OSError, AttributeError):
        return None


def load_line(start, end, cores):
    """Loop 3091 (the forty-first pass): the admission receipt says what a
    gate waited at the door and nothing about what it ran into once
    admitted — the fortieth tranche's lane gates ran 18–32 minutes against
    9.5 while the receipt read seconds, and Ramon read a load of 32 on
    sixteen cores off `top` (Loop 3143). The box's load at the gate's start
    and end, beside its cores, is the contention a pass reads off the gate
    itself; a load past the cores is dilation, whatever the receipt said."""
    def part(v):
        return 'n/a' if v is None else f'{v:.1f}'
    return f"Host load: {part(start)} at start, {part(end)} at end, on {cores} cores"


def admission_summary(report_dir):
    """The trial's second number (DECISIONS 75): the seconds this gate's
    leases spent waiting for host admission, summed over every `resource
    budget timing:` receipt the report's logs carry — the isolated runner's
    per-test logs and the steps' own — written to `admission.json`."""
    waits = []
    for log in sorted(Path(report_dir).rglob('*.log')):
        with open(log, errors='replace') as stream:
            for line in stream:
                if line.startswith('resource budget timing: '):
                    try:
                        waits.append(float(json.loads(line.split(': ', 1)[1])['wait_seconds']))
                    except (ValueError, KeyError, TypeError):
                        pass
    summary = {'schema': 1, 'leases': len(waits), 'wait_seconds': round(sum(waits), 1),
               'longest_wait_seconds': round(max(waits), 1) if waits else 0.0,
               'admission_off': OFF_MARKER.exists()}
    (Path(report_dir) / 'admission.json').write_text(json.dumps(summary, indent=2) + '\n')
    return summary


def admission_line(summary):
    return (f"Admission wait: {summary['wait_seconds']} s over {summary['leases']} leases, "
            f"longest {summary['longest_wait_seconds']} s" + (' (admission off)' if summary['admission_off'] else ''))


def step_name(command):
    """The step a gate command is, in the words the handoff uses."""
    if command[-1].endswith('offline_tests.py'):
        return 'offline'
    if 'clippy' in command:
        return 'clippy'
    if 'fmt' in command:
        return 'fmt'
    if command[:3] == ['cargo', 'run', '-p']:
        return 'survey'
    if '--release' in command:
        return 'release'
    if any(str(a).endswith('guard.sh') for a in command):
        return 'guard'
    return command[-1]


def minutes_line(names, seconds):
    """`Gate minutes: offline 0.3, clippy 1.9, …; not reached: …` — where a
    gate's minutes went, step by step (the forty-second pass: the lane gate
    is two thirds of what a landing waits, and the receipt said only that
    its steps ran)."""
    ran = [f'{n} {s / 60:.1f}' for n, s in zip(names, seconds)]
    rest = names[len(seconds):]
    line = 'Gate minutes: ' + (', '.join(ran) or 'none ran')
    if rest:
        line += f"; not reached: {', '.join(rest)}"
    return line


def steps_line(commands, reached):
    """`Gate steps: <ran>; not reached: <rest>` — the last line a gate prints.

    A step that ran and failed is on the ran side (its own output says
    so); the point of the line is the other side: what a red gate never
    got to, which is what a worker's "red only on my lines" claim has to
    be checked against.
    """
    names = [step_name(c) for c in commands]
    ran = names[:len(reached)]
    rest = names[len(reached):]
    line = f"Gate steps: {len(ran)} of {len(names)} ran ({', '.join(ran) or 'none'})"
    if rest:
        line += f"; not reached: {', '.join(rest)}"
    return line


def _run_steps(commands, *, run, env, audit_dir, report_dir, require_fixtures, reached,
               lane=False, word=(), seconds=None):
    for command in commands:
        began = time.monotonic()
        child_env = env.copy()
        is_release = '--release' in command
        completed = False
        held = None
        if is_release:
            child_env['RON_FIXTURE_AUDIT_DIR'] = str(audit_dir)
        if lane and step_name(command) == 'guard':
            child_env['RON_LANE'] = '1'
        extra = {'log': str(report_dir / ('release-tests.log' if is_release else step_name(command) + '.log'))}
        try:
            reached.append(command)
            run(command, cwd=ROOT, env=child_env, check=True, **extra)
            completed = True
        except subprocess.CalledProcessError as failure:
            if not (lane and is_release and failure.returncode == CARGO_TESTS_FAILED):
                raise
            held = failure
        finally:
            if seconds is not None:
                seconds.append(time.monotonic() - began)
            if is_release:
                summary = summarize_requests(audit_dir, release_completed=completed)
                output = report_dir / 'fixture-coverage.json'
                output.write_text(json.dumps(summary, indent=2) + '\n')
                print(f"Fixture audit: {summary['observed_requests']} requests, "
                      f"{summary['missing_fixtures']} missing fixtures; {output}", flush=True)
                for row in summary['fixtures']:
                    if row['missing_requests']:
                        print(f"  missing: {row['name']}", flush=True)
                tests = summarize_tests(extra['log'], release_completed=completed) \
                    if Path(extra['log']).is_file() else \
                    {'schema': 1, 'release_completed': completed, 'binaries': [],
                     'failed_tests': [], 'binaries_failed': 0}
                (report_dir / 'test-summary.json').write_text(json.dumps(tests, indent=2) + '\n')
                print('Test summary: ' + ('; '.join(
                    f"{row['name']} {row['result']} {row['passed']} passed"
                    + (f" / {row['failed']} failed" if row['failed'] else '')
                    for row in tests['binaries']) or 'no test result observed'), flush=True)
                for name in tests['failed_tests']:
                    print(f"  failed: {name}", flush=True)
        if held is not None:
            forgiven, line = lane_verdict(tests['failed_tests'])
            print(line, flush=True)
            if not forgiven:
                raise held
        if is_release:
            if not summary['observed_requests']:
                raise ValueError('release produced no fixture audit; coverage is unobserved')
            if not tests['binaries']:
                raise ValueError('release printed no test result; the suite is unobserved')
            if require_fixtures and summary['missing_fixtures']:
                raise ValueError('requested fixtures are missing; see fixture-coverage.json')
            if word and (missed := unmatched_filters(extra['log'], word)):
                raise ValueError(f"the word's filters matched no test that ran: {', '.join(missed)}")


STALE_AGENT_HOURS = 6


def stale_agent_worktrees(porcelain, age_hours, *, limit=STALE_AGENT_HOURS):
    """Loop 2724: the `agent-*` worktrees `git worktree list --porcelain`
    names whose age (`age_hours(path)`, None when unknown) is past the limit.
    The harness cuts a subagent's worktree under the main checkout's
    `.claude/worktrees/`, every lane's in one directory; a sweep's go with
    its landing (the frame's row), and three of the thirty-fifth tranche's
    stood a day, two with unmerged tips, named by nothing."""
    out = []
    for block in porcelain.strip().split('\n\n'):
        head = block.strip().split('\n', 1)[0]
        if not head.startswith('worktree '):
            continue
        path = head[len('worktree '):].strip()
        if '/.claude/worktrees/agent-' not in path:
            continue
        age = age_hours(path)
        if age is not None and age > limit:
            out.append((path, age))
    return out


def refuse_stale_agent_worktrees(stale, *, lane):
    """The booking gate refuses; a lane gate never looks, because the
    directory is every lane's and a red would land on the wrong lane."""
    if lane or not stale:
        return
    names = ', '.join(f'{Path(p).name} ({h:.1f} h)' for p, h in stale)
    raise ValueError(f'{len(stale)} stale agent-* worktree(s) under the main checkout, past '
                     f'{STALE_AGENT_HOURS} h: {names} — a sweep\'s go with its landing (Loop 2724); '
                     f'`git worktree remove` each once read, or `git worktree prune`')


def stale_agent_worktrees_now():
    porcelain = subprocess.run(['git', 'worktree', 'list', '--porcelain'], cwd=ROOT, check=True,
                               capture_output=True, text=True).stdout
    now = time.time()

    def age_hours(path):
        try:
            return (now - os.path.getmtime(path)) / 3600
        except OSError:
            return None
    return stale_agent_worktrees(porcelain, age_hours)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path, help='owned install directory; always exported as RON_INSTALL')
    parser.add_argument('--report-dir', type=Path, help='fresh output directory; defaults to a retained temporary directory')
    parser.add_argument('--require-fixtures', action='store_true', help='fail if any observed fixture request was missing')
    parser.add_argument('--test-threads', type=int, choices=WIDTHS, default=2,
                        help='release width: 2-4 under 20 GiB in combined mode, 6/8 under 32 GiB; '
                             'isolated rondata tests each use one thread; default: 2')
    execution = parser.add_mutually_exclusive_group()
    execution.add_argument('--isolated', action='store_true', default=None,
                           help='isolate rondata tests (default for all gate modes)')
    execution.add_argument('--combined', action='store_false', dest='isolated',
                           help='reference combined runner; still shares host resource admission')
    parser.add_argument('--lane', action='store_true',
                        help="a worker's gate: the suite without the slow timed tests, plus --tests; "
                             "red only on the commander's lines goes on to guard and exits 0")
    parser.add_argument('--tests', nargs='+', metavar='FILTER',
                        help="with --lane, required: the word's own tests, kept however slow; "
                             'each must match a test that ran')
    parser.add_argument('--no-widen', action='store_true',
                        help="with --lane: leave the timed tests to the booking gate even on a branch that can "
                             "move a pin — the sweep and chapter kinds' (DECISIONS 83); the AI lane widens")
    parser.add_argument('--item', type=int, metavar='N',
                        help='with --lane: the item; the gate refuses a tree with no '
                             'docs/journal/*-item-N.md (parked 1699)')
    parser.add_argument('--scope', metavar='REF',
                        help='the booking gate\'s: the tip the last whole gate ran on; the timed tests run only '
                             'when a merge since it can move a pin (every third landing takes no --scope)')
    parser.add_argument('--base', default='worktree-replan-pdb', metavar='REF',
                        help='with --lane: the branch the lane was cut from; a branch that touches '
                             'sim code outside sweep/ or a rondata::diff row list since it runs every '
                             'timed test (Loop 2415); default: worktree-replan-pdb')
    args = parser.parse_args()
    try:
        refuse_stale_agent_worktrees(stale_agent_worktrees_now(), lane=args.lane)
        gate(args.install, report_dir=args.report_dir, require_fixtures=args.require_fixtures,
             test_threads=args.test_threads, lane=args.lane, tests=args.tests, item=args.item,
             base=args.base, scope=args.scope, isolated=args.isolated, no_widen=args.no_widen)
    except ValueError as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    main()
