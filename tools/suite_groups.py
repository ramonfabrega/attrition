#!/usr/bin/env python3
"""suite_groups.py — the release suite's groups, from a gate's own fixture audit.

    python3 tools/suite_groups.py --from <report dir> --out docs/audit/2026-11-13-suite-groups.json
    python3 tools/suite_groups.py --peaks <report dir> --out docs/audit/2026-11-13-suite-groups.json
    python3 tools/suite_groups.py --audit <report dir A> <report dir B>
    python3 tools/suite_groups.py --stand-ups <report dir>

Item 4216 (DECISIONS 132 §11): the isolated runner (`tools/isolated_tests.py`)
runs a group's tests in one process, and this writes the groups. Nothing here
is a guess about a test's name: a gate's fixture audit says which test asked
for which kept file, and

  * the tests of **one game** are those joined by a draw-site trace they both
    ask for (`rontrace-*.log`: the long capture's trace is the walk's base,
    and a window test asks for its window's trace beside it), among the
    tests that ask for at most `GAME_FIXTURES` files and `GAME_TRACES`
    traces (a test that reads the traces of many games would join them all:
    one endpoint test asks for eleven, and with it 157 tests of six modules
    were one group);
  * the tests of **one corpus sweep** are a module's tests that each ask for
    more than `SWEEP_FIXTURES` files — map generation's, which read every
    kept preamble through one in-process cache.

A test the audit never saw (a new one) is in no group and runs alone, as
every test did; a stale file costs seconds, never a verdict. A test the
runner's profile runs wider than one slot is left out.

`--peaks` reads a grouped run's `measurements.jsonl` and writes each
group's peak resident set into the groups file's `memory_mib`: the runner
reserves from it (`isolated_tests.group_memory`), and a group no run has
measured reserves as an unknown test does. A group whose process was
stopped has no peak and keeps none. `--from` keeps the peaks of the file
it rewrites for every group whose members did not change.

`--audit` compares two audits test for test, as multisets: the fixtures each
test asked for and how many times. It is the group unit's guard — a cache
that one test fills and another reads must credit the reader
(`testenv::Shared`), or the reader's rows are gone — and it exits 1 on any
test whose rows differ. Tests only one side ran are listed and are not a
difference.

`--stand-ups` counts the audit's games and the tests that walk them: the
`Suite:` score's first number is the processes that stand a kept game up.
"""
import argparse
import collections
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))

GAME_FIXTURES = 20
GAME_TRACES = 2
SWEEP_FIXTURES = 100
TRACE = 'rontrace-'


def read_audit(source):
    """`{test: Counter(fixture)}` from a gate's report directory (its
    `fixture-requests/*.tsv`), a directory of such files, or a JSON file
    this tool or a baseline wrote (`{test: {fixture: count}}`)."""
    source = Path(source)
    if source.is_file():
        return {test: collections.Counter(rows) for test, rows in json.loads(source.read_text()).items()}
    directory = source / 'fixture-requests' if (source / 'fixture-requests').is_dir() else source
    by = collections.defaultdict(collections.Counter)
    for path in sorted(directory.glob('*.tsv')):
        for line in path.read_text().splitlines():
            columns = line.split('\t')
            if len(columns) != 3 or columns[0] not in ('0', '1'):
                raise ValueError(f'malformed fixture audit: {path}')
            name, test = (bytes.fromhex(value).decode('utf-8') for value in columns[1:])
            by[test][name] += 1
    return dict(by)


def components(tests, links):
    """The connected components of `tests` under `links(test)` — the
    fixtures that join it to another — as sorted lists, largest link
    first in none: the order is the tests' own."""
    parent = {}

    def find(node):
        while parent.setdefault(node, node) != node:
            parent[node] = parent[parent[node]]
            node = parent[node]
        return node

    for test in tests:
        for fixture in links(test):
            parent[find(('test', test))] = find(('fixture', fixture))
    found = collections.defaultdict(list)
    for test in tests:
        if ('test', test) in parent:
            found[find(('test', test))].append(test)
    return sorted(sorted(group) for group in found.values())


def game_key(tests, audit):
    """A game group's name: its smallest trace, without the file's dress."""
    traces = sorted(f for test in tests for f in audit[test] if f.startswith(TRACE))
    return 'game:' + traces[0][len(TRACE):].removesuffix('.log')


def walks_a_game(rows):
    """Whether a test's requests are those of a walk of one game: few
    files, and one trace or two (the base's and a window's)."""
    traces = sum(f.startswith(TRACE) for f in rows)
    return 1 <= len(rows) <= GAME_FIXTURES and 1 <= traces <= GAME_TRACES


def groups_of(audit, wide=()):
    """`{key: [test, ...]}`: the corpus sweeps by module, then the games."""
    out = {}
    sweeps = collections.defaultdict(list)
    for test, rows in audit.items():
        if len(rows) > SWEEP_FIXTURES and test not in wide:
            sweeps[test.split('::')[0]].append(test)
    for module, tests in sorted(sweeps.items()):
        if len(tests) > 1:
            out[f'sweep:{module}'] = sorted(tests)
    games = [test for test, rows in audit.items() if walks_a_game(rows) and test not in wide]
    for tests in components(games, lambda test: [f for f in audit[test] if f.startswith(TRACE)]):
        if len(tests) > 1:
            key = game_key(tests, audit)
            assert key not in out, key
            out[key] = tests
    return out


