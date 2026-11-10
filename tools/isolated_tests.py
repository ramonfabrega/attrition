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
import queue
import re
import shutil
import signal
import subprocess
import sys
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from resource_budget import DEFAULT, MEMORY_MIB, QUANTUM_MIB
from test_runtime import child_environment, allocator_policy, execution_timeout

PROFILE = ROOT / 'docs/audit/2026-10-16-isolated-profile.json'
ONE = re.compile(r'^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;', re.M)
VERDICT = re.compile(r'^test result: (?:ok|FAILED)\. \d+ passed; \d+ failed;', re.M)


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


def width(name, profile, slots):
    """The CPU slots a test reserves: its profile row's `cpu`, default one,
    never more than the pool (item 4072, Loop 4042). A test whose row says
    `cpu: k` runs k-wide inside its one process — the coverage pin's walk —
    and is told so by `RON_TEST_CPU`; the pool counts it as k of its slots,
    so the width is reserved rather than taken from tests the pool thinks
    are alone."""
    row = profile.get(name) or {}
    return max(1, min(slots, int(row.get('cpu', 1))))


def wrapper(directory, memory, cpu=1, limit=None, timeout=None):
    return [sys.executable, str(ROOT / 'tools/resource_budget.py'), '--directory', str(directory),
            '--cpu', str(cpu), '--memory-mib', str(memory)] + (
            ['--limit-mib', str(limit)] if limit is not None else []) + (
            ['--timeout', str(timeout)] if timeout is not None else []) + ['--']


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


def stopped_at(log):
    """`(rss, limit)` in MiB when the resource budget stopped the test, else
    `(None, None)` — the budget's own stderr line, read back (Loop 3167)."""
    with log.open() as stream:
        for part in iter(lambda: stream.readline(65536), ''):
            found = re.search(r'resource budget: RSS (\d+) MiB / limit (\d+) MiB', part)
            if found:
                return int(found[1]), int(found[2])
    return None, None


def resource_timing(log):
    with log.open() as stream:
        for part in iter(lambda: stream.readline(65536), ''):
            if part.startswith('resource budget timing: '):
                return json.loads(part.removeprefix('resource budget timing: '))
    return None


def run_one(binary, name, number, output, directory, memory, cancelled, known=True, cpu=1):
    log = output / 'tests' / f'{number:04d}.log'
    start = time.monotonic()
    stamp = binary_stamp(binary)
    env = child_environment(test_name=name)
    env['RON_TEST_CPU'] = str(cpu)
    with log.open('x') as sink:
        limit = min(MEMORY_MIB, math.ceil(memory * 1.5 / QUANTUM_MIB) * QUANTUM_MIB) if known else UNKNOWN_LIMIT_MIB
        command = wrapper(directory, memory, cpu=cpu, limit=limit, timeout=execution_timeout(name)) + ['/usr/bin/time', '-l', str(binary), '--exact', name, '--test-threads=1']
        with subprocess.Popen(command, stdout=sink, stderr=subprocess.STDOUT, env=env) as child:
            while child.poll() is None:
                if cancelled():
                    child.terminate()  # Supervisor terminates and reaps its process group.
                try:
                    child.wait(timeout=0.2)
                except subprocess.TimeoutExpired:
                    pass
    passed, clock, rss = read_result(log)
    stopped, seen_limit = stopped_at(log)
    success = (child.returncode == 0 and passed == 1 and clock is not None and rss is not None
               and binary_stamp(binary) == stamp)
    return dict(name=name, success=success, exit=child.returncode, elapsed=time.monotonic()-start,
                user=clock[0] if clock else None, system=clock[1] if clock else None,
                rss_bytes=rss, reservation_mib=memory, stopped_mib=stopped, limit_mib=seen_limit or limit,
                log=str(log), allocator=allocator_policy(env), resource_timing=resource_timing(log), cpu_slots=cpu)


def head_running(log):
    """Whether a test's log shows libtest's banner: the budget admitted it
    and its binary is running."""
    try:
        with log.open() as stream:
            return 'running ' in stream.read(65536)
    except FileNotFoundError:
        return False


