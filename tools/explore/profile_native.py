#!/usr/bin/env python3
"""Sample a prebuilt Rust libtest process on macOS; run under tools/memcap.sh.

Build with CARGO_PROFILE_RELEASE_DEBUG=1 for symbols. Capture files and graphs
are local artifacts; select a new output directory outside the repository.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time


def executable_leaves(sample, binary):
    """The system sampler's self counts, across all threads, for this image."""
    top = sample.partition('Sort by top of stack, same collapsed')[2].partition('Binary Images:')[0]
    rows = re.findall(r'^\s*(.+?)  \(in ' + re.escape(binary.name) + r'\)\s+(\d+) *$', top, re.M)
    names = [name.strip() for name, _ in rows[:5]]
    demangler = shutil.which('llvm-cxxfilt') or shutil.which('c++filt')
    if names and demangler:
        proc = subprocess.run([demangler], input='\n'.join(names)+'\n', capture_output=True, text=True)
        if proc.returncode == 0 and len(proc.stdout.splitlines()) == len(names):
            names = proc.stdout.splitlines()
    return [(name, int(count)) for name, (_, count) in zip(names, rows)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("test", help="exact libtest test name")
    parser.add_argument("output", type=Path)
    parser.add_argument("--with-test", action="append", default=[], help="additional exact test in this same process")
    parser.add_argument("--seconds", type=int, default=90)
    parser.add_argument("--delay-seconds", type=int, default=0, help="sample after this many seconds of test execution")
    parser.add_argument("--interval-ms", type=int, default=5)
    args = parser.parse_args()
    if args.seconds <= 0 or args.interval_ms <= 0:
        parser.error("sampling duration and interval must be positive")
    if args.delay_seconds < 0:
        parser.error("sampling delay must be nonnegative")
    binary = args.binary.resolve(strict=True)
    env = dict(os.environ, RUST_TEST_THREADS="2")
    if not env.get("RON_INSTALL"):
        parser.error("set RON_INSTALL so install-dependent checks do not skip")
    tests = [args.test, *args.with_test]
    listing = subprocess.run(
        [str(binary), "--exact", *tests, "--list", "--format", "terse"],
        env=env, check=True, capture_output=True, text=True,
    )
    listed = {line.removesuffix(": test") for line in listing.stdout.splitlines() if line.endswith(": test")}
    if set(tests) != listed or len(tests) != len(listed):
        parser.error("selection must list each requested test exactly once")
    args.output.mkdir(parents=True, exist_ok=False)
    if audit := env.get("RON_FIXTURE_AUDIT_DIR"):
        Path(audit).mkdir(parents=True, exist_ok=True)
    start = time.monotonic()
    with (args.output / "test.log").open("w") as log:
        test = subprocess.Popen(
            [str(binary), "--exact", *tests, "--nocapture", "--test-threads=1"],
            env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        try:
            if args.delay_seconds:
                try:
                    test.wait(timeout=args.delay_seconds)
                except subprocess.TimeoutExpired:
                    pass
                else:
                    raise RuntimeError("test finished before the requested sample interval")
            sampled = subprocess.run(
                ["/usr/bin/sample", str(test.pid), str(args.seconds),
                 str(args.interval_ms), "-mayDie", "-fullPaths", "-file",
                 str(args.output / "sample.txt")], capture_output=True, text=True,
            )
            (args.output / "sample-command.log").write_text(
                sampled.stdout + sampled.stderr)
            sample_path = args.output / "sample.txt"
            if (sampled.returncode or not sample_path.is_file()
                    or "Call graph:" not in sample_path.read_text()):
                raise RuntimeError("sampling failed; see sample-command.log")
            code = test.wait()
        finally:
            if test.poll() is None:
                test.kill()
                test.wait()
    elapsed = time.monotonic() - start
    output = (args.output / "test.log").read_text()
    leaves = executable_leaves(sample_path.read_text(), binary)
    for name, count in leaves:
        print(f"  {count:6d} self samples in executable  {name}")
    result = {"test": args.test, "tests": tests, "binary": str(binary), "test_exit": code,
              "sample_exit": sampled.returncode,
              "elapsed_s": elapsed,
              "interval_ms": args.interval_ms, "max_sample_seconds": args.seconds,
              "delay_seconds": args.delay_seconds, "executable_leaves": leaves}
    (args.output / "result.json").write_text(json.dumps(result, indent=2))
    print(json.dumps(result))
    if code or not re.search(rf"^test result: ok\. {len(tests)} passed; 0 failed;", output, re.M):
        raise RuntimeError("selected process did not report every requested test passed")


if __name__ == "__main__":
    main()
