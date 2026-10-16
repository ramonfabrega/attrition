#!/usr/bin/env python3
"""Cooperative, host-wide admission and sampled memory enforcement (POSIX).

Locks are never unlinked: kernel ownership, inherited by the workload, survives
an abruptly killed supervisor. All cooperating worktrees use the same directory.
The fixed v1 policy is eight CPU slots and 16 GiB reserved RSS. This is not an
OS limit on unrelated applications; sampling can overshoot between observations.
"""
import argparse
import fcntl
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

CPU = 8
MEMORY_MIB = 16384
QUANTUM_MIB = 128
DEFAULT = Path.home() / 'Library/Caches/attrition/resources-v1'


class Lease:
    def __init__(self, directory, cpu, memory_mib):
        if not 1 <= cpu <= CPU or not 1 <= memory_mib <= MEMORY_MIB:
            raise ValueError('reservation exceeds fixed host budget')
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=True, mode=0o700)
        self.cpu = cpu
        self.memory = math.ceil(memory_mib / QUANTUM_MIB)
        self.fds = []

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
        return True

    def close(self):
        # Do not LOCK_UN: an inheriting child may still be using the reservation.
        for fd in self.fds:
            os.close(fd)
        self.fds = []


def tree_rss(root):
    rows = subprocess.check_output(['ps', '-axo', 'pid=,ppid=,rss='], text=True)
    table = {}
    for line in rows.splitlines():
        pid, parent, rss = map(int, line.split())
        table[pid] = (parent, rss)
    if os.getpid() not in table:
        raise RuntimeError('RSS sampler cannot see its own process')
    selected = {root}
    while True:
        children = {p for p, (parent, _) in table.items() if parent in selected}
        expanded = selected | children
        if expanded == selected:
            break
        selected = expanded
    return sum(table[p][1] for p in selected if p in table)


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


def supervise(command, *, directory=DEFAULT, cpu=1, memory_mib=MEMORY_MIB, timeout=1800):
    lease = Lease(directory, cpu, memory_mib)
    child = None
    cancelled = False
    peak = 0
    def cancel(_sig, _frame):
        nonlocal cancelled
        cancelled = True
    previous = {s: signal.signal(s, cancel) for s in (signal.SIGINT, signal.SIGTERM)}
    try:
        probe = subprocess.check_output(['ps', '-o', 'rss=', '-p', str(os.getpid())], text=True)
        if int(probe.strip()) <= 0:
            raise RuntimeError('RSS sampler unavailable')
        while not lease.try_acquire():
            if cancelled:
                return 130
            time.sleep(0.2)
        if cancelled:
            return 130
        env = dict(os.environ, RON_TEST_MEMCAP_GIB=str(math.ceil(memory_mib / 1024)))
        child = subprocess.Popen(command, start_new_session=True, pass_fds=lease.fds, env=env)
        start = time.monotonic()
        try:
            child.wait(timeout=0.02)
        except subprocess.TimeoutExpired:
            pass
        while child.poll() is None:
            if cancelled:
                return 130
            rss = tree_rss(child.pid)
            peak = max(peak, rss)
            if rss > memory_mib * 1024:
                print(f'resource budget: RSS {rss // 1024} MiB exceeds reservation {memory_mib} MiB', file=sys.stderr)
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
                except (OSError, subprocess.SubprocessError):
                    child.kill()
                    child.wait()
                    raise
        finally:
            lease.close()
            for sig, handler in previous.items():
                signal.signal(sig, handler)
            print(f'resource budget: sampled peak {peak // 1024} MiB', file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, default=DEFAULT)
    parser.add_argument('--cpu', type=int, default=1)
    parser.add_argument('--memory-mib', type=int, default=MEMORY_MIB)
    parser.add_argument('--timeout', type=float, default=1800)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command or args.timeout <= 0:
        parser.error('a command and positive timeout are required')
    try:
        return supervise(command, directory=args.directory, cpu=args.cpu,
                         memory_mib=args.memory_mib, timeout=args.timeout)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f'resource budget: {exc}', file=sys.stderr)
        return 125


if __name__ == '__main__':
    sys.exit(main())