def isolated(binary, output, directory, slots=8, skips=(), stop=None, started=None):
    """`stop` and `started` are the release step's lanes (Loop 4042): a
    signal here stops the lane beside the library too, and `started` is set
    once the head of the queue — the widest, longest test — is running, so
    the lane beside it is admitted after the library's head, never before."""
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
    profile_doc = json.loads(PROFILE.read_text())
    profile = profile_doc['tests']
    pending = sorted(enumerate(names), key=lambda p: -profile.get(p[1], {}).get('elapsed', 0))
    (output / 'manifest.json').write_text(json.dumps(dict(binary=str(binary), binary_stamp=stamp, tests=names, ignored=ignored, skipped=skipped, requested_skips=list(skips),
        install=os.environ.get('RON_INSTALL'), slots=slots, resource_directory=str(directory)), indent=2)+'\n')
    cancelled = False
    def cancel(_sig, _frame):
        nonlocal cancelled
        cancelled = True
        if stop is not None:
            stop.set()
    def halted():
        return cancelled or (stop is not None and stop.is_set())
    previous = {s: signal.signal(s, cancel) for s in (signal.SIGINT, signal.SIGTERM)}
    rows = []
    start = time.monotonic()
    head = None
    try:
        with ThreadPoolExecutor(max_workers=slots) as pool:
            # Slots are counted by width (item 4072): the head of the queue
            # starts when its width fits, and waits otherwise — first in,
            # first out, so a wide test is never starved by narrow ones.
            active = {}
            used = 0
            while (pending and not halted()) or active:
                while pending and not halted() and used + width(pending[0][1], profile, slots) <= slots:
                    i, name = pending.pop(0)
                    cpu = width(name, profile, slots)
                    future = pool.submit(run_one, binary, name, i, output, directory,
                                         reservation(name, profile), halted, name in profile, cpu)
                    if head is None:
                        head = (future, output / 'tests' / f'{i:04d}.log')
                    active[future] = cpu
                    used += cpu
                waiting = started is not None and not started.is_set()
                done, _ = wait(active, return_when=FIRST_COMPLETED, timeout=0.1 if waiting else 0.5)
                if waiting and head is not None and (head[0].done() or head_running(head[1])):
                    started.set()
                for future in done:
                    used -= active.pop(future)
                    row = future.result()
                    rows.append(row)
                    with (output / 'measurements.jsonl').open('a') as sink:
                        sink.write(json.dumps(row)+'\n')
                    print(f"test {row['name']} ... {'ok' if row['success'] else 'FAILED'}", flush=True)
    finally:
        if started is not None:
            started.set()
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    cancelled = halted()
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
    for row in rows:
        # Loop 3167: a test the budget stopped under a limit its own profile
        # row set is a stale row, not a 137 — say so, with the remedy.
        if not row['success'] and row['exit'] == 137 and row.get('stopped_mib') is not None and row['name'] in profile:
            known = round(profile[row['name']]['rss_bytes'] / 2**20)
            print(f"stale profile row: {row['name']} stopped at {row['stopped_mib']} MiB against its "
                  f"{row['limit_mib']} MiB limit, reserved {row['reservation_mib']} MiB from a {known} MiB row "
                  f"({PROFILE.name}, tree {profile_doc.get('tree', '?')}): raise its rss_bytes to the measured RSS "
                  f"or re-profile; alone it may pass", flush=True)
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
        # Loop 3777 (the sixty-third pass): the emu cache now keeps the
        # over-cap rows compressed, and every whole gate verifies them; at
        # eight threads under verify with two giants bound `sweep_u_z` read
        # 3,859 MiB sampled, 6 % under the old 4,096 (and the build before
        # the bound was stopped at 4,110). The lib's 6,144: the four targets
        # run one at a time, so the reservation is held by one at a time.
        return min(test_threads, 8), 6144
    if target['kind'] == ['lib'] and target['name'] == 'fixed':
        return min(test_threads, 8), 512
    if target['kind'] == ['lib'] and target['name'] == 'sim':
        # Loop 3363: at eight threads the lib peaked at 4218, 4249 and 4262
        # MiB on three whole gates and was stopped with no test red (the
        # 12000-row sweeps in `sweep/mod.rs`, which the standalone targets
        # cannot hold); the same tree at four threads peaks under 4096. The
        # limit follows the width, 512 MiB a thread past four.
        return min(test_threads, 8), 4096 + 512 * max(0, min(test_threads, 8) - 4)
    return 8, 32768


def package_name(manifest_path, read=lambda path: Path(path).read_text()):
    """The `[package] name` of a Cargo manifest — what `--exclude` names."""
    import tomllib
    return tomllib.loads(read(manifest_path))['package']['name']


