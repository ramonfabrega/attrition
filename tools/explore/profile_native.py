#!/usr/bin/env python3
"""Sample one prebuilt Rust libtest test on macOS; run under tools/memcap.sh.

Build with CARGO_PROFILE_RELEASE_DEBUG=1 for symbols. Capture files and graphs
are local artifacts; select a new output directory outside the repository.
"""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("test", help="exact libtest test name")
    parser.add_argument("output", type=Path)
    parser.add_argument("--seconds", type=int, default=90)
    parser.add_argument("--interval-ms", type=int, default=5)
    args = parser.parse_args()
    if args.seconds <= 0 or args.interval_ms <= 0:
        parser.error("sampling duration and interval must be positive")
    binary = args.binary.resolve(strict=True)
    env = dict(os.environ, RUST_TEST_THREADS="2")
    if not env.get("RON_INSTALL"):
        parser.error("set RON_INSTALL so install-dependent checks do not skip")
    listing = subprocess.run(
        [str(binary), "--exact", args.test, "--list", "--format", "terse"],
        env=env, check=True, capture_output=True, text=True,
    )
    if f"{args.test}: test" not in listing.stdout.splitlines():
        parser.error("exact test name was not listed by the binary")
    args.output.mkdir(parents=True, exist_ok=False)
    start = time.monotonic()
    with (args.output / "test.log").open("w") as log:
        test = subprocess.Popen(
            [str(binary), "--exact", args.test, "--nocapture"],
            env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        try:
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
    output = (args.output / "test.log").read_text()
    result = {"test": args.test, "binary": str(binary), "test_exit": code,
              "sample_exit": sampled.returncode,
              "elapsed_s": time.monotonic() - start,
              "interval_ms": args.interval_ms, "max_sample_seconds": args.seconds}
    (args.output / "result.json").write_text(json.dumps(result, indent=2))
    print(json.dumps(result))
    if code or not re.search(r"test result: ok\. 1 passed; 0 failed;", output):
        raise RuntimeError("selected test did not report exactly one pass")


if __name__ == "__main__":
    main()
