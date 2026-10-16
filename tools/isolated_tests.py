#!/usr/bin/env python3
"""Run all Cargo release test targets, isolating each rondata library test.

Every discovered nonignored test runs exactly once, including unknown tests.
No verdicts are cached. Test logs, discovery manifests and measurements survive
failures. Resource admission is shared with other cooperating worktrees.
"""
import argparse
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
from resource_budget import DEFAULT, MEMORY_MIB

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


def reservation(name, profile):
    row = profile.get(name)
    # Unknown tests run alone with the entire host memory reservation.
    return MEMORY_MIB if row is None else min(MEMORY_MIB, max(128, math.ceil(row['rss_bytes'] * 1.5 / 2**27) * 128))


def wrapper(directory, memory, cpu=1):
    return [sys.executable, str(ROOT / 'tools/resource_budget.py'), '--directory', str(directory),
            '--cpu', str(cpu), '--memory-mib', str(memory), '--']


def run_one(binary, name, number, output, directory, memory, cancelled):
    log = output / 'tests' / f'{number:04d}.log'
    start = time.monotonic()
    with log.open('x') as sink:
        command = wrapper(directory, memory) + ['/usr/bin/time', '-l', str(binary), '--exact', name, '--test-threads=1']
        with subprocess.Popen(command, stdout=sink, stderr=subprocess.STDOUT) as child:
            while child.poll() is None:
                if cancelled():
                    child.terminate()  # Supervisor terminates and reaps its process group.
                try:
                    child.wait(timeout=0.2)
                except subprocess.TimeoutExpired:
                    pass
    text = log.read_text()
    clock = re.search(r'([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys', text)
    rss = re.search(r'(\d+)\s+maximum resident set size', text)
    success = child.returncode == 0 and len(ONE.findall(text)) == 1 and bool(clock and rss)
    return dict(name=name, success=success, exit=child.returncode, elapsed=time.monotonic()-start,
                user=float(clock[2]) if clock else None, system=float(clock[3]) if clock else None,
                rss_bytes=int(rss[1]) if rss else None, reservation_mib=memory, log=str(log))


def isolated(binary, output, directory, slots=8):
    output.mkdir(parents=True, exist_ok=False)
    (output / 'tests').mkdir()
    all_tests, ignored = discover(binary), discover(binary, True)
    if not all_tests or not set(ignored) <= set(all_tests):
        raise ValueError('invalid or empty test manifest')
    names = [n for n in all_tests if n not in set(ignored)]
    profile = json.loads(PROFILE.read_text())['tests']
    pending = sorted(enumerate(names), key=lambda p: -profile.get(p[1], {}).get('elapsed', 0))
    (output / 'manifest.json').write_text(json.dumps(dict(binary=str(binary), tests=names, ignored=ignored), indent=2)+'\n')
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
                                         reservation(name, profile), lambda: cancelled)
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
    complete = len(rows) == len(names) and {r['name'] for r in rows} == set(names)
    passed = sum(r['success'] for r in rows)
    success = complete and passed == len(names) and not cancelled
    summary = dict(complete=complete, success=success, passed=passed, failed=len(rows)-passed,
                   ignored=len(ignored), elapsed=time.monotonic()-start,
                   cpu=sum(r['user']+r['system'] for r in rows if r['user'] is not None and r['system'] is not None))
    (output / 'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(f"test result: {'ok' if success else 'FAILED'}. {passed} passed; {len(rows)-passed} failed; "
          f"{len(ignored)} ignored; 0 measured; 0 filtered out; finished in {summary['elapsed']:.2f}s", flush=True)
    return 0 if success else (130 if cancelled else 101)


def release(output, directory, slots, test_threads):
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, CARGO_BUILD_JOBS='8')
    build = subprocess.run(wrapper(directory, MEMORY_MIB, 8) +
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
    exits = []
    for executable, target in targets:
        print(f"     Running unittests {target['src_path']} ({executable})", flush=True)
        if target['name'] == 'rondata' and 'lib' in target['kind']:
            rc = isolated(Path(executable), output / 'rondata', directory, slots)
        else:
            rc = subprocess.run(wrapper(directory, MEMORY_MIB, 8) +
                                [executable, f'--test-threads={test_threads}'], cwd=ROOT, env=env).returncode
        exits.append(rc)
        if rc in (130, -signal.SIGINT, -signal.SIGTERM):
            return 130
    # --no-run does not discover doctests. Keep Cargo responsible for them.
    exits.append(subprocess.run(wrapper(directory, MEMORY_MIB, 8) +
                 ['cargo', 'test', '--workspace', '--release', '--doc', '--no-fail-fast', '--',
                  f'--test-threads={test_threads}'], cwd=ROOT, env=env).returncode)
    (output / 'exit-codes.json').write_text(json.dumps(exits)+'\n')
    return next((rc for rc in exits if rc), 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--resource-directory', type=Path, default=DEFAULT)
    parser.add_argument('--slots', type=int, choices=range(1, 9), default=8)
    parser.add_argument('--test-threads', type=int, default=4)
    args = parser.parse_args()
    return release(args.output.resolve(), args.resource_directory, args.slots, args.test_threads)


if __name__ == '__main__':
    sys.exit(main())