def kept_targets(rows, exclude=(), read=lambda path: Path(path).read_text()):
    """The test executables Cargo's `--no-run --message-format=json` rows
    name, less the packages in `exclude` (Loop 3653, the fifty-seventh
    pass: a lane that touches no sim path leaves the sim crate's binaries
    to the booking gate; the gate passes `--exclude-package sim`). Each
    kept row is `(executable, target)`; each left one is `(package,
    src_path)`."""
    kept, left, names = [], [], {}
    for row in rows:
        if not (row.get('reason') == 'compiler-artifact' and row.get('executable') and row['profile']['test']):
            continue
        manifest = row.get('manifest_path')
        if manifest not in names:
            names[manifest] = package_name(manifest, read=read) if manifest else None
        if names[manifest] in exclude:
            left.append((names[manifest], row['target']['src_path']))
        else:
            kept.append((row['executable'], row['target']))
    return kept, left


def verdicts(log):
    """The libtest `test result:` lines a binary's log holds, read in bounded
    chunks: one binary, one verdict."""
    count = 0
    with log.open(errors='replace') as stream:
        while part := stream.readline(65536):
            count += len(VERDICT.findall(part))
    return count


def run_beside(executable, target, number, output, directory, test_threads, selection, env, stop, origin):
    """One binary of the lane beside the library, its output to its own log
    (printed whole once the library's block is done, so the gate's reader
    keys every line to its binary). A binary that exits zero without one
    verdict is an infrastructure failure, never a pass."""
    log = output / 'beside' / f'{number:02d}-{target["name"]}.log'
    cpu, memory = target_budget(target, test_threads)
    stamp = binary_stamp(executable)
    begun = time.monotonic()
    with log.open('x') as sink, subprocess.Popen(
            wrapper(directory, memory, cpu) + [executable, f'--test-threads={test_threads}', *selection],
            cwd=ROOT, env=env, stdout=sink, stderr=subprocess.STDOUT) as child:
        while child.poll() is None:
            if stop.is_set():
                child.terminate()  # The supervisor terminates and reaps its process group.
            try:
                child.wait(timeout=0.2)
            except subprocess.TimeoutExpired:
                pass
    rc = child.returncode
    if binary_stamp(executable) != stamp:
        rc = 125
    seen = verdicts(log)
    note = None
    if rc == 0 and seen != 1:
        rc = 125
        note = f'{target["src_path"]} exited 0 but printed {seen} verdicts; one binary, one verdict (Loop 4042)'
    return dict(executable=executable, name=target['name'], src_path=target['src_path'], log=str(log),
                exit=rc, verdicts=seen, start=begun - origin, end=time.monotonic() - origin, note=note)


