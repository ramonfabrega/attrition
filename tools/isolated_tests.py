#!/usr/bin/env python3
"""Run all Cargo release test targets, isolating each rondata library test.

Every selected nonignored test runs exactly once, including unknown tests.
Only the gate's explicit exact-name exclusions are applied.
No verdicts are cached. Test logs, discovery manifests and measurements survive
failures. Resource admission is shared with other cooperating worktrees.
"""
import argparse
import hashlib
from concurrent.futures import ThreadPoolExecutor, wait, FIRST_COMPLETED
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from resource_budget import DEFAULT, MEMORY_MIB, QUANTUM_MIB
from test_runtime import child_environment, allocator_policy

PROFILE = ROOT / 'docs/audit/2026-10-16-isolated-profile.json'
ONE = re.compile(r'^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;', re.M)


def discover(binary, ignored=False):
    result = subprocess.run([str(binary), '--list', '--format=terse'] +
                            (['--ignored'] if ignored else []), check=True, capture_output=True, text=True)
    lines = result.stdout.splitlines()
    if any(not line.endswith(': test') for line in lines):
        raise ValueError('unexpected libtest discovery output')
    names = [line[:-6] for line in lines]
    if len(set(names)) != len(names):
        raise ValueError('duplicate discovered test')
    return names


def binary_stamp(binary):
    stat = Path(binary).stat()
    return (stat.st_dev, stat.st_ino, stat.st_size, stat.st_mtime_ns)


UNKNOWN_MIB = 8192
UNKNOWN_LIMIT_MIB = 32768


def reservation(name, profile):
    row = profile.get(name)
    # A test the profile does not know reserves 8 GiB and runs under a 32 GiB
    # ceiling — the old lane memcap, not the whole host: every test a tranche
    # adds is unknown until the profile is refreshed (DECISIONS 75).
    return UNKNOWN_MIB if row is None else min(MEMORY_MIB, max(QUANTUM_MIB,
        math.ceil(row['rss_bytes'] / (QUANTUM_MIB * 2**20)) * QUANTUM_MIB))


def wrapper(directory, memory, cpu=1, limit=None):
    return [sys.executable, str(ROOT / 'tools/resource_budget.py'), '--directory', str(directory),
            '--cpu', str(cpu), '--memory-mib', str(memory)] + (
            ['--limit-mib', str(limit)] if limit is not None else []) + ['--']


def read_result(log):
    """Bound memory even when a failing whole-record assertion prints huge rows."""
    passed = 0
    clock = rss = None
    with log.open() as stream:
        while part := stream.readline(65536):
            passed += len(ONE.findall(part))
            found = re.search(r'([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys', part)
            if found:
                clock = (float(found[2]), float(found[3]))
            found = re.search(r'(\d+)\s+maximum resident set size', part)
            if found:
                rss = int(found[1])
    return passed, clock, rss


def resource_timing(log):
    with log.open() as stream:
        for part in iter(lambda: stream.readline(65536), ''):
            if part.startswith('resource budget timing: '):
                return json.loads(part.removeprefix('resource budget timing: '))
    return None


def run_one(binary, name, number, output, directory, memory, cancelled, known=True):
    log = output / 'tests' / f'{number:04d}.log'
    start = time.monotonic()
    stamp = binary_stamp(binary)
    env = child_environment(test_name=name)
    with log.open('x') as sink:
        limit = min(MEMORY_MIB, math.ceil(memory * 1.5 / QUANTUM_MIB) * QUANTUM_MIB) if known else UNKNOWN_LIMIT_MIB
        command = wrapper(directory, memory, limit=limit) + ['/usr/bin/time', '-l', str(binary), '--exact', name, '--test-threads=1']
        with subprocess.Popen(command, stdout=sink, stderr=subprocess.STDOUT, env=env) as child:
            while child.poll() is None:
                if cancelled():
                    child.terminate()  # Supervisor terminates and reaps its process group.
                try:
                    child.wait(timeout=0.2)
                except subprocess.TimeoutExpired:
                    pass
    passed, clock, rss = read_result(log)
    success = (child.returncode == 0 and passed == 1 and clock is not None and rss is not None
               and binary_stamp(binary) == stamp)
    return dict(name=name, success=success, exit=child.returncode, elapsed=time.monotonic()-start,
                user=clock[0] if clock else None, system=clock[1] if clock else None,
                rss_bytes=rss, reservation_mib=memory, log=str(log), allocator=allocator_policy(env),
                resource_timing=resource_timing(log))


