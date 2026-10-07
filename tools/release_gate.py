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
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]

RUNNING = re.compile(r'^\s*Running (?:unittests )?\S+ \((\S+)\)$')
DOC_TESTS = re.compile(r'^\s*Doc-tests (\S+)$')
RESULT = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;')
FAILED_TEST = re.compile(r'^test (\S+) \.\.\. FAILED$')


def run_logged(command, *, cwd, env, check, log=None):
    """`subprocess.run`, or — with `log` — the same command with its output
    copied line by line to that file as well as to stdout. No shell pipe
    stands between the command and its exit status: a `| tee` launders
    `memcap.sh`'s 137 into a 0, and the release command's exit code is the
    gate's verdict."""
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
    'the_handoff_s_census_is_the_pinned_share',
)
# **The lane gate's suite** (1567): the timed tests over this many seconds
# alone are the booking gate's. The file is the twenty-seventh pass's
# measure by `tools/suite_timings.py` (`docs/audit/2026-10-07-suite-timings.txt`,
# tree 850a2ef8: 475 of 779 rondata tests over a second, 2,818 s in all);
# the pass re-times it (`suite_timings.py --stale` says how far it has
# aged — 14 untimed and 6 gone when this one was made), and a newer
# measure is a new dated file this constant moves to.
TIMINGS = ROOT / 'docs/audit/2026-10-07-suite-timings.txt'
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
        return True, (f"Lane verdict: red only on the commander's lines "
                      f"({len(failed_tests)}): " + ', '.join(failed_tests))
    return False, (f"Lane verdict: red on the worker's own ({len(own)}): "
                   + (', '.join(own) or 'no failed test named; the suite did not finish'))


def gate(install, *, report_dir=None, require_fixtures=False, test_threads=2, run=run_logged,
         lane=False, tests=None):
    if type(test_threads) is not int or test_threads not in (2, 3, 4):
        raise ValueError('test threads must be 2, 3, or 4')
    tests = list(tests or [])
    if lane and not tests:
        raise ValueError("a lane gate names its word's tests: --tests FILTER [FILTER ...] "
                         '(the widening, the pinned walk); the full suite is the booking gate\'s')
    if tests and not lane:
        raise ValueError('--tests is a lane gate\'s; the booking gate runs the whole suite')
    if any(not f.strip() for f in tests):
        raise ValueError('an empty --tests filter would keep every slow test')
    skips = lane_skips(TIMINGS.read_text(), tests) if lane else []
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
    env = os.environ.copy()
    env.pop('RON_FIXTURE_AUDIT_DIR', None)
    env.pop('RON_TEST_MEMCAP_GIB', None)
    env['RUST_TEST_THREADS'] = '2'
    env['RON_INSTALL'] = str(install)
    policy = {'schema': 1, 'release_test_threads': test_threads, 'memory_cap_gib': 20}
    if lane:
        policy['lane'] = {'tests': tests, 'timings': str(TIMINGS.relative_to(ROOT)),
                          'budget_seconds': LANE_BUDGET_SECONDS, 'skipped': len(skips)}
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
        ['zsh', 'tools/memcap.sh', '20', 'cargo', 'test', '--release', '--no-fail-fast', '--', f'--test-threads={test_threads}']
        + (['--exact'] + [arg for name in skips for arg in ('--skip', name)] if lane else []),
        ['zsh', 'tools/guard.sh'],
    ]
    if lane:
        print(f"Lane suite: {len(skips)} timed tests over {LANE_BUDGET_SECONDS:g} s alone left to the "
              f"booking gate ({TIMINGS.relative_to(ROOT)}); the word's: {', '.join(tests)}", flush=True)
    reached = []
    try:
        _run_steps(commands, run=run, env=env, audit_dir=audit_dir, report_dir=report_dir,
                   require_fixtures=require_fixtures, reached=reached, lane=lane, word=tests)
    finally:
        print(steps_line(commands, reached), flush=True)
    return report_dir


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
    if command[-1].endswith('guard.sh'):
        return 'guard'
    return command[-1]


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
               lane=False, word=()):
    for command in commands:
        child_env = env.copy()
        is_release = '--release' in command
        completed = False
        held = None
        if is_release:
            child_env['RON_FIXTURE_AUDIT_DIR'] = str(audit_dir)
        if lane and step_name(command) == 'guard':
            child_env['RON_LANE'] = '1'
        extra = {'log': str(report_dir / 'release-tests.log')} if is_release else {}
        try:
            reached.append(command)
            run(command, cwd=ROOT, env=child_env, check=True, **extra)
            completed = True
        except subprocess.CalledProcessError as failure:
            if not (lane and is_release and failure.returncode == CARGO_TESTS_FAILED):
                raise
            held = failure
        finally:
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path, help='owned install directory; always exported as RON_INSTALL')
    parser.add_argument('--report-dir', type=Path, help='fresh output directory; defaults to a retained temporary directory')
    parser.add_argument('--require-fixtures', action='store_true', help='fail if any observed fixture request was missing')
    parser.add_argument('--test-threads', type=int, choices=(2, 3, 4), default=2,
                        help='release width under the 20 GiB monitor; default: 2')
    parser.add_argument('--lane', action='store_true',
                        help="a worker's gate: the suite without the slow timed tests, plus --tests; "
                             "red only on the commander's lines goes on to guard and exits 0")
    parser.add_argument('--tests', nargs='+', metavar='FILTER',
                        help="with --lane, required: the word's own tests, kept however slow; "
                             'each must match a test that ran')
    args = parser.parse_args()
    try:
        gate(args.install, report_dir=args.report_dir, require_fixtures=args.require_fixtures,
             test_threads=args.test_threads, lane=args.lane, tests=args.tests)
    except ValueError as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    main()
