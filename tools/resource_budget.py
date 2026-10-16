#!/usr/bin/env python3
"""Cooperative, host-wide admission and sampled memory enforcement (POSIX).

Locks are never unlinked: kernel ownership, inherited by the workload, survives
an abruptly killed supervisor. All cooperating worktrees use the same directory.
The fixed v1 policy is sixteen CPU slots and 32 GiB reserved RSS. This is not an
OS limit on unrelated applications; sampling can overshoot between observations.
"""
import argparse
import fcntl
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

CPU = 16
MEMORY_MIB = 32768
QUANTUM_MIB = 256
DEFAULT = Path.home() / 'Library/Caches/attrition/resources-v1'


def check_policy(directory):
    policy = dict(schema=1, cpu_slots=CPU, memory_mib=MEMORY_MIB, quantum_mib=QUANTUM_MIB)
    path = directory / 'policy.json'
    with (directory / 'admission.lock').open('a') as mutex:
        fcntl.flock(mutex, fcntl.LOCK_EX)
        if path.exists():
            with path.open() as stream:
                actual = json.loads(stream.read(4096))
            if actual != policy:
                raise ValueError(f'incompatible shared resource policy: {path}')
        else:
            temporary = directory / f'.policy-{os.getpid()}-{time.monotonic_ns()}'
            with temporary.open('x') as stream:
                json.dump(policy, stream)
                stream.flush()
            try:
                os.link(temporary, path)
            finally:
                temporary.unlink()


