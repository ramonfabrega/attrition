#!/usr/bin/env python3
"""Render one named test thread, including I/O waits, from macOS sample output.

Requires an external checkout of brendangregg/FlameGraph and llvm-cxxfilt.
The upstream collapser normally removes some waiting stacks; keep them here
so the graph's denominator is all samples from the selected test thread.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import re
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sample", type=Path)
    parser.add_argument("test", help="name in the sampled thread's label")
    parser.add_argument("output", type=Path, help="new output directory")
    parser.add_argument("--flamegraph", required=True, type=Path)
    parser.add_argument("--cxxfilt", required=True, type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    with tempfile.NamedTemporaryFile(mode="w", suffix=".awk") as waits:
        waits.write("BEGIN { for (key in _IGNORE) delete _IGNORE[key] }\n")
        waits.flush()
        collapsed = subprocess.run(
            ["awk", "-f", str(args.flamegraph / "stackcollapse-sample.awk"),
             "-f", waits.name, str(args.sample)],
            check=True, capture_output=True, text=True,
        ).stdout
    # Demangle complete Rust symbols before rendering; don't truncate generic
    # names at whitespace or change the counts emitted by the collapser.
    symbols = sorted(set(re.findall(r"_R[A-Za-z0-9_]+", collapsed)))
    demangled = subprocess.run(
        [str(args.cxxfilt)], input="\n".join(symbols) + "\n",
        check=True, capture_output=True, text=True,
    ).stdout.splitlines() if symbols else []
    if len(symbols) != len(demangled):
        raise ValueError("demangler changed symbol line count")
    names = dict(zip(symbols, (name.replace(";", ":") for name in demangled)))
    collapsed = re.sub(r"_R[A-Za-z0-9_]+", lambda m: names[m[0]], collapsed)
    rows = [row for row in collapsed.splitlines()
            if args.test in row.split(";", 1)[0]]
    if not rows:
        raise ValueError("no matching test thread; inspect the raw sample labels")
    inclusive, leaves = Counter(), Counter()
    total = 0
    for row in rows:
        stack, count = row.rsplit(" ", 1)
        count = int(count)
        if count < 0:
            raise ValueError("negative stack weight: unsupported sample shape")
        total += count
        frames = stack.split(";")
        leaves[frames[-1]] += count
        for frame in set(frames):
            inclusive[frame] += count
    if not total:
        raise ValueError("selected thread contains no samples")
    folded = args.output / "test.folded"
    folded.write_text("\n".join(rows) + "\n")
    summary = {"samples": total, "inclusive": inclusive.most_common(),
               "self": leaves.most_common()}
    (args.output / "hotspots.json").write_text(json.dumps(summary, indent=2))
    with (args.output / "flamegraph.svg").open("w") as output:
        subprocess.run(
            ["perl", str(args.flamegraph / "flamegraph.pl"), "--title",
             "Capture test thread: all sampled states", "--width", "1600",
             str(folded)], check=True, stdout=output,
        )
    print(f"{total} test-thread samples; {args.output / 'flamegraph.svg'}")


if __name__ == "__main__":
    main()
