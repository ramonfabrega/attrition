#!/usr/bin/env python3
"""One leader's LEADERDATA block at one frame, flattened.

    leader.py FRAME WHO [file] [--arrays] [--fields a,b,c]

A `LEADERS=9` record is ~10k lines, most of it `[scan]` arrays
(`reg_buildings[64][129]` alone is 8,256 entries), so by default the
scalars are printed one per line and each array is summarised as
`name: {index: value}` over its non-zero entries — which is what a census
check wants to read. `--arrays` prints the arrays whole. `--fields` keeps
only the named keys (array names match their base name).

The record's two layout traps (`docs/ORACLE.md`): `leader_flags` and
`leader_flags2` are printed *before* `BEGIN LEADERDATA` and belong to the
block that follows; nested `BEGIN` blocks (`PERSONALITY`, the encrypted
goods, `SITES`, `MAKELIST`) are prefixed with their block name.
"""
import os
import re
import sys

DEFAULT = os.path.expanduser(
    "~/ron-data/"
    "AppData/Roaming/Microsoft Games/Rise of Nations/Logs/gamelog.txt"
)


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = [a for a in sys.argv[1:] if a.startswith("--")]
    frame, who = int(args[0]), int(args[1])
    path = args[2] if len(args) > 2 else DEFAULT
    arrays = "--arrays" in opts
    fields = None
    for o in opts:
        if o.startswith("--fields="):
            fields = set(o[len("--fields="):].split(","))

    in_frame = False
    pending_flags = []
    block = None  # list of (indent, line) while inside the wanted block
    depth_stack = []
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            s = line.rstrip("\n")
            st = s.strip()
            if st.startswith("BEGIN FRAME"):
                n = int(st.split()[2])
                if in_frame and block is not None:
                    break
                in_frame = n == frame
                continue
            if not in_frame:
                continue
            indent = len(s) - len(s.lstrip())
            if block is None:
                if st.startswith("leader_flags"):
                    pending_flags.append(st)
                    continue
                if st == "BEGIN LEADERDATA":
                    block = []
                    block_indent = indent
                    depth_stack = []
                    header = list(pending_flags)
                    pending_flags = []
                continue
            # Inside a LEADERDATA block.
            if indent <= block_indent and st:
                # The block ended: was it ours?
                if any(l == "who %d" % who for _, l in block):
                    emit(header, block, arrays, fields)
                    return
                block = None
                pending_flags = []
                if st.startswith("leader_flags"):
                    pending_flags.append(st)
                elif st == "BEGIN LEADERDATA":
                    block = []
                    block_indent = indent
                    header = []
                continue
            block.append((indent, st))
        if block is not None and any(l == "who %d" % who for _, l in block):
            emit(header, block, arrays, fields)
            return
    sys.exit("no LEADERDATA for who %d at frame %d" % (who, frame))


def emit(header, block, arrays, fields):
    # Walk with a block-name stack from indentation.
    stack = []  # (indent, name)
    scans = {}  # (prefix, name) -> [values]
    order = []
    out = []
    for h in header:
        out.append(h)
    for indent, st in block:
        while stack and indent <= stack[-1][0]:
            stack.pop()
        if st.startswith("BEGIN "):
            stack.append((indent, st[6:]))
            continue
        prefix = ".".join(n for _, n in stack)
        m = re.match(r"^(\S+?)(\[[^\]]*\])?\s+(.*)$", st)
        if not m:
            continue
        name, sub, val = m.group(1), m.group(2), m.group(3)
        if sub is not None:
            key = (prefix, name)
            if key not in scans:
                scans[key] = []
                order.append(("scan", key))
            scans[key].append(val)
        else:
            order.append(("kv", (prefix, name, val)))
    for kind, item in order:
        if kind == "kv":
            prefix, name, val = item
            if fields and name not in fields:
                continue
            out.append(("%s.%s" % (prefix, name) if prefix else name) + " " + val)
        else:
            prefix, name = item
            if fields and name not in fields:
                continue
            vals = scans[item]
            label = ("%s.%s" % (prefix, name) if prefix else name)
            if arrays:
                out.append(label + "[] " + " ".join(vals))
            else:
                nz = {i: v for i, v in enumerate(vals) if v.strip() not in ("0", "-1", "", "0.000000")}
                out.append("%s[%d] %s" % (label, len(vals), nz if nz else "all zero/-1"))
    print("\n".join(out))


if __name__ == "__main__":
    main()
