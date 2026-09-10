#!/usr/bin/env python3
"""Run the full local gate with an explicit install, including its data survey."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


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


def gate(install, *, report_dir=None, require_fixtures=False, test_threads=2, run=subprocess.run):
    if type(test_threads) is not int or test_threads not in (2, 3, 4):
        raise ValueError('test threads must be 2, 3, or 4')
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
    (report_dir / 'gate-policy.json').write_text(json.dumps(
        {'schema': 1, 'release_test_threads': test_threads, 'memory_cap_gib': 20},
        indent=2) + '\n')
    # No game launch: rondata surveys the user's data files.
    commands = [
        [sys.executable, 'tools/offline_tests.py'],
        ['cargo', 'run', '-p', 'rondata', '--', str(install)],
        ['zsh', 'tools/memcap.sh', '20', 'cargo', 'test', '--release', '--', f'--test-threads={test_threads}'],
        ['cargo', 'clippy', '--all-targets', '--', '-D', 'warnings'],
        ['cargo', 'fmt', '--check'],
        ['zsh', 'tools/guard.sh'],
    ]
    for command in commands:
        child_env = env.copy()
        is_release = '--release' in command
        completed = False
        if is_release:
            child_env['RON_FIXTURE_AUDIT_DIR'] = str(audit_dir)
        try:
            run(command, cwd=ROOT, env=child_env, check=True)
            completed = True
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
        if is_release:
            if not summary['observed_requests']:
                raise ValueError('release produced no fixture audit; coverage is unobserved')
            if require_fixtures and summary['missing_fixtures']:
                raise ValueError('requested fixtures are missing; see fixture-coverage.json')
    return report_dir


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path, help='owned install directory; always exported as RON_INSTALL')
    parser.add_argument('--report-dir', type=Path, help='fresh output directory; defaults to a retained temporary directory')
    parser.add_argument('--require-fixtures', action='store_true', help='fail if any observed fixture request was missing')
    parser.add_argument('--test-threads', type=int, choices=(2, 3, 4), default=2,
                        help='release width under the 20 GiB monitor; default: 2')
    args = parser.parse_args()
    try:
        gate(args.install, report_dir=args.report_dir, require_fixtures=args.require_fixtures, test_threads=args.test_threads)
    except ValueError as exc:
        parser.error(str(exc))


if __name__ == '__main__':
    main()