def isolated(binary, output, directory, slots=8, skips=()):
    output.mkdir(parents=True, exist_ok=False)
    (output / 'tests').mkdir()
    stamp = binary_stamp(binary)
    all_tests, ignored = discover(binary), discover(binary, True)
    if not all_tests or not set(ignored) <= set(all_tests):
        raise ValueError('invalid or empty test manifest')
    excluded = set(skips)
    skipped = [n for n in all_tests if n in excluded]
    ignored = [n for n in ignored if n not in excluded]
    names = [n for n in all_tests if n not in set(ignored) and n not in excluded]
    if not names and not skipped:
        raise ValueError('no nonignored tests discovered')
    profile = json.loads(PROFILE.read_text())['tests']
    pending = sorted(enumerate(names), key=lambda p: -profile.get(p[1], {}).get('elapsed', 0))
    (output / 'manifest.json').write_text(json.dumps(dict(binary=str(binary), binary_stamp=stamp, tests=names, ignored=ignored, skipped=skipped, requested_skips=list(skips),
        install=os.environ.get('RON_INSTALL'), slots=slots, resource_directory=str(directory)), indent=2)+'\n')
    cancelled = False
    def cancel(_sig, _frame):
        nonlocal cancelled
        cancelled = True
    previous = {s: signal.signal(s, cancel) for s in (signal.SIGINT, signal.SIGTERM)}
    rows = []
    start = time.monotonic()
    try:
        with ThreadPoolExecutor(max_workers=slots) as pool:
            active = {}
            while (pending and not cancelled) or active:
                while pending and not cancelled and len(active) < slots:
                    i, name = pending.pop(0)
                    future = pool.submit(run_one, binary, name, i, output, directory,
                                         reservation(name, profile), lambda: cancelled, name in profile)
                    active[future] = name
                done, _ = wait(active, return_when=FIRST_COMPLETED, timeout=0.5)
                for future in done:
                    active.pop(future)
                    row = future.result()
                    rows.append(row)
                    with (output / 'measurements.jsonl').open('a') as sink:
                        sink.write(json.dumps(row)+'\n')
                    print(f"test {row['name']} ... {'ok' if row['success'] else 'FAILED'}", flush=True)
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    complete = (len(rows) == len(names) and {r['name'] for r in rows} == set(names)
                and binary_stamp(binary) == stamp)
    passed = sum(r['success'] for r in rows)
    success = complete and passed == len(names) and not cancelled
    summary = dict(complete=complete, success=success, passed=passed, failed=len(rows)-passed,
                   ignored=len(ignored), filtered=len(skipped), elapsed=time.monotonic()-start,
                   cpu=sum(r['user']+r['system'] for r in rows if r['user'] is not None and r['system'] is not None))
    (output / 'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(f"test result: {'ok' if success else 'FAILED'}. {passed} passed; {len(rows)-passed} failed; "
          f"{len(ignored)} ignored; 0 measured; {len(skipped)} filtered out; finished in {summary['elapsed']:.2f}s", flush=True)
    if cancelled:
        return 130
    if not complete:
        return 125
    for row in rows:
        if not row['success'] and row['exit'] != 101:
            return row['exit'] or 125
    return 0 if success else 101


def target_budget(target, test_threads):
    if target['kind'] == ['test'] and target['name'] in ('sweep_a_f', 'sweep_g_o', 'sweep_p_t', 'sweep_u_z'):
        return min(test_threads, 8), 4096
    if target['kind'] == ['lib'] and target['name'] in ('fixed', 'sim'):
        return min(test_threads, 8), 512 if target['name'] == 'fixed' else 4096
    return 8, 32768


def release(output, directory, slots, test_threads, skips=()):
    selection = (['--exact'] + [arg for name in skips for arg in ('--skip', name)]) if skips else []
    output.mkdir(parents=True, exist_ok=False)
    env = child_environment()
    env['CARGO_BUILD_JOBS'] = '8'
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    (output / 'revision.txt').write_text(revision + '\n')
    build = subprocess.run(wrapper(directory, 32768, 8) +
                           ['cargo', 'test', '--workspace', '--release', '--no-run', '--message-format=json'],
                           cwd=ROOT, env=env, capture_output=True, text=True)
    (output / 'build.jsonl').write_text(build.stdout)
    (output / 'build.stderr').write_text(build.stderr)
    print(build.stderr, end='', flush=True)
    if build.returncode:
        return build.returncode
    targets = []
    for line in build.stdout.splitlines():
        row = json.loads(line)
        if row.get('reason') == 'compiler-artifact' and row.get('executable') and row['profile']['test']:
            targets.append((row['executable'], row['target']))
    if not targets or len({p for p, _ in targets}) != len(targets):
        raise ValueError('empty or duplicate Cargo executable manifest')
    (output / 'targets.json').write_text(json.dumps(targets, indent=2)+'\n')
    digests = {}
    for executable, _ in targets:
        with open(executable, 'rb') as stream:
            digests[executable] = hashlib.file_digest(stream, 'sha256').hexdigest()
    (output / 'binary-sha256.json').write_text(json.dumps(digests, indent=2)+'\n')
    stamps = {executable: binary_stamp(executable) for executable, _ in targets}
    exits = []
    for executable, target in targets:
        if binary_stamp(executable) != stamps[executable]:
            raise ValueError('test executable changed after Cargo discovery')
        print(f"     Running unittests {target['src_path']} ({executable})", flush=True)
        if target['name'] == 'rondata' and 'lib' in target['kind']:
            rc = isolated(Path(executable), output / 'rondata', directory, slots, skips=skips)
        else:
            cpu, memory = target_budget(target, test_threads)
            rc = subprocess.run(wrapper(directory, memory, cpu) +
                                [executable, f'--test-threads={test_threads}', *selection], cwd=ROOT, env=env).returncode
        if binary_stamp(executable) != stamps[executable]:
            rc = 125
        exits.append(rc)
        if rc in (130, -signal.SIGINT, -signal.SIGTERM):
            return 130
    # --no-run does not discover doctests. Keep Cargo responsible for them.
    exits.append(subprocess.run(wrapper(directory, 32768, 8) +
                 ['cargo', 'test', '--workspace', '--release', '--doc', '--no-fail-fast', '--',
                  f'--test-threads={test_threads}', *selection], cwd=ROOT, env=env).returncode)
    (output / 'exit-codes.json').write_text(json.dumps(exits)+'\n')
    return next((rc for rc in exits if rc), 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--resource-directory', type=Path, default=DEFAULT)
    parser.add_argument('--slots', type=int, choices=range(1, 9), default=8)
    parser.add_argument('--test-threads', type=int, default=4)
    parser.add_argument('--skip', action='append', default=[], help='exact libtest names excluded by the gate selection')
    args = parser.parse_args()
    return release(args.output.resolve(), args.resource_directory, args.slots, args.test_threads, args.skip)


if __name__ == '__main__':
    sys.exit(main())
