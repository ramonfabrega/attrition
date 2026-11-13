#!/usr/bin/env python3
"""Run the full local gate with an explicit install, including its data survey.

**Two gates, one script** (1567, `docs/DECISIONS.md` 63 amended). With no
flag this is the commander's booking gate: six steps, the whole release
suite, nothing forgiven. With `--lane` it is a worker's, and since the
twenty-seventh tranche it no longer runs the whole suite — every landing
ran it three times on one box (the worker's own run, its lane gate, the
booking gate), and a third word lane bought no wall clock for that reason.

The lane gate's release step runs:

  * every test of every crate that the timings file **does not name** —
    the file (`TIMINGS`, every rondata release test alone, one at a time)
    names the rondata tests it timed, and since the pass of 2026-11-13
    (second, DECISIONS 132) a lane's own budget is zero
    (`LANE_OWN_BUDGET_SECONDS`): every row it names is `--skip`ped by exact
    name, where the scoped booking gate skips only the rows over
    `LANE_BUDGET_SECONDS`. A test the file does not name (the worker's new
    widening among them) runs, and so do the crates it does not time; a
    paperwork guard (`PAPERWORK_GUARDS`: `writers::`, `ledger::`,
    `docs_guard`, `the_handoff_s_`) runs whatever its timing (item 3373),
    and so do the pins a lane's rows answer to (`LANE_PINS`: the floors
    guards and the census pin);
  * **the word's own tests**, `--tests FILTER ...`, which the brief names
    and the lane gate refuses to run without: a timed test that contains a
    filter is kept, however slow, and a filter that matched no test that
    ran turns the gate red — a typo is not a pass;
  * and **never** the two coverage pins or the rest of the slow walks
    (`coverage::every_key_the_dump_prints_is_read_or_pinned`, 118 s alone;
    `..._every_parsed_field_is_compared_by_the_instrument_or_pinned`, 393 s
    alone on 2026-10-27 — 345 s nine windows earlier, so about five
    seconds a window the coverage driver is handed, and four to five
    times that inside a whole gate at `--test-threads 8`, which is why its
    budget is `test_runtime.py`'s 3600 s and not the default 1800):
    those, and the whole suite, are the booking gate's. A worker whose
    landing reads a new key passes `--tests coverage::` and runs them.

A branch that can move a timed pin widens the lane gate to the whole
suite (`lane_widens`, Loop 2415); since the fifty-fifth pass a pair
stand-up's row-list edits — its `mod` line, its scored triple, its ledger
row, its new module, and since the fifty-sixth its coverage window line
and its two test-kit rows — do not (`row_only_edits`, Loop 3608 and its
sequel), and the receipt's `row_only` names them.
A lane whose item moves no test (a retirement, a tooling item) gates with
`--lane --whole` in place of `--tests`: every test, the timed ones
included, with the lane's forgiveness and `Lane verdict:` (Loop 3489). Since the fifty-seventh pass a lane that touches no path
under `crates/sim/` — nor the workspace manifest, the lock, the toolchain
pin or cargo's config — also leaves the sim crate's five test binaries — the four
emulator sweep targets and the lib — to the booking gate (`sim_crate_left`,
Loop 3653: 680 of a stand-up's 1,044 release seconds at four threads were theirs,
on binaries its branch cannot change), widened or not since the fifty-eighth
(the cut had been tied to the fast suite, and two stand-ups that widened on one
data-crate file each ran the binaries for eleven minutes apiece); the receipt's
`sim_crate` says which. On such a lane a `--tests` filter whose every
match is a sim test is dropped and named (`Lane filter dropped:`, Loop
3823) rather than turning the gate red for matching nothing that ran.
Since the pass of 2026-11-13 (second, DECISIONS 132 §6) a lane that touches
no path under `crates/client/` or `crates/render/` — nor the four workspace
inputs — leaves the client crate's targets, its wall-clock `clock` test
among them, to the booking gate the same way (`client_crate_left`; the
receipt's `client_crate`), and drops a filter whose every match is a client
test.

The census pin runs in every lane gate (a lane pin, `LANE_PINS`) and its red is
forgiven as the commander's line (Loop 2367); since the seventy-fifth pass
the forgiven verdict carries the measured (backed, port, functions) against
the pin's, read off the assertion's panic, so the journal's quoted verdict
hands the commander the re-pin (`census_moved`, Loop 4030).

The worker's own full run, measured on the tree after its last `ccc
update`, is still owed before the gate (the brief's "measure" rows); the
lane gate is the confirmation of the word and the paperwork, not the
search for a re-pin. Suites a landing: three to two.
"""
import argparse
import fcntl
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
from resource_budget import CPU as HOST_CPU_SLOTS, MEMORY_MIB as HOST_MEMORY_MIB, OFF_MARKER, admission_off

# One whole lane gate at a time, host-wide (the tranche of 2026-11-12).
WHOLE_LANE_LOCK = Path.home() / 'Library/Caches/attrition/whole-lane.lock'

RUNNING = re.compile(r'^\s*Running (?:unittests )?\S+ \((\S+)\)$')
DOC_TESTS = re.compile(r'^\s*Doc-tests (\S+)$')
RESULT = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;')
FAILED_TEST = re.compile(r'^test (\S+) \.\.\. FAILED$')


def phase_budget(command):
    """Small already-built checks can coexist; possible compilations drain slots."""
    if (any(str(a).endswith(('offline_tests.py', 'shape.py')) for a in command)
            or 'fmt' in command):
        return 2, 1024
    if any(str(a).endswith('guard.sh') for a in command) and '--built' not in command:
        return 2, 4096  # the reflex's reservation (3 GiB measured); its rebuild ceiling is phase_limit's
    # The gate's `guard.sh --built` runs before the release step since Loop
    # 4070, so it is where the release test binaries are built: a build's
    # reservation, as clippy's and the survey's.
    return 8, 32768


