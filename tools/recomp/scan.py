#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["capstone==5.0.3"]
# ///
"""scan.py — how many of a set of functions lift mechanically, and why the rest do not.

    uv run tools/recomp/scan.py <install>/riseofnations.exe [--out DIR] (--docs DIR | <va>…)

`--docs DIR` takes every `@<8 hex>` citation in DIR's top-level .md files —
the addresses the specification cites, read the way the paperwork guard
reads them. Each function is lifted on its own (its callees are named, not
followed) and, when it lifts, compiled on its own to an object with clang.
The report is the share that lifted and compiled, and the failures grouped
by the kind of the first instruction the lifter refused. The per-function
rows go to `<out>/scan.tsv`; `<out>` defaults to `target/recomp/`.
"""
import concurrent.futures
import glob
import os
import re
import subprocess
import sys
from collections import Counter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from image import Functions, Image  # noqa: E402
from lift import RT, LiftError, Lifter  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
CITE = re.compile(r"@([0-9a-f]{8})\b")
X87 = re.compile(r"^f(ld|st|i|add|sub|mul|div|com|xch|abs|chs|sqrt|n?st|prem|rndint|scale|sin|cos|ptan|patan|yl2x|2xm1|xam|tst|ucom|cmov|free|incstp|decstp|wait|clex|init|save|rstor|xsave|xrstor|emms)")
SSE = re.compile(r"(ss|sd|ps|pd)$|^(movd|movq|movdqa|movdqu|movaps|movups|cvt|ucomis|comis|p[a-z]+|pxor|pand|por|ldmxcsr|stmxcsr|sqrt|rsqrt|rcp|shuf|unpck|andn?p|orp|xorp|maxp|minp)")


def kind(err):
    """The failure's kind, from the lifter's message `<addr>: <mn> <ops>[ — why]`."""
    msg = str(err)
    if "segment-relative" in msg:
        return "fs: (TIB / SEH)"
    if "runs past" in msg:
        return "bounds (runs into the next function)"
    if "undecodable" in msg:
        return "undecodable"
    if "looping" in msg:
        return "discovery looping"
    if "only the 32-bit form" in msg:
        return "8/16-bit mul, div or push"
    head = msg.split(": ", 1)[1] if ": " in msg else msg
    mn = head.split(" — ")[0].split(" ")[0]
    if "es:[" in head:
        return "string op (rep stos / movs)"
    if X87.match(mn):
        return "x87"
    if SSE.search(mn) or "xmm" in head:
        return "SSE"
    return f"int: {mn}"


def compile_one(out_dir, entry, c_text):
    path = os.path.join(out_dir, "scan", f"f_{entry:08x}.c")
    with open(path, "w") as f:
        f.write(c_text)
    r = subprocess.run(["clang", "-std=c11", "-O1", "-c", "-w", "-I", RT, "-o", "/dev/null", path],
                       capture_output=True, text=True)
    return entry, r.returncode == 0, r.stderr.strip().splitlines()[:1]


def main(argv):
    args = argv[1:]
    out_dir = os.path.join(HERE, "..", "..", "target", "recomp")
    if "--out" in args:
        k = args.index("--out")
        out_dir = args[k + 1]
        del args[k:k + 2]
    if len(args) < 2:
        print(__doc__)
        return 2
    exe = args[0]
    image = Image(exe)
    if args[1] == "--docs":
        cited = set()
        for md in glob.glob(os.path.join(args[2], "*.md")):
            for m in CITE.finditer(open(md, encoding="utf-8").read()):
                cited.add(int(m.group(1), 16))
        entries = sorted(a for a in cited if image.is_code(a))
        print(f"{len(cited)} addresses cited in {args[2]}, {len(entries)} in .text")
    else:
        entries = [int(a, 16) for a in args[1:]]
    funcs = Functions(image, os.path.dirname(os.path.abspath(exe)))
    os.makedirs(os.path.join(out_dir, "scan"), exist_ok=True)

    lifted, failed = {}, {}
    for e in entries:
        lifter = Lifter(image, funcs)
        try:
            callees = lifter.lift(e)
            protos = "".join(f"void f_{c:08x}(cpu_t *c);\n" for c in sorted(callees | {e}))
            lifted[e] = '#include "recomp.h"\n#include <stdint.h>\n' + protos + lifter.lifted[e] + "\n"
        except LiftError as err:
            failed[e] = err
    compiled, cerr = {}, {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as pool:
        for e, ok, lines in pool.map(lambda e: compile_one(out_dir, e, lifted[e]), lifted):
            compiled[e] = ok
            if not ok:
                cerr[e] = lines[0] if lines else "?"

    with open(os.path.join(out_dir, "scan.tsv"), "w") as f:
        f.write("va\tname\tlifted\tcompiled\tfailure\n")
        for e in entries:
            if e in failed:
                f.write(f"{e:08x}\t{funcs.name(e)}\tno\t-\t{failed[e]}\n")
            else:
                f.write(f"{e:08x}\t{funcs.name(e)}\tyes\t{'yes' if compiled[e] else 'no'}\t{cerr.get(e, '')}\n")

    n = len(entries)
    ok = sum(compiled.values())
    print(f"functions: {n}   lifted: {len(lifted)} ({100 * len(lifted) / n:.0f}%)   lifted and compiled: {ok} ({100 * ok / n:.0f}%)")
    kinds = Counter(kind(err) for err in failed.values())
    print("failures by kind:")
    for k, v in kinds.most_common():
        print(f"  {v:5d}  {k}")
    if cerr:
        print("compile errors:")
        for e, line in cerr.items():
            print(f"  {e:08x} {funcs.name(e)}: {line}")
    print(f"rows: {os.path.join(out_dir, 'scan.tsv')}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