class Lease:
    def __init__(self, directory, cpu, memory_mib):
        if not 1 <= cpu <= CPU or not 1 <= memory_mib <= MEMORY_MIB:
            raise ValueError('reservation exceeds fixed host budget')
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=True, mode=0o700)
        check_policy(self.directory)
        self.cpu = cpu
        self.memory = math.ceil(memory_mib / QUANTUM_MIB)
        self.fds = []
        self.identity = f'{os.getpid()}-{time.monotonic_ns()}'

    def describe(self, child=None):
        with (self.directory / 'admission.lock').open('a') as mutex:
            fcntl.flock(mutex, fcntl.LOCK_EX)
            self._describe(child)

    def _describe(self, child=None):
        data = json.dumps(dict(lease=self.identity, supervisor_pid=os.getpid(), child_pid=child,
                               cpu=self.cpu, memory_mib=self.memory * QUANTUM_MIB)).encode()
        for fd in self.fds[:self.cpu]:
            os.ftruncate(fd, 0)
            if os.pwrite(fd, data, 0) != len(data):
                raise OSError('incomplete lease metadata write')

    def try_acquire(self):
        if self.fds:
            raise RuntimeError('lease already acquired')
        with (self.directory / 'admission.lock').open('a') as mutex:
            fcntl.flock(mutex, fcntl.LOCK_EX)
            for kind, needed, total in [('cpu', self.cpu, CPU),
                                        ('memory', self.memory, MEMORY_MIB // QUANTUM_MIB)]:
                found = 0
                for i in range(total):
                    fd = os.open(self.directory / f'{kind}-{i}.lock', os.O_RDWR | os.O_CREAT, 0o600)
                    try:
                        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    except BlockingIOError:
                        os.close(fd)
                        continue
                    self.fds.append(fd)
                    found += 1
                    if found == needed:
                        break
                if found != needed:
                    self.close()
                    return False
            self._describe()
        return True

    def close(self):
        # Do not LOCK_UN: an inheriting child may still be using the reservation.
        for fd in self.fds:
            os.close(fd)
        self.fds = []


def status(directory=DEFAULT):
    """Report only kernel-held slots; stale file contents never imply ownership."""
    try:
        mutex = (Path(directory) / 'admission.lock').open('r')
    except FileNotFoundError:
        return []
    with mutex:
        fcntl.flock(mutex, fcntl.LOCK_SH)
        return _status(directory)


def _status(directory):
    held = []
    for index in range(CPU):
        try:
            fd = os.open(Path(directory) / f'cpu-{index}.lock', os.O_RDONLY)
        except FileNotFoundError:
            continue
        try:
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError:
                try:
                    owner = json.loads(os.pread(fd, 4096, 0))
                except (ValueError, UnicodeError):
                    owner = None
                held.append(dict(slot=index, owner=owner))
        finally:
            os.close(fd)
    return held


def rss_for_roots(table, roots):
    """Count each process once, including orphaned members of owned groups."""
    selected = set(roots) | {pid for pid, (_, group, _) in table.items() if group in roots}
    children = {}
    for pid, (parent, _, _) in table.items():
        children.setdefault(parent, []).append(pid)
    pending = list(selected)
    while pending:
        for child in children.get(pending.pop(), []):
            if child not in selected:
                selected.add(child)
                pending.append(child)
    return sum(table[p][2] for p in selected if p in table)


def sample_rss(root, directory):
    rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,pgid=,rss='], text=True)
    table = {}
    for line in rows.splitlines():
        pid, parent, group, rss = map(int, line.split())
        table[pid] = (parent, group, rss)
    if os.getpid() not in table:
        raise RuntimeError('RSS sampler cannot see its own process')
    roots = {root}
    for slot in status(directory):
        owner = slot['owner']
        if (not isinstance(owner, dict) or type(owner.get('supervisor_pid')) is not int
                or owner['supervisor_pid'] <= 0
                or (owner.get('child_pid') is not None
                    and (type(owner['child_pid']) is not int or owner['child_pid'] <= 0))):
            raise RuntimeError('cannot monitor a held lease with invalid owner metadata')
        if owner.get('child_pid'):
            roots.add(owner['child_pid'])
        elif owner.get('supervisor_pid') not in table:
            raise RuntimeError('cannot locate the workload of an orphaned lease')
    return rss_for_roots(table, {root}), rss_for_roots(table, roots)


def stop_group(child):
    # macOS may deny killpg even for a session created by this process. Select
    # only our child's new process group, then signal its individual members,
    # as the established memcap does. Never signal by executable name.
    found = subprocess.run(['pgrep', '-g', str(child.pid)], capture_output=True, text=True)
    if found.returncode not in (0, 1):
        raise RuntimeError('cannot enumerate owned process group')
    members = [int(pid) for pid in found.stdout.split()]
    for pid in members:
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    child.wait()


def supervise(command, *, directory=DEFAULT, cpu=1, memory_mib=MEMORY_MIB, limit_mib=None, timeout=1800):
    if not math.isfinite(timeout) or timeout <= 0:
        raise ValueError('execution timeout must be positive and finite')
    limit_mib = memory_mib if limit_mib is None else limit_mib
    if not memory_mib <= limit_mib <= MEMORY_MIB:
        raise ValueError('RSS limit must cover the reservation and fit the host budget')
    lease = Lease(directory, cpu, memory_mib)
    child = None
    cancelled = False
    peak = combined_peak = 0
    requested = time.monotonic()
    admitted = None
    def cancel(_sig, _frame):
        nonlocal cancelled
        cancelled = True
    previous = {s: signal.signal(s, cancel) for s in (signal.SIGINT, signal.SIGTERM)}
    try:
        probe = subprocess.check_output(['ps', '-o', 'rss=', '-p', str(os.getpid())], text=True)
        if int(probe.strip()) <= 0:
            raise RuntimeError('RSS sampler unavailable')
        waiting = False
        while not lease.try_acquire():
            if not waiting:
                print(f'resource budget: supervisor {os.getpid()} waiting for shared admission', file=sys.stderr)
                waiting = True
            if cancelled:
                return 130
            time.sleep(0.2)
        if cancelled:
            return 130
        admitted = time.monotonic()
        env = dict(os.environ, RON_TEST_MEMCAP_GIB=str(math.ceil(limit_mib / 1024)))
        child = subprocess.Popen(['/usr/bin/nice', '-n', '10', *command],
                                 start_new_session=True, pass_fds=lease.fds, env=env)
        lease.describe(child.pid)
        start = time.monotonic()
        try:
            child.wait(timeout=0.02)
        except subprocess.TimeoutExpired:
            pass
        while child.poll() is None:
            if cancelled:
                return 130
            rss, combined = sample_rss(child.pid, directory)
            peak = max(peak, rss)
            combined_peak = max(combined_peak, combined)
            if rss > limit_mib * 1024 or combined > MEMORY_MIB * 1024:
                print(f'resource budget: RSS {rss // 1024} MiB / limit {limit_mib} MiB; '
                      f'combined {combined // 1024} MiB / host limit {MEMORY_MIB} MiB — stopping owned workload', file=sys.stderr)
                return 137
            if time.monotonic() - start > timeout:
                print(f'resource budget: execution timeout {timeout}s', file=sys.stderr)
                return 124
            try:
                child.wait(timeout=min(2.0, max(0.01, timeout - (time.monotonic() - start))))
            except subprocess.TimeoutExpired:
                pass
        return child.returncode if child.returncode >= 0 else 128 - child.returncode
    finally:
        try:
            if child is not None:
                # Also remove descendants that outlived their direct parent.
                try:
                    stop_group(child)
                except (OSError, RuntimeError, subprocess.SubprocessError):
                    child.kill()
                    child.wait()
                    raise
        finally:
            lease.close()
            for sig, handler in previous.items():
                signal.signal(sig, handler)
            finished = time.monotonic()
            timing = dict(wait_seconds=(admitted if admitted is not None else finished) - requested,
                          run_seconds=finished - admitted if admitted is not None else 0,
                          admitted=admitted is not None, cpu_slots=cpu, reservation_mib=memory_mib)
            print('resource budget timing: ' + json.dumps(timing), file=sys.stderr)
            print(f'resource budget: sampled peak {peak // 1024} MiB; combined peak {combined_peak // 1024} MiB', file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=DEFAULT)
    parser.add_argument('--status', action='store_true', help='show kernel-held CPU slots and reported owners')
    parser.add_argument('--cpu', type=int, default=1)
    parser.add_argument('--memory-mib', type=int, default=MEMORY_MIB)
    parser.add_argument('--limit-mib', type=int, help='hard sampled RSS ceiling; defaults to reservation')
    parser.add_argument('--timeout', type=float, default=1800)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.status:
        print(json.dumps(status(args.directory), indent=2))
        return 0
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command or args.timeout <= 0:
        parser.error('a command and positive timeout are required')
    try:
        return supervise(command, directory=args.directory, cpu=args.cpu,
                         memory_mib=args.memory_mib, limit_mib=args.limit_mib, timeout=args.timeout)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f'resource budget: {exc}', file=sys.stderr)
        return 125


if __name__ == '__main__':
    sys.exit(main())