def stand_ups(audit):
    """`(games, walkers)`: the games the audit's tests walk — the
    components of the tests that ask for a trace and at most
    `GAME_FIXTURES` files — and how many such tests there are. One process
    a test, every walker stands its game up from frame 0."""
    games = [test for test, rows in audit.items() if walks_a_game(rows)]
    return len(components(games, lambda test: [f for f in audit[test] if f.startswith(TRACE)])), len(games)


def audit_differences(a, b):
    """`(differing, only_a, only_b)`: for each test both audits name, the
    fixtures whose request counts differ as `{fixture: (count a, count
    b)}`; and the tests one side alone names. The thread named `unnamed`
    is no test and is compared like one."""
    differing = {}
    for test in sorted(set(a) & set(b)):
        delta = {fixture: (a[test].get(fixture, 0), b[test].get(fixture, 0))
                 for fixture in sorted(set(a[test]) | set(b[test]))
                 if a[test].get(fixture, 0) != b[test].get(fixture, 0)}
        if delta:
            differing[test] = delta
    return differing, sorted(set(a) - set(b)), sorted(set(b) - set(a))


def peaks_of(report):
    """`{group: MiB}` from a grouped run's `measurements.jsonl`: the peak
    resident set of each group's process that exited by itself (zero, or
    101 for a failing test), in whole MiB, rounded up."""
    report = Path(report)
    path = report / 'measurements.jsonl' if report.is_dir() else report
    peaks = {}
    for line in path.read_text().splitlines():
        row = json.loads(line)
        if row.get('group') and row.get('rss_bytes') and row.get('group_exit') in (0, 101):
            peaks[row['group']] = max(peaks.get(row['group'], 0), -(-row['rss_bytes'] // 2**20))
    return peaks


def wide_tests():
    """The tests the runner's profile runs wider than one slot."""
    import isolated_tests
    profile = json.loads(isolated_tests.PROFILE.read_text())['tests']
    return {name for name, row in profile.items() if int(row.get('cpu', 1)) > 1}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('--from', dest='source', type=Path, help="a gate's report directory, or a saved audit")
    ap.add_argument('--out', type=Path, help='the groups file to write (default: stdout)')
    ap.add_argument('--audit', nargs=2, type=Path, metavar=('A', 'B'),
                    help='compare two audits test for test, as multisets; exit 1 on a difference')
    ap.add_argument('--stand-ups', type=Path, metavar='REPORT', help="count an audit's games and their walkers")
    ap.add_argument('--peaks', type=Path, metavar='REPORT',
                    help="write each group's measured peak from a grouped run into --out's memory_mib")
    args = ap.parse_args(argv)
    if args.audit:
        a, b = (read_audit(path) for path in args.audit)
        differing, only_a, only_b = audit_differences(a, b)
        print(f'{len(set(a) & set(b))} tests in both audits; {len(differing)} differ; '
              f'{len(only_a)} only in A, {len(only_b)} only in B')
        for test, delta in list(differing.items())[:40]:
            shown = ', '.join(f'{fixture} {x}→{y}' for fixture, (x, y) in list(delta.items())[:3])
            print(f'  differs  {test}: {len(delta)} fixtures ({shown})')
        for label, names in (('only in A', only_a), ('only in B', only_b)):
            for test in names[:20]:
                print(f'  {label}  {test}')
        return 1 if differing else 0
    if args.stand_ups:
        games, walkers = stand_ups(read_audit(args.stand_ups))
        print(f'{walkers} tests walk {games} games')
        return 0
    if args.peaks and not args.source:
        if not args.out or not args.out.is_file():
            ap.error('--peaks rewrites an existing groups file: name it with --out')
        doc = json.loads(args.out.read_text())
        measured = peaks_of(args.peaks)
        memory = dict(doc.get('memory_mib', {}))
        memory.update({key: mib for key, mib in measured.items() if key in doc['groups']})
        doc['memory_mib'] = dict(sorted(memory.items()))
        doc['memory_source'] = f'tools/suite_groups.py --peaks {args.peaks.name}'
        args.out.write_text(json.dumps(doc, indent=1) + '\n')
        print(f"wrote {args.out}: {len(memory)} of {len(doc['groups'])} groups have a measured peak; "
              f"largest {max(memory.values(), default=0)} MiB")
        return 0
    if not args.source:
        ap.error('one of --from, --peaks, --audit and --stand-ups is needed')
    audit = read_audit(args.source)
    groups = groups_of(audit, wide=wide_tests())
    memory = {}
    if args.out and args.out.is_file():
        before = json.loads(args.out.read_text())
        memory = {key: mib for key, mib in before.get('memory_mib', {}).items()
                  if groups.get(key) == before['groups'].get(key)}
    if args.peaks:
        memory.update({key: mib for key, mib in peaks_of(args.peaks).items() if key in groups})
    tree = subprocess.run(['git', 'rev-parse', '--short=8', 'HEAD'], cwd=ROOT,
                          capture_output=True, text=True, check=True).stdout.strip()
    doc = {'schema': 1, 'tree': tree,
           'source': f"tools/suite_groups.py --from {args.source.name}: {len(audit)} tests' fixture requests",
           'groups': groups}
    if memory:
        doc['memory_mib'] = dict(sorted(memory.items()))
    text = json.dumps(doc, indent=1) + '\n'
    if args.out:
        args.out.write_text(text)
        held = sum(len(tests) for tests in groups.values())
        print(f'wrote {args.out}: {len(groups)} groups hold {held} tests; '
              f'largest {max(map(len, groups.values()), default=0)}')
    else:
        sys.stdout.write(text)
    return 0


if __name__ == '__main__':
    sys.exit(main())