def release(output, directory, slots, test_threads, skips=(), exclude_packages=()):
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
    rows = [json.loads(line) for line in build.stdout.splitlines()]
    targets, left = kept_targets(rows, exclude=tuple(exclude_packages))
    for package, src_path in left:
        print(f"     Leaving unittests {src_path} to the booking gate (package {package}, --exclude-package; Loop 3653)",
              flush=True)
    if not targets or len({p for p, _ in targets}) != len(targets):
        raise ValueError('empty or duplicate Cargo executable manifest')
    (output / 'targets.json').write_text(json.dumps(targets, indent=2)+'\n')
    digests = {}
    for executable, _ in targets:
        with open(executable, 'rb') as stream:
            digests[executable] = hashlib.file_digest(stream, 'sha256').hexdigest()
    (output / 'binary-sha256.json').write_text(json.dumps(digests, indent=2)+'\n')
    stamps = {executable: binary_stamp(executable) for executable, _ in targets}
    # Two lanes (Loop 4042): the isolated rondata library on its own pool,
    # and beside it every other binary, one at a time at the gate's width
    # under the same per-binary reservations as before. On 0e28830d6's whole
    # gate they ran one after another, the 454 s of binaries Cargo lists
    # first and then the library's 1,250 s; beside it the step is the
    # library's. The lane beside starts once the library's head test is
    # running, so the eight-wide coverage walk is admitted first.
    library = [(e, t) for e, t in targets if t['name'] == 'rondata' and 'lib' in t['kind']]
    beside = [(e, t) for e, t in targets if (e, t) not in library]
    (output / 'beside').mkdir()
    stop, started = threading.Event(), threading.Event()
    if not library:
        started.set()
    finished = queue.Queue()
    origin = time.monotonic()

    def lane():
        try:
            started.wait()
            for number, (executable, target) in enumerate(beside):
                if stop.is_set():
                    break
                if binary_stamp(executable) != stamps[executable]:
                    raise ValueError('test executable changed after Cargo discovery')
                finished.put(run_beside(executable, target, number, output, directory, test_threads,
                                        selection, env, stop, origin))
        except BaseException as error:  # Re-raised on the main thread.
            finished.put(error)
        finally:
            finished.put(None)

    def halt(_sig, _frame):
        stop.set()
    previous = {s: signal.signal(s, halt) for s in (signal.SIGINT, signal.SIGTERM)}
    exits_by, rows, spans = {}, [], {}
    worker = threading.Thread(target=lane, name='release-lane-beside', daemon=True)
    worker.start()
    try:
        for executable, target in library:
            if binary_stamp(executable) != stamps[executable]:
                raise ValueError('test executable changed after Cargo discovery')
            print(f"     Running unittests {target['src_path']} ({executable})", flush=True)
            begun = time.monotonic() - origin
            try:
                rc = isolated(Path(executable), output / 'rondata', directory, slots, skips=skips,
                              stop=stop, started=started)
            finally:
                started.set()
            if binary_stamp(executable) != stamps[executable]:
                rc = 125
            exits_by[executable] = rc
            spans['library'] = dict(start=begun, end=time.monotonic() - origin, exit=rc)
            if rc in (130, -signal.SIGINT, -signal.SIGTERM):
                stop.set()
        # The library's block is whole; each binary beside it prints its own
        # header and log in Cargo's order as it finishes.
        while True:
            try:
                row = finished.get(timeout=0.5)
            except queue.Empty:
                continue
            if row is None:
                break
            if isinstance(row, BaseException):
                raise row
            rows.append(row)
            print(f"     Running unittests {row['src_path']} ({row['executable']})", flush=True)
            with open(row['log'], errors='replace') as stream:
                shutil.copyfileobj(stream, sys.stdout, 65536)
            sys.stdout.flush()
            if row['note']:
                print(row['note'], flush=True)
            exits_by[row['executable']] = row['exit']
            if row['exit'] in (130, -signal.SIGINT, -signal.SIGTERM):
                stop.set()
    finally:
        if worker.is_alive():
            # A raise before the library ran must not leave the lane
            # waiting on a head that will never run.
            stop.set()
            started.set()
            worker.join()
        for sig, handler in previous.items():
            signal.signal(sig, handler)
    spans['side'] = [{k: row[k] for k in ('name', 'executable', 'start', 'end', 'exit', 'verdicts')} for row in rows]
    (output / 'lanes.json').write_text(json.dumps(spans, indent=2) + '\n')
    side_seconds = sum(row['end'] - row['start'] for row in rows)
    library_seconds = spans['library']['end'] - spans['library']['start'] if 'library' in spans else 0.0
    last = max([row['end'] for row in rows] + [spans.get('library', {}).get('end', 0.0)])
    first = f", the first {rows[0]['start'] - spans['library']['start']:.1f} s after it" if rows and library else ''
    print(f"Release lanes: library {library_seconds:.1f} s; beside it {side_seconds:.1f} s over {len(rows)} "
          f"binaries{first}; both done at {last:.1f} s", flush=True)
    if stop.is_set() or any(rc in (130, -signal.SIGINT, -signal.SIGTERM) for rc in exits_by.values()):
        return 130
    # No binary is lost in silence: every executable Cargo built has its exit.
    if set(exits_by) != set(stamps):
        missing = sorted(set(stamps) - set(exits_by))
        print(f"release lanes: {len(missing)} test executable(s) never ran: {missing}", flush=True)
        return 125
    exits = [exits_by[executable] for executable, _ in targets]
    # --no-run does not discover doctests. Keep Cargo responsible for them.
    exits.append(subprocess.run(wrapper(directory, 32768, 8) +
                 ['cargo', 'test', '--workspace', '--release', '--doc', '--no-fail-fast']
                 + [arg for package in exclude_packages for arg in ('--exclude', package)]
                 + ['--', f'--test-threads={test_threads}', *selection], cwd=ROOT, env=env).returncode)
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
    parser.add_argument('--exclude-package', action='append', default=[],
                        help="a workspace package whose test binaries are left to the booking gate (Loop 3653)")
    args = parser.parse_args()
    return release(args.output.resolve(), args.resource_directory, args.slots, args.test_threads, args.skip,
                   exclude_packages=args.exclude_package)


if __name__ == '__main__':
    sys.exit(main())