def phase_limit(command):
    """The sampled RSS ceiling where it is not the reservation: the guard's
    debug rebuild after an edit (DECISIONS 75)."""
    if any(str(a).endswith('guard.sh') for a in command) and '--built' not in command:
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
COMMANDERS_LINES = (
    'the_handoff_s_scoreboard_is_the_floors',
    # The blind list's line (Loop 3416, the forty-eighth pass).
    'the_handoff_s_blind_list_is_the_pinned_count',
    # The held-out battery's line (the forty-eighth pass).
    'the_handoff_s_held_out_line_is_the_pinned_battery',
    # The rubric's line (the forty-ninth pass, Ramon's: the steer's pin).
    'the_handoff_s_rubric_is_the_pinned_scorecard',
    # The structure track's line (item 4008, STRUCTURE §2); the pin's own
    # half, `the_pinned_shape_is_the_tree_s`, is a worker's and not here.
    'the_handoff_s_structure_line_is_the_pinned_shape',
    # The renderer track's line (the seventy-fifth pass, RENDERER §5.4).
    'the_handoff_s_renderer_line_is_the_pinned_counter',
    # The start track's line (item 4033, START §5).
    'the_handoff_s_start_line_is_the_pinned_counter',
    # The checksum track's line (item 4060, CHECKSUM §5).
    'the_handoff_s_checksum_line_is_the_pinned_counter',
    # The lobby page's line (item 4184, LOBBY §5).
    'the_handoff_s_lobby_line_is_the_pinned_counter',
    # The HUD's line (item 4197, HUD §5).
    'the_handoff_s_hud_line_is_the_pinned_counter',
    # The renderer's clock line (item 4161, RENDERER §3.11).
    'the_handoff_s_clock_line_is_the_pinned_ceilings',
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
    'the_handoff_s_late_ages_are_the_pinned_words',
    # The melee's line (item 3733, GOLDEN §149).
    'the_handoff_s_melee_is_the_pinned_word',
    # The map generator's line (item 3776, MAPGEN §15).
    'the_handoff_s_mapgen_is_the_pins',
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
    # Not forgiven, and not this one's to forgive (Loop 3449): the quote
    # this forgiveness waits for is written after the lane gate, and a
    # quote that cites a red by `name@address` is a citation
    # `blind::tests::the_blind_list_is_the_pinned_residue` reads — 3442's
    # booking gate was red on it alone. `blind::` is the worker's own red;
    # `tools/guard.sh`, the reflex run after the quote, runs it.
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
# **Two budgets** (the pass of 2026-11-13 (second), DECISIONS 132). The
# scoped booking gate's is a second, as since 1567: a timed row over it is
# left to the next whole gate. A lane gate's own is zero: every row the
# file names is left to the booking gate unless a `--tests` filter names
# it, it is a paperwork guard, or it is a pin a lane's own rows answer to
# (`LANE_PINS`); a test the file has never timed — the lane's own new
# tests among them — still runs. Measured by the pass (four unwidened
# `--no-widen` lane gates of items 4200, 4201 and 4203, at four threads):
# release 11.0–12.6 minutes, of which the brief's filters were 645–740
# test-seconds and the paperwork 11–18; the rest, 1,258–1,690
# test-seconds and five to seven minutes of wall, was ~920 rows the file
# times under a second, which a loaded host ran at 1.6–2.0 s each — work
# the booking gate repeats on the merged tree. In 43 gate logs no red was
# a test a scoped gate would have skipped. A zero budget leaves a row the
# file timed at 0.00 too (`over_budget`): the file names it, so it is the
# booking gate's.
LANE_BUDGET_SECONDS = 1.0
LANE_OWN_BUDGET_SECONDS = 0.0
RAN_TEST = re.compile(r'^test (\S+) \.\.\. (?:ok|FAILED)$')
# **The paperwork guards run in every lane gate, whatever their timing**
# (item 3373, the forty-sixth pass): `writers::a_field_the_sim_writes_is_
# read_by_something` timed at 1.08 s, so `--lane --no-widen` left it to the
# booking gate, and 3362's booking gate went red on it after a green lane
# gate. A guard that reads the tree's own source or paperwork is cheap and
# is the worker's to keep green, so the budget never skips a test whose
# name contains one of these: the field-writer guards (`writers::`), the
# widening ledger (`ledger::`), the sim's paperwork guard (`docs_guard`,
# not timed today, named so a re-timing cannot skip it) and the handoff's
# readers of `docs/QUEUE.md` (`the_handoff_s_`, the commander's lines, which
# a lane forgives by name and still runs).
PAPERWORK_GUARDS = ('writers::', 'ledger::', 'docs_guard', 'the_handoff_s_')
# **And the pins a lane's own rows answer to, under the lane's zero budget**
# (the same pass). Under the second's budget every one of these ran in
# every lane gate, being under it, and two rules lean on that: a row-list
# edit does not widen a lane because the test-kit rows are held by "the
# fast floors guards every lane gate runs" (Loop 3608's sequel, above
# `ROW_LISTS`), and a forgiven census red hands the commander the re-pin
# only if the lane ran the census pin (Loop 4030 — 0.72 s alone; a sweep
# lane moves it and names no `census::`). `diff::floors::` is ~60 tests
# timed at 0.00–0.36 s each; kept, they cost seconds of wall.
LANE_PINS = ('diff::floors::', 'the_census_s_simulation_share_is_pinned')
LANE_KEPT = PAPERWORK_GUARDS + LANE_PINS


def _timed_rows(timings):
    for line in timings.splitlines():
        if not line.strip() or line.startswith('#'):
            continue
        seconds, name = line.split('\t')
        yield float(seconds), name


def is_paperwork(name):
    return any(p in name for p in PAPERWORK_GUARDS)


def is_kept(name, kept=PAPERWORK_GUARDS):
    """Whether a budget never skips `name`: it contains one of `kept`."""
    return any(p in name for p in kept)


def over_budget(seconds, budget):
    """A row a budget leaves: over it, and every row under a zero budget
    (DECISIONS 132), the file's 0.00 rows included."""
    return budget <= 0 or seconds > budget


def lane_skips(timings, tests, budget=LANE_BUDGET_SECONDS, kept=PAPERWORK_GUARDS):
    """The timed tests a gate leaves to the next gate: every name over
    `budget` (`over_budget`), which no word filter in `tests` names, and
    which `kept` does not keep (`PAPERWORK_GUARDS`; a lane's `LANE_KEPT`).
    `timings` is the file's text: `seconds<TAB>name` rows, `#` comments."""
    return sorted(name for seconds, name in _timed_rows(timings)
                  if over_budget(seconds, budget) and not any(f in name for f in tests)
                  and not is_kept(name, kept))


def kept_line(paperwork, kept=PAPERWORK_GUARDS):
    """The `Lane suite:` line's clause for the paperwork the budget kept."""
    if not paperwork:
        return ''
    why = 'item 3373' + ('; the lane pins, DECISIONS 132' if kept != PAPERWORK_GUARDS else '')
    return f"; {len(paperwork)} over budget kept as paperwork ({', '.join(kept)}; {why})"


def lane_paperwork(timings, budget=LANE_BUDGET_SECONDS, kept=PAPERWORK_GUARDS):
    """The timed tests over `budget` a gate keeps because `kept` names them
    — the count its `Lane suite:` line says."""
    return sorted(name for seconds, name in _timed_rows(timings)
                  if over_budget(seconds, budget) and is_kept(name, kept))


def budget_clause(skips, budget):
    """The `Lane suite:` line's count of what the budget left, naming the
    budget it used (DECISIONS 132)."""
    if budget <= 0:
        return (f"{len(skips)} timed tests — every row the timings file names that no filter, paperwork "
                "guard or lane pin keeps, under the lane's own budget 0 s (DECISIONS 132) — left to the")
    return f"{len(skips)} timed tests over {budget:g} s alone left to the"


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
# Item 4009 moved the standalone sweeps to `crates/sim/tests/sweep/`; their
# four integration targets are `crates/sim/tests/sweep_*.rs`.
# The pass of 2026-11-12 (DECISIONS 127): and the two leaves of the data
# crate. `TIMINGS` times `rondata` alone; nothing depends on `client`, and
# the painter (`render::silhouette`) is read by the client alone. Both
# facts are asserted by `test_a_leaf_of_the_data_crate_does_not_widen_a_gate`.
NEVER_WIDENS = ('crates/sim/src/sweep/', 'crates/sim/tests/sweep', 'crates/rondata/src/census.rs',
                'crates/rondata/src/blind.rs', 'crates/client/', 'crates/render/src/silhouette.rs')


def lane_widens(paths, row_only=()):
    """The paths of a branch that can move the timed `diff::` pins, less
    the row-list edits `row_only_edits` found (a lane gate's; the booking
    gate passes none)."""
    return sorted(p for p in paths
                  if (p.startswith('crates/') or p in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml'))
                  and not p.startswith(NEVER_WIDENS) and p not in set(row_only))


# Loop 3608 (the fifty-fifth pass): eight of the fifty-fifth tranche's twenty
# landings stood an early-age pair up and touched no sim path, and every one
# of their lane gates ran the whole timed suite — release 16.0–17.7 minutes
# at four threads, the 788 tests over a second among them — for three
# row-list edits and a new file: the pair's `mod` line in `diff.rs`, its
# scored triple in `battery.rs`, its ledger row in `ledger.rs`, and the pair
# module itself. None can move another test's pin: a `mod` line declares, a
# new module is read by nothing that exists, the ledger's rows are the
# `ledger::` guards' (run in every lane gate whatever their timing) and a
# scored triple is read by the pair's own `is_no_other_lobby` (its `--tests`)
# and by the battery's held-out selection, which the booking gate keeps. So a
# lane gate reads the hunks: a path whose every changed line is one of these
# rows, with nothing removed, does not widen it, and a new `.rs` under
# `diff/` does not either. The booking gate's `--scope` keeps the path rule
# — a wrong triple is its to catch — and the policy receipt names the rows.
# The fifty-sixth pass (Loop 3608's sequel): every stand-up of the fifty-sixth
# tranche also added one `|| super::<pair>::<window>(),` line to the coverage
# driver and two literal rows to the test kit — the pair's `WIDENINGS` tuple
# and its `AiWord` — so the two paths widened every one of them and zero of
# twenty lane gates kept the fast suite (release 16–24 minutes in every
# journal; the dry run over the tranche's merges says eight of twenty would
# under this reading). The window line is read by the `coverage::` pins the
# lane's `--tests` names, the test-kit rows by the fast floors guards every
# lane gate runs; a removed line, a helper, a `use` or a field of another
# name is an edit.
ROW_LISTS = {
    'crates/rondata/src/diff.rs': re.compile(r'^(?:pub )?mod [a-z0-9_]+;$|^#\[cfg\(test\)\]$'),
    'crates/rondata/src/diff/battery.rs': re.compile(r'^\s*\(\d+, \d+, \d+\),$|^\s*//.*$'),
    'crates/rondata/src/ledger.rs': re.compile(r'^\s*\($|^\s*"[^"]*",$|^\s*\),$|^\s*//.*$'),
    'crates/rondata/src/diff/coverage.rs': re.compile(r'^\s*\|\| super::[a-z0-9_]+::[a-z0-9_]+\(\),$|^\s*//.*$'),
    'crates/rondata/src/diff/testkit.rs': re.compile(
        r'^\s*//.*$|^\s*$|^\s*\($|^\s*\),$|^\s*AiWord \{$|^\s*\},$'
        r'|^\s*"[^"]*",$|^\s*\("[^"]*", \d+\),$|^\s*\d[\d_]*,$|^\s*None,$'
        r'|^\s*Some\("[^"]*"\),$|^\s*Some\(crate::diff::[a-z0-9_]+::[A-Z0-9_]+\),$'
        r'|^\s*crate::diff::[a-z0-9_]+::[A-Z0-9_]+,$'
        r'|^\s*(?:line|map|named|window|endpoint): (?:"[^"]*"|None|Some\("[^"]*"\)),$'
        r'|^\s*word: crate::diff::[a-z0-9_]+::[A-Z0-9_]+,$|^\s*length: \d[\d_]*,$'),
}
NEW_MODULES = 'crates/rondata/src/diff/'


def row_only_edits(base, run=subprocess.run):
    """The branch's paths that cannot move a pin by their content: a new
    module under `diff/`, and a row list (`ROW_LISTS`) whose hunks only add
    rows. Empty when `base` is not a ref this tree knows."""
    try:
        fork = run(['git', 'merge-base', base, 'HEAD'], capture_output=True, text=True,
                   cwd=ROOT, check=True).stdout.strip()
        status = run(['git', 'diff', '--name-status', fork, 'HEAD'], capture_output=True, text=True,
                     cwd=ROOT, check=True).stdout
    except subprocess.CalledProcessError:
        return set()
    found = set()
    changed = {}
    for line in status.splitlines():
        parts = line.split('\t')
        if len(parts) >= 2:
            changed[parts[-1]] = parts[0][:1]
    for path, kind in changed.items():
        if kind == 'A' and path.startswith(NEW_MODULES) and path.endswith('.rs'):
            found.add(path)
    for path, row in ROW_LISTS.items():
        if path not in changed:
            continue
        try:
            diff = run(['git', 'diff', '-U0', fork, 'HEAD', '--', path], capture_output=True, text=True,
                       cwd=ROOT, check=True).stdout
        except subprocess.CalledProcessError:
            continue
        lines = [l for l in diff.splitlines()
                 if (l.startswith('+') or l.startswith('-')) and not l.startswith(('+++', '---'))]
        if lines and all(l.startswith('+') and row.match(l[1:]) for l in lines):
            found.add(path)
    return found


# Loop 3653 (the fifty-seventh pass): every stand-up's lane gate of the
# fifty-seventh tranche fired Loop 3608's rule — 758 timed tests left to the
# booking gate, each gate's `gate-policy.json` says — and still ran 16–19
# release minutes, because `TIMINGS` times the data crate alone and the sim
# crate's five test binaries ran whole: in the pass's replay of 3626's gate
# at four threads, the four emulator sweep targets 583 s and the lib 98 s
# against the data crate's kept set 275 s and the build 84 s (release 17.4).
# A branch that touches no path under `crates/sim/` cannot change those
# binaries, so its lane gate leaves the sim crate to the booking gate, which
# runs it on every merge; the sim paperwork guards (`docs_guard`, `no_float`)
# run in the guard step (`guard.sh --built`) on every lane as before. A sweep
# lane touches `sweep/` and keeps them; a sim edit widened already.
# Widened or not (the fifty-eighth pass, Loop 3653's sequel): the cut was
# first tied to a lane keeping the fast suite, and two of the fifty-eighth
# tranche's nine stand-ups widened on one data-crate file each (3629 a
# comment moved in `battery.rs`, 3665 a debug print in `harness.rs`) and ran
# the five binaries for eleven minutes apiece — release 18.0 and 18.3
# against the cut stand-ups' 5.6–6.9. The sim crate depends on nothing in
# the workspace, so a data-crate edit cannot move them; what can besides a
# path under the crate is the workspace manifest, the lock, the toolchain
# pin and cargo's config, and those keep the crate.
# And `tools/emu/` (Loop 3777, the sixty-third pass): the emulator scripts,
# their shared modules and `sweep_obj.tsv` decide the sweeps' rows, and the
# emu cache keys on them, so a lane that edits only those moves the four
# sweep binaries' verdicts as surely as a crate edit; it keeps the crate.
SIM_CRATE = 'crates/sim/'
SIM_BUILD_INPUTS = (SIM_CRATE, 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/',
                    'tools/emu/')


def sim_crate_left(touched, *, lane, whole):
    """Whether a lane gate leaves the sim crate's test binaries to the
    booking gate: a lane, not `--whole`, whose branch (`touched`, None when
    the base is not a ref) touches nothing the binaries are built from
    (`SIM_BUILD_INPUTS`) — widened or not."""
    return bool(lane and not whole and touched is not None
                and not any(p.startswith(SIM_BUILD_INPUTS) for p in touched))


# **The client crate's targets, the same way** (the pass of 2026-11-13,
# second; DECISIONS 132 §6). The client's clock test
# (`crates/client/tests/clock.rs`) is a wall-clock ceiling test: it ran
# 218–362 s beside the rondata lib on lanes 4200 and 4203, and it is behind
# 2 of the 7 red gate logs on disk (4171, 4179), both under load — p99 over
# its ceiling, green on the next run. TIMINGS times the data crate alone, so
# no budget ever left it, and with a lane's own budget at zero the data
# crate's library no longer hides it: its side lane (the clock, the client
# lib's 43–75 s) becomes the release step's wall. A branch that touches no
# path under `crates/client/` or `crates/render/` (the painter the client
# draws with) — nor the workspace manifest, the lock, the toolchain pin or
# cargo's config — leaves the client's targets to the booking gate, which
# runs them on the merged tree; lane mode only, never under `--whole`, and
# widened or not, as the sim crate's cut. The render and fixed crates'
# own targets keep running: 0–239 s and 0–52 s on those lanes' report dirs,
# and a render lane's own.
CLIENT_PACKAGE = 'client'
CLIENT_BUILD_INPUTS = ('crates/client/', 'crates/render/', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
                       '.cargo/')


def client_crate_left(touched, *, lane, whole):
    """Whether a lane gate leaves the client crate's test targets (its lib
    and `clock`) to the booking gate: a lane, not `--whole`, whose branch
    (`touched`, None when the base is not a ref) touches nothing under
    `CLIENT_BUILD_INPUTS` — widened or not (DECISIONS 132 §6)."""
    return bool(lane and not whole and touched is not None
                and not any(p.startswith(CLIENT_BUILD_INPUTS) for p in touched))


# Loop 3823 (the sixty-fifth pass): `map_lakes` and `terrain_groups`, tests
# in `crates/sim` alone, passed 3819's brief — `brief.py` lists the whole
# workspace's tests (Loop 3818) — and its first lane gate exited 2 on them
# before its guard step: the branch touched no sim path, so the crate was
# left to the booking gate and the filters matched no test that ran. Seven
# of the tranche's twenty briefs named such a filter for a lane that
# touched no sim path. The lane drops it and names it; the exit 2 stays for
# a filter that matches nothing anywhere. The test list is `brief.py`'s.
def crate_tests():
    """`{crate: {test name: ignored?}}` over the workspace, `brief.py`'s lister."""
    import brief
    return brief.crate_tests()


def sim_only_filters(tests, by_crate):
    """The filters whose every live match is in the sim crate (`brief.sim_only`)."""
    import brief
    return brief.sim_only(tests, by_crate)


def client_only_filters(tests, by_crate):
    """The filters whose every live match is in the client crate (`brief.only_in`)."""
    import brief
    return brief.only_in(tests, by_crate, CLIENT_PACKAGE)


def dropped_line(t, crate='crates/sim', why='Loop 3823'):
    return (f'Lane filter dropped: {t} — every match is in {crate}, which this lane left to the '
            f'booking gate ({why})')


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


# The commander's paperwork: paths a lane never edits or whose concurrent
# edits git merges clean, so an overlap on them alone is no reason to gate
# again. Loop 3401 (the forty-seventh pass): 3393's second to fourth lane
# gates ended on `Base moved:` and each forced a `ccc update` and another
# full lane gate (~55 minutes), yet every update merged clean — the third's
# and fourth's overlap was docs/PARKED.md alone, whose tail the commander's
# bookings append to while a lane renames its own reserved slot. QUEUE.md
# and JOURNAL.md are the commander's alone; RUNS.md merges by union
# (.gitattributes).
COMMANDER_PAPERWORK = ('docs/QUEUE.md', 'docs/PARKED.md', 'docs/JOURNAL.md', 'docs/RUNS.md')


def split_paperwork(overlap, row_lists=()):
    """(blocking, paperwork): an overlap parted into the paths that refuse a
    lane's done line and the commander's paperwork, which does not (Loop
    3401) — nor do the row lists `row_list_overlap` found (Loop 3694)."""
    overlap = list(overlap or [])
    paperwork = [p for p in overlap if p in COMMANDER_PAPERWORK]
    forgiven = set(paperwork) | set(row_lists or ())
    return [p for p in overlap if p not in forgiven], paperwork


# Loop 3694 (the fifty-ninth pass's finding, built by the sixtieth): the
# fifty-ninth's residue lanes ran their lane gate three to five times each
# on `Base moved:` — 908 of the tranche's 1,145 waiting minutes on the gate
# class while the door had room — because every landing touches the same
# paths: the rubric pin in `census.rs`, the item's section in `docs/AI.md`,
# the test-kit rows, the coverage driver's window line, a capture's row.
# The sixtieth's two lanes still took a `--keep-conflicts` update and a
# second gate on exactly those (3704 three gates, 3706 four, 3698 and 3700
# two). A path whose hunks on the base's side are rows a merge keeps both
# of cannot make the merged tree red where both sides were green: a row
# list's added rows (`ROW_LISTS`, the lane cut's own regexes), a document
# (the paperwork guards read it, and `guard.sh` runs after the update), a
# capture's row, and the census pin with its comment (re-pinned after the
# update, as every landing does). So the lane updates with
# `--keep-conflicts`, keeps both sides, re-pins, runs the guards and lands
# on this verdict; the scoped booking gate is the check on every merge. A
# removed row, an edit, or any other path still refuses as before.
ROW_LIST_CAPTURES = 'tools/gamelog/captures.txt'
ROW_LIST_CENSUS = 'crates/rondata/src/census.rs'
CENSUS_PIN_LINES = re.compile(r'^\s*///.*$|^pub const RUBRIC_UNIT_TYPES_SEEN: usize = \d+;$|^\s*$')
ANY_ROW = re.compile(r'^.*$')


def row_list_overlap(base, overlap, run=subprocess.run):
    """The overlap paths whose hunks on `base`'s side, since this branch
    left it, are rows a merge keeps both of (Loop 3694): a document under
    `docs/`, a `ROW_LISTS` file or the captures file whose hunks only add
    rows, and the census file whose changed lines are the rubric pin and
    its comment. Empty when `base` is not a ref this tree knows."""
    overlap = [p for p in (overlap or []) if p not in COMMANDER_PAPERWORK]
    if not overlap:
        return []
    try:
        fork = run(['git', 'merge-base', base, 'HEAD'], capture_output=True, text=True,
                   cwd=ROOT, check=True).stdout.strip()
    except subprocess.CalledProcessError:
        return []
    found = []
    for path in overlap:
        if path.startswith('docs/') and path.endswith('.md'):
            found.append(path)
            continue
        row = ROW_LISTS.get(path)
        removals_ok = False
        if path == ROW_LIST_CAPTURES:
            row = ANY_ROW
        elif path == ROW_LIST_CENSUS:
            row, removals_ok = CENSUS_PIN_LINES, True
        if row is None:
            continue
        try:
            diff = run(['git', 'diff', '-U0', fork, base, '--', path], capture_output=True, text=True,
                       cwd=ROOT, check=True).stdout
        except subprocess.CalledProcessError:
            continue
        lines = [l for l in diff.splitlines()
                 if (l.startswith('+') or l.startswith('-')) and not l.startswith(('+++', '---'))]
        if lines and all((l.startswith('+') or removals_ok) and row.match(l[1:]) for l in lines):
            found.append(path)
    return found


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


def base_line(base, commits, overlap, row_lists=()):
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
    blocking, paperwork = split_paperwork(overlap, row_lists)
    row_lists = [p for p in (row_lists or ()) if p in set(overlap or [])]
    if blocking:
        shown = ', '.join(blocking[:4]) + (', …' if len(blocking) > 4 else '')
        forgiven = ([f"the commander's paperwork ({', '.join(paperwork)})"] if paperwork else []) + \
                   ([f"the row lists on the base's side ({', '.join(row_lists)})"] if row_lists else [])
        return line + (f"{len(blocking)} path(s) both sides touched ({shown}): take `ccc update <your ref>` "
                       "(`--keep-conflicts` on a conflict), gate again, and only then your done line — a "
                       "landing announced here backs out at `ccc merge` (Loop 3341)"
                       + (f"; {' and '.join(forgiven)} are no reason of their own (Loops 3401, 3694)"
                          if forgiven else ''))
    if row_lists:
        return line + (f"the only path(s) both sides touched are row lists on the base's side "
                       f"({', '.join(row_lists)})"
                       + (f" and the commander's paperwork ({', '.join(paperwork)})" if paperwork else '')
                       + ": take `ccc update <your ref> --keep-conflicts` at this gate's exit, keep both "
                       "sides' rows, re-pin `census::RUBRIC_UNIT_TYPES_SEEN` where it conflicted, run "
                       "`tools/guard.sh`, and land on this verdict saying which base the gate ran on — the "
                       "booking gate is the check on every merge (Loop 3694)")
    if paperwork:
        return line + (f"the only path(s) both sides touched are the commander's paperwork "
                       f"({', '.join(paperwork)}): take `ccc update <your ref>` at this gate's exit and "
                       "land on this verdict if the update is clean, saying which base the gate ran on "
                       "(Loop 3401)")
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


# Loop 3761: a lane gate wholly green printed no verdict, and docs_guard
# asks every journal to quote one.
LANE_GREEN = 'Lane verdict: green, nothing red'


# Loop 4030 (the seventy-fifth pass): 3995 wired a port-backed function and
# its lane gate ran the census pin — 0.72 s alone, under the lane budget, so
# every lane runs it — and forgave its red as the commander's line (Loop
# 2367), and the verdict its journal quoted named the test and never the
# count. The pin stood at 588 against a measured 589 until the booking gate
# went red on it alone, the seventy-fifth tranche's one red gate and the
# seventy-third's. The row was filed as "the lane's filters never reach
# `census::`"; they did, and a filter would have changed nothing. A forgiven
# census red now carries the measured triple against the pin's, read off
# the assertion's own panic, so the quoted verdict hands the commander the
# re-pin.
CENSUS_PIN = 'the_census_s_simulation_share_is_pinned'
CENSUS_MOVED = re.compile(r'the census moved:.*\n\s*left: \((\d+), (\d+), (\d+)\)\n\s*right: '
                          r'\((\d+), (\d+), (\d+)\)')


def census_moved(text):
    """((backed, port, functions) measured, the pin's) off the census
    pin's panic in `text`, or None when it is not there (Loop 4030)."""
    found = CENSUS_MOVED.search(text)
    if not found:
        return None
    numbers = tuple(int(n) for n in found.groups())
    return numbers[:3], numbers[3:]


def census_logs(release_log, report_dir):
    """The texts a census panic can be in: the release log (a combined run
    prints every failure's output there) and each failed test's own log an
    isolated run's `measurements.jsonl` names."""
    texts = [Path(release_log).read_text(errors='replace')] if Path(release_log).is_file() else []
    for rows in Path(report_dir).rglob('measurements.jsonl'):
        for line in rows.read_text().splitlines():
            try:
                row = json.loads(line)
            except ValueError:
                continue
            if not row.get('success') and str(row.get('name', '')).endswith(CENSUS_PIN) \
                    and Path(str(row.get('log', ''))).is_file():
                texts.append(Path(row['log']).read_text(errors='replace'))
    return texts


def census_clause(census):
    """The forgiven verdict's census clause (Loop 4030)."""
    if census is None:
        return ('; the census moved and its measure is not in the log: run `python3 tools/census.py '
                '--layers --json` and say (backed, port, functions) in your done line — the commander '
                're-pins `census::SIMULATION_*` and the `Census:` line at the merge (Loop 4030)')
    (backed, port, functions), (pin_backed, pin_port, pin_functions) = census
    moves = [f'{what} {was} → {now}' for what, was, now in
             (('backed', pin_backed, backed), ('port', pin_port, port), ('functions', pin_functions, functions))
             if was != now]
    return (f'; the census moved ({", ".join(moves)}): measured (backed, port, functions) '
            f'({backed}, {port}, {functions}) against the pin\'s ({pin_backed}, {pin_port}, {pin_functions}) '
            '— say it in your done line; the commander re-pins `census::SIMULATION_*` and the `Census:` '
            'line at the merge (Loop 4030)')


def lane_verdict(failed_tests, census=None):
    """(forgiven, line): whether every red test is one of the commander's
    lines, and the sentence the gate prints either way. `census` is
    `census_moved`'s reading, for a red census pin (Loop 4030)."""
    own = [name for name in failed_tests
           if name.rsplit('::', 1)[-1] not in COMMANDERS_LINES]
    if failed_tests and not own:
        remedy = ('; quote this gate\'s `Lane verdict:` and `Gate minutes:` lines in your journal '
                  'before your done line — the booking gate holds the quote (3277)'
                  if any(name.endswith(JOURNAL_QUOTE_GUARD) for name in failed_tests) else '')
        remedy += (census_clause(census) if any(name.endswith(CENSUS_PIN) for name in failed_tests) else '')
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
# The gate's steps (Loop 4070): pins, offline, clippy, fmt, survey, guard,
# release — the release command's index in `gate()`'s list.
RELEASE_STEP = 6


def memory_cap_gib(test_threads):
    return 20 if test_threads <= 4 else 40


# Loop 3489 (the fifty-first pass): a lane whose item moves no test — a
# retirement, a tooling item — had no lane gate that ran as briefed: `--lane`
# without `--tests` is refused, so it ran the unflagged booking gate, which
# forgives nothing. `--lane --whole` runs every test, the timed ones
# included, with the lane's forgiveness of the commander's lines and its
# `Lane verdict:`; the caches' verifiers stay the booking gate's.
def check_whole(*, lane, tests, whole, no_widen):
    """The `--whole` refusals, read before anything runs or prints."""
    if not whole:
        return
    if not lane:
        raise ValueError('--whole is a lane gate\'s (--lane --whole): the booking gate is already '
                         'the whole suite (Loop 3489)')
    if tests:
        raise ValueError('--whole runs every test, so it names no --tests: a lane with a word '
                         'gates with --tests (Loop 3489)')
    if no_widen:
        raise ValueError('--whole runs every test, the timed ones included; --no-widen would '
                         'skip them (Loop 3489)')


def gate(install, *, report_dir=None, require_fixtures=False, test_threads=2, run=run_logged,
         lane=False, tests=None, item=None, base='worktree-replan-pdb', scope=None, isolated=None,
         no_widen=False, whole=False, git=subprocess.run, whole_lock=None):
    # `git` runs the gate's own reads of the tree (`branch_paths`,
    # `base_moved`); the offline cases hand it a fixture so none reads the
    # tree the gate runs in (item 3366).
    # Parked 1713: two lanes `pkill -f release_gate.py`'d every gate on the
    # box to stop their own; a gate names its pid first so it can be killed
    # by it.
    check_whole(lane=lane, tests=tests, whole=whole, no_widen=no_widen)
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
    if lane and not tests and not whole:
        raise ValueError("a lane gate names its word's tests: --tests FILTER [FILTER ...] "
                         '(the widening, the pinned walk); a lane whose item moves no test '
                         'gates with --lane --whole (Loop 3489)')
    if tests and not lane:
        raise ValueError('--tests is a lane gate\'s; the booking gate runs the whole suite')
    if no_widen and not lane:
        raise ValueError('--no-widen is a lane gate\'s: the booking gate always widens on a path that can move a pin')
    if any(not f.strip() for f in tests):
        raise ValueError('an empty --tests filter would keep every slow test')
    if lane:
        # Item 3366: 3360's refusal was one line at the log's foot, under
        # seven offline cases that had failed for reading the same moved
        # tree, and it read as a broken tool. The base is read before the
        # first step too; when both sides touched a path the gate says so
        # first and runs nothing. It is read again after the steps (below),
        # for a base that moves while the gate runs.
        # Loop 3401: an overlap on the commander's paperwork alone does not
        # refuse; the line after the steps names it.
        commits, overlap = base_moved(base, run=git)
        # Loop 3694: an overlap whose hunks on the base's side are rows a
        # merge keeps both of does not refuse either; the receipt names
        # the classes so the next pass can count what the rule forgave.
        row_lists = row_list_overlap(base, overlap, run=git)
        blocking, paperwork_overlap = split_paperwork(overlap, row_lists)
        base_overlap = {'blocking': blocking, 'row_lists': row_lists, 'paperwork': paperwork_overlap}
        if blocking:
            print(base_line(base, commits, overlap, row_lists), flush=True)
            raise ValueError(f"the base moved under this branch and both sides touched "
                             f"{', '.join(blocking)}: refused before the offline step, nothing ran; "
                             "take `ccc update <your ref>` and gate again (Loop 3341, item 3366)")
    skips, widened, touched, paperwork, row_only = [], [], None, [], []
    # The budget the release step used: None where it skipped nothing
    # (`--whole`, a widened lane or scope, the whole booking gate).
    budget, kept = None, PAPERWORK_GUARDS
    if lane or scope is not None:
        touched = branch_paths(base if lane else scope, run=git)
        # Loop 3608: a lane's row-list edits and new `diff/` modules do not
        # widen it; the booking gate's scope keeps the path rule.
        row_only = sorted(row_only_edits(base, run=git)) if lane else []
        widened = lane_widens(touched or [], row_only=row_only)
        # DECISIONS 83 (the forty-fourth steer): every lane of one tranche
        # touched a sim path and every lane gate ran the whole timed suite
        # at four threads (6–26 minutes, median 18.9) while fifteen of the
        # twenty moved no pin. The AI lane's brief widens; a sweep's or a
        # chapter's says `--no-widen`, and the booking gate holds the pins.
        if (not widened or no_widen) and not whole:
            # DECISIONS 132: a lane's own budget is zero, the scope's a second.
            budget, kept = ((LANE_OWN_BUDGET_SECONDS, LANE_KEPT) if lane
                            else (LANE_BUDGET_SECONDS, PAPERWORK_GUARDS))
            timings = TIMINGS.read_text()
            skips = lane_skips(timings, tests, budget, kept)
            paperwork = lane_paperwork(timings, budget, kept)
    sim_left = sim_crate_left(touched, lane=lane, whole=whole)
    sim_clause = ("; the sim crate's test binaries left to the booking gate — no path under "
                  f"{SIM_CRATE} touched (Loop 3653)" if sim_left else '')
    client_left = client_crate_left(touched, lane=lane, whole=whole)
    client_clause = ("; the client crate's test targets left to the booking gate — no path under "
                     "crates/client/ or crates/render/ touched, and its clock test is a wall-clock "
                     "ceiling a loaded host fails (DECISIONS 132 §6)" if client_left else '')
    left_packages = (['sim'] if sim_left else []) + ([CLIENT_PACKAGE] if client_left else [])
    install = Path(install).resolve()
    if not (install / 'Data/rules.xml').is_file():
        raise ValueError(f'not an install: missing {install / "Data/rules.xml"}')
    by_crate = crate_tests() if (sim_left or client_left) and tests else {}
    dropped = sim_only_filters(tests, by_crate) if sim_left and tests else []
    client_dropped = client_only_filters(tests, by_crate) if client_left and tests else []
    tests = [t for t in tests if t not in dropped and t not in client_dropped]
    for t in dropped:
        print(dropped_line(t), flush=True)
    for t in client_dropped:
        print(dropped_line(t, 'crates/client', 'DECISIONS 132 §6'), flush=True)
    dropped = dropped + client_dropped
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
    booking_whole = scope is None and not lane
    if booking_whole:
        env['RON_VERIFY_ORACLE'] = '1'
        env['RON_VERIFY_SETUP_OBSERVATIONS'] = '1'
        env['RON_VERIFY_EMU'] = '1'
    policy ={'schema': 1, 'release_test_threads': test_threads,
              'test_allocator': allocator_policy(env),
              'verify_caches': booking_whole,
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
                           'budget_seconds': budget, 'timings': str(TIMINGS.relative_to(ROOT))}
    if lane:
        policy['lane'] = {'tests': tests, 'timings': str(TIMINGS.relative_to(ROOT)), 'no_widen': bool(no_widen),
                          'whole': bool(whole),
                          'budget_seconds': budget, 'skipped': len(skips),
                          'paperwork_kept': len(paperwork), 'base': base, 'widened': widened,
                          'row_only': row_only, 'dropped': dropped,
                          'base_overlap': base_overlap,
                          'sim_crate': 'left to the booking gate' if sim_left else 'run',
                          'client_crate': 'left to the booking gate' if client_left else 'run'}
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
    # **The pins that read the tree go first** (Loop 4070): 4063's inline
    # test tail was red on the structure pin at minute 49.5 of a whole gate,
    # a comparison that takes a second. `pins` is that comparison with
    # nothing built (`tools/shape.py --check`, never forgiven, a worker's
    # half), and the guard step — the blind list, the paperwork guards, the
    # structure pin itself — runs before the release step, where it builds
    # the release binaries the release step then runs. A pin that only the
    # release step held was a pin only minute 49 held. The release step
    # still runs every one of them.
    commands = [
        [sys.executable, 'tools/shape.py', '--check'],
        [sys.executable, 'tools/offline_tests.py'],
        ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'],
        ['cargo', 'fmt', '--check'],
        ['cargo', 'run', '-p', 'rondata', '--', str(install)],
        # The reflex script on the release profile (the thirty-fifth steer: a
        # debug build a gate was three minutes, forty-four gates a tranche);
        # before the release step since Loop 4070, so its build is the
        # release step's and its reds come before the suite's fifty minutes.
        ['zsh', 'tools/guard.sh', '--built'],
        ['zsh', 'tools/memcap.sh', str(min(32, memory_cap_gib(test_threads))), 'cargo', 'test', '--release',
         '--no-fail-fast']
        + (['--workspace'] + [a for p in left_packages for a in ('--exclude', p)] if left_packages else [])
        + ['--', f'--test-threads={test_threads}']
        + (['--exact'] + [arg for name in skips for arg in ('--skip', name)] if (lane or scope is not None) else []),
    ]
    if isolated:
        commands[RELEASE_STEP] = ([sys.executable, 'tools/isolated_tests.py', '--release',
                        '--output', str(report_dir / 'isolated'), '--test-threads', str(test_threads)]
                       + [a for p in left_packages for a in ('--exclude-package', p)]
                       + [arg for name in skips for arg in ('--skip', name)])
    if scope is not None:
        print((f"Booking gate scoped to {scope}: " +
               (f"every timed test — a merge touches {len(widened)} path(s) that can move the pins ({widened[0]}"
                f"{', …' if len(widened) > 1 else ''})"
                if widened else
                f"{len(skips)} timed tests left to the next whole gate" +
                (f"; {scope} is not a ref here, so nothing widened" if touched is None else ''))), flush=True)
    if lane and whole:
        print("Lane suite: every test, the timed ones included (--whole, a lane whose item moves no "
              "test; Loop 3489); the commander's lines forgiven as on any lane", flush=True)
    elif lane and widened and no_widen:
        print(f"Lane suite: {budget_clause(skips, budget)} "
              f"booking gate by the brief's --no-widen, though the branch touches {len(widened)} path(s) "
              f"that can move a pin ({widened[0]}{', …' if len(widened) > 1 else ''}); the booking gate "
              "holds the pins (DECISIONS 83)" + kept_line(paperwork, kept) + client_clause, flush=True)
    elif lane and widened:
        print(f"Lane suite: every timed test — the branch touches {len(widened)} path(s) that can move "
              f"the diff:: pins ({widened[0]}{', …' if len(widened) > 1 else ''}; Loop 2415); "
              f"the word's: {', '.join(tests)}" + sim_clause + client_clause, flush=True)
    elif lane:
        print(f"Lane suite: {budget_clause(skips, budget)} "
              f"booking gate ({TIMINGS.relative_to(ROOT)}); the word's: {', '.join(tests)}"
              + (f"; base {base} is not a ref here, so nothing widened" if touched is None else '')
              + kept_line(paperwork, kept) + sim_clause + client_clause, flush=True)
    reached = []
    seconds = []
    verdicts = []
    held = whole_lane_lock(lane=lane, whole=whole, item=item, path=whole_lock)
    load_at_start = host_load()
    top_at_start = host_top()
    try:
        _run_steps(commands, run=run, env=env, audit_dir=audit_dir, report_dir=report_dir,
                   require_fixtures=require_fixtures, reached=reached, lane=lane, word=tests,
                   seconds=seconds, verdicts=verdicts)
        if lane:
            # Loop 3341: the base is read after the steps, so the line is
            # the freshest a lane can have before its done line.
            commits, overlap = base_moved(base, run=git)
            row_lists = row_list_overlap(base, overlap, run=git)
            print(base_line(base, commits, overlap, row_lists), flush=True)
            blocking, _ = split_paperwork(overlap, row_lists)
            if blocking:
                raise ValueError(f"the base moved under this branch and both sides touched "
                                 f"{', '.join(blocking)}: take `ccc update <your ref>` and gate again "
                                 "before your done line (Loop 3341)")
            if not verdicts:
                print(LANE_GREEN, flush=True)
    finally:
        print(minutes_line([step_name(c) for c in commands], seconds), flush=True)
        print(steps_line(commands, reached), flush=True)
        print(admission_line(admission_summary(report_dir)), flush=True)
        print(load_line(load_at_start, host_load(), os.cpu_count() or 0), flush=True)
        print(top_line(host_top(), top_at_start), flush=True)
        if held is not None:
            os.close(held)
    return report_dir


def whole_lane_lock(*, lane, whole, item=None, path=None, off=None, clock=time.monotonic):
    """The tranche of 2026-11-12: two lanes ran `--lane --whole` gates beside
    a builder's whole diff:: runs and the booking gate, the budget's 20 slots
    interleaved them test by test (admission 5,466 s summed on one booking
    gate) and the host's load reached 50.9 on 16. A whole lane gate takes an
    exclusive flock on `WHOLE_LANE_LOCK` before its first step and holds the
    fd to its exit; one that finds it held names the holder and blocks. A
    booking gate and a worded lane gate take none, nor does the admission
    kill switch. The file is never unlinked (`resource_budget.py`'s
    discipline: an unlinked lock is two locks). Returns the held fd, or None."""
    if not (lane and whole):
        return None
    if admission_off() if off is None else off:
        return None
    path = Path(path or WHOLE_LANE_LOCK)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(path, os.O_RDWR | os.O_CREAT, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        holder = os.pread(fd, 512, 0).decode(errors='replace').strip() or 'a holder that wrote no name'
        print(f"Whole lane: waiting on {holder}", flush=True)
        started = clock()
        fcntl.flock(fd, fcntl.LOCK_EX)
        print(f"Whole lane: waited {clock() - started:.0f} s", flush=True)
    os.ftruncate(fd, 0)
    os.pwrite(fd, f"pid {os.getpid()} item {item if item is not None else 'unnamed'} started "
                  f"{time.strftime('%Y-%m-%dT%H:%M:%S%z')}\n".encode(), 0)
    return fd


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


def host_top():
    """The five commands with the most summed CPU, or None where `ps`
    cannot be read."""
    try:
        out = subprocess.run(['ps', '-Ao', 'pcpu=,comm='], capture_output=True, text=True, timeout=20)
    except (OSError, subprocess.SubprocessError):
        return None
    return host_top_summary(out.stdout) if out.returncode == 0 else None


def host_top_summary(ps_text, n=5):
    """The pass of 2026-11-12: two timing tests went red at a load of 46
    to 51 on 16 cores, and the receipt's `Host load:` said how much and
    nothing about what. `ps -Ao pcpu=,comm=`'s text, summed by the
    command's basename (a cargo test run is many `rustc`s, a gate many
    test binaries), the `n` largest first, each to one decimal."""
    sums = {}
    for line in ps_text.splitlines():
        parts = line.strip().split(None, 1)
        if len(parts) != 2:
            continue
        try:
            cpu = float(parts[0])
        except ValueError:
            continue
        name = os.path.basename(parts[1].rstrip('/')) or parts[1]
        sums[name] = sums.get(name, 0.0) + cpu
    ranked = sorted(sums.items(), key=lambda kv: (-kv[1], kv[0]))[:n]
    return [(name, round(cpu, 1)) for name, cpu in ranked]


def top_line(end, start):
    """`Host top: cargo 812.3, rustc 400.1, … at end; … at start` — the
    receipt's second contention line, beside `Host load:`, whose shape
    `tools/gates.py` parses and which this line leaves alone."""
    def part(top):
        return 'n/a' if not top else ', '.join(f'{name} {cpu:.1f}' for name, cpu in top)
    return f"Host top: {part(end)} at end; {part(start)} at start"


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
    if any(str(a).endswith('shape.py') for a in command):
        return 'pins'
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
               lane=False, word=(), seconds=None, verdicts=None):
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
            census = None
            if any(name.endswith(CENSUS_PIN) for name in tests['failed_tests']):
                census = next(filter(None, map(census_moved, census_logs(extra['log'], report_dir))), None)
            forgiven, line = lane_verdict(tests['failed_tests'], census=census)
            print(line, flush=True)
            if verdicts is not None:
                verdicts.append(line)
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
    names whose age (`age_hours(path)`, None when unknown) is past the limit,
    each as `(path, age, head)`.
    The harness cuts a subagent's worktree under the main checkout's
    `.claude/worktrees/`, every lane's in one directory; a sweep's go with
    its landing (the frame's row), and three of the thirty-fifth tranche's
    stood a day, two with unmerged tips, named by nothing."""
    out = []
    for block in porcelain.strip().split('\n\n'):
        lines = block.strip().split('\n')
        if not lines[0].startswith('worktree '):
            continue
        path = lines[0][len('worktree '):].strip()
        if '/.claude/worktrees/agent-' not in path:
            continue
        head = next((l[len('HEAD '):].strip() for l in lines[1:] if l.startswith('HEAD ')), '')
        age = age_hours(path)
        if age is not None and age > limit:
            out.append((path, age, head))
    return out


def refuse_stale_agent_worktrees(stale, *, lane, merged):
    """The booking gate refuses a stale tree whose tip is not in the gated
    commit's history (`merged(head)` false): that is work nothing names,
    which is what Loop 2724 was written for. A merged one is clutter and is
    printed, never refused (Loop 4090): since the chain is built by the
    driver's in-session builders every step leaves one, the driver's reap of
    it is refused by the classifier and by the harness's own lock, and a
    gate that stops on it stops on something no session can clear. A lane
    gate never looks, because the directory is every lane's and a red would
    land on the wrong lane."""
    if lane or not stale:
        return
    def names(rows):
        return ', '.join(f'{Path(p).name} ({h:.1f} h)' for p, h, _ in rows)
    done = [row for row in stale if merged(row[2])]
    open_ = [row for row in stale if not merged(row[2])]
    if done:
        print(f'Agent worktrees: {len(done)} merged agent-* tree(s) stand past {STALE_AGENT_HOURS} h: '
              f'{names(done)} — clutter, reaped at a pass (`python3 tools/reap_agents.py --yes`)')
    if open_:
        raise ValueError(f'{len(open_)} stale agent-* worktree(s) under the main checkout, past '
                         f'{STALE_AGENT_HOURS} h, with a tip this commit does not hold: {names(open_)} — '
                         f'a sweep\'s go with its landing (Loop 2724); merge or read each, then '
                         f'`git worktree remove` it')


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


def agent_tip_is_merged(head):
    """Whether `head` is in the history of the commit the gate runs on."""
    return bool(head) and subprocess.run(['git', 'merge-base', '--is-ancestor', head, 'HEAD'], cwd=ROOT,
                                         capture_output=True).returncode == 0


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
    parser.add_argument('--whole', action='store_true',
                        help="with --lane, in place of --tests: every test, the timed ones included, with "
                             "the lane's forgiveness and verdict — a lane whose item moves no test (Loop 3489)")
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
        check_whole(lane=args.lane, tests=args.tests, whole=args.whole, no_widen=args.no_widen)
        refuse_stale_agent_worktrees(stale_agent_worktrees_now(), lane=args.lane, merged=agent_tip_is_merged)
        gate(args.install, report_dir=args.report_dir, require_fixtures=args.require_fixtures,
             test_threads=args.test_threads, lane=args.lane, tests=args.tests, item=args.item,
             base=args.base, scope=args.scope, isolated=args.isolated, no_widen=args.no_widen,
             whole=args.whole)
    except ValueError as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    main()
