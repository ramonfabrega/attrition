"""Show where a saved isolated release spent time and sample a rondata test process."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile

from test_runtime import execution_timeout

ROOT = Path(__file__).resolve().parents[1]
VERIFIERS = ('RON_VERIFY_ORACLE', 'RON_VERIFY_SETUP_OBSERVATIONS', 'RON_VERIFY_EMU')


def replay_settings(env):
    """Runtime switches that change how an isolated replay obtains observations."""
    return {k: v for k, v in env.items() if k.startswith(('RON_VERIFY_', 'RON_DISABLE_', 'RON_PROFILE_'))
            or k in ('RON_ORACLE_CACHE_DIR', 'RON_GOLDEN_DIR', 'RON_SHARE_WALKS',
                     'RON_SUITE_GROUP', 'RON_SUITE_GROUP_TESTS')}


def measurements(directory):
    with (directory / 'rondata/measurements.jsonl').open() as stream:
        return [json.loads(line) for line in stream]


def report(directory):
    lanes = json.loads((directory / 'lanes.json').read_text())
    finishes = {row['name']: row['end'] for row in lanes['side']}
    if 'library' in lanes:
        finishes['rondata library'] = lanes['library']['end']
    if finishes:
        last = max(finishes, key=finishes.get)
        print(f'Release timing: {last} finished last at {finishes[last]:.1f} s')
    if not (directory / 'rondata/measurements.jsonl').is_file():
        return
    cpu = Counter()
    sampleable = set()
    for row in measurements(directory):
        if row['user'] is None or row['system'] is None:
            continue
        unit = row.get('group') or row['name']
        cpu[unit] += row['user'] + row['system']
        if not row.get('group') or 'share_walks' in row:
            sampleable.add(unit)
    for unit, seconds in cpu.most_common(5):
        print(f'  {seconds:8.1f} CPU s  {unit}')
    if sampleable:
        print('Profile the highest-CPU reproducible process: ' + shlex.join([
            'python3', 'tools/gate_profile.py', str(directory.resolve()),
            '--sample', next(unit for unit, _ in cpu.most_common() if unit in sampleable)]))


def test_requests(directory, names):
    """Compare presence and count for this test without loading other tests' rows."""
    encoded = {name.encode().hex().encode() for name in names}
    requests = Counter()
    for path in directory.glob('*.tsv'):
        with path.open('rb') as stream:
            for line in stream:
                row = line.rstrip(b'\n')
                if row.rsplit(b'\t', 1)[-1] in encoded:
                    requests[row] += 1
    return requests


def sample(directory, name, binary, output, seconds, delay=0):
    manifest = json.loads((directory / 'rondata/manifest.json').read_text())
    row = next((r for r in measurements(directory) if r['name'] == name or r.get('group') == name), None)
    if row is None:
        raise ValueError(f'no recorded run of {name}')
    group = row.get('group')
    tests = manifest['groups'][group] if group else [name]
    if group and 'share_walks' not in row:
        raise ValueError(f"older report lacks the shared-walk setting for {group}")
    original = manifest['binary']
    binary = binary or Path(original)
    expected = json.loads((directory / 'binary-sha256.json').read_text())[original]
    stamp = binary.stat()
    with binary.open('rb') as stream:
        if hashlib.file_digest(stream, 'sha256').hexdigest() != expected:
            raise ValueError('binary differs from the gate; pass --binary with a matching saved executable')
    if not manifest['install'] or not (Path(manifest['install']) / 'Data/rules.xml').is_file():
        raise ValueError('the recorded game install is unavailable')
    settings = manifest.get('replay_settings')
    if settings is None:
        policy = json.loads((directory.parent / 'gate-policy.json').read_text())
        settings = {k: '1' for k in VERIFIERS} if policy['verify_caches'] else {}
    env = os.environ.copy()
    for key in replay_settings(env):
        del env[key]
    env.update(replay_settings(settings))
    env.update(RON_INSTALL=manifest['install'], RON_TEST_CPU=str(row['cpu_slots']),
               RON_TEST_ALLOCATOR=row['allocator']['requested'])
    if group:
        env.update(RON_SUITE_GROUP=group, RON_SUITE_GROUP_TESTS=str(len(tests)), RON_SHARE_WALKS=row['share_walks'])
        print(f'Profiling all {len(tests)} tests in {group}', flush=True)
    env.pop('MallocLargeCache', None)
    if row['allocator']['MallocLargeCache'] is not None:
        env['MallocLargeCache'] = row['allocator']['MallocLargeCache']
    if output is None:
        output = Path(tempfile.mkdtemp(prefix='attrition-native-')) / 'profile'
    output = output.resolve()
    env['RON_FIXTURE_AUDIT_DIR'] = str(output / 'fixture-requests')
    print(f'Native profile and fixture requests: {output}', flush=True)
    timeout = row.get('timeout_seconds', execution_timeout(name))
    timeout_args = ['--timeout', str(timeout)] if timeout is not None else []
    code = subprocess.run([
        sys.executable, str(ROOT / 'tools/resource_budget.py'),
        '--directory', manifest['resource_directory'], '--cpu', str(row['cpu_slots']),
        '--memory-mib', str(row['reservation_mib']), '--limit-mib', str(row['limit_mib']),
        *timeout_args, '--', sys.executable,
        str(ROOT / 'tools/explore/profile_native.py'), str(binary.resolve()), tests[0],
        str(output), '--seconds', str(seconds), '--delay-seconds', str(delay),
        *[arg for test in tests[1:] for arg in ('--with-test', test)]], env=env, cwd=ROOT).returncode
    if code:
        return code
    after = binary.stat()
    if (stamp.st_dev, stamp.st_ino, stamp.st_size, stamp.st_mtime_ns) != (
            after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
        raise ValueError('test binary changed during profiling')
    baseline = directory.parent / 'fixture-requests'
    if baseline.is_dir():
        expected = test_requests(baseline, tests)
        actual = test_requests(output / 'fixture-requests', tests)
        if actual != expected:
            raise ValueError('profiled test requested different fixtures than the saved gate')
        print(f'Profile fixture audit matched all {sum(actual.values())} requests')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path, help='whole gate report or its isolated directory')
    parser.add_argument('--sample', metavar='TEST_OR_GROUP', help='record a macOS native sample under the measured reservation')
    parser.add_argument('--binary', type=Path, help='saved copy of the gate binary')
    parser.add_argument('--output', type=Path, help='new profile directory (default: a new temporary directory)')
    parser.add_argument('--seconds', type=int, default=18)
    parser.add_argument('--delay-seconds', type=int, default=0, help='delay sampling until the test is warmed up')
    args = parser.parse_args()
    directory = args.report.resolve()
    if (directory / 'isolated').is_dir():
        directory = directory / 'isolated'
    try:
        if args.sample:
            if args.delay_seconds < 0:
                parser.error('--delay-seconds must be nonnegative')
            if args.seconds < 1:
                parser.error('--seconds must be positive')
            return sample(directory, args.sample, args.binary, args.output, args.seconds, args.delay_seconds)
        report(directory)
        return 0
    except (OSError, ValueError, KeyError) as exc:
        parser.exit(1, f'{parser.prog}: {exc}\n')


if __name__ == '__main__':
    sys.exit(main())
