#!/usr/bin/env python3
"""console.py — the original's console vocabulary, re-derived from the install.

    console.py [--install DIR] [--decomp DIR] table     every command, index, help
    console.py [...]                          chat      only what a `cheat ` line reaches
    console.py [...]                          console   only what a `!` line reaches

`ConsoleWin::init_cmds@007e1340` fills `ConsoleWin::commands`, an array of
`ConsCmd` (0x28 bytes: name at +0, help at +0x14, both `String`), by copying
entries out of `int_str_array` — the positional string table the game loads
from `Data/internal_strings.xml`, stride 0x14.  So a command's index, its
name and its help text are all readable without running anything: parse the
assignments out of the decompiled initialiser, divide by the strides, and
index the XML.

`ConsoleWin::parse_cmd@007d6470` matches the first token against
`commands[i].name` for i in 1..N and calls `run_cmd(this, i, args, from_chat,
no_mouse)`.  `run_cmd` opens with

    if (from_chat != 0) goto switchD_007d6c52_caseD_9;
    switch (command_index) { ... }        <- console only
  switchD_007d6c52_caseD_9:
    switch (command_index) { ... }        <- reachable from chat as well

so the two switches partition the vocabulary, and which switch a command's
`case` label sits in is exactly whether `cheat <cmd>` can reach it.  That is
what `chat` and `console` report.

Both inputs live outside the repo: the install (`--install`, or $RON_INSTALL)
and the Ghidra export (`--decomp`, or ~/ghidra-projects/decomp).
"""
import argparse
import os
import re
import sys

CMD_STRIDE = 0x28   # sizeof(ConsCmd)
STR_STRIDE = 0x14   # sizeof(String)
INIT_CMDS = "funcs/ConsoleWin/init_cmds@007e1340.c"
RUN_CMD = "funcs/ConsoleWin/run_cmd@007d6a70.c"
# run_cmd's `if (from_chat != 0) goto ...` and the label it jumps to
CHAT_GOTO = re.compile(r"if \(param_3 != 0\) goto (switchD_\w+);")

ASSIGN = re.compile(
    r"String::operator=\(\s*"
    r"(?:\*\(String \*\*\)&\(this->commands\)\.field_0x10"
    r"|\(String \*\)\(\*\(int \*\)&\(this->commands\)\.field_0x10 \+ (0x[0-9a-f]+|\d+)\))"
    r"\s*,\s*\(String \*\)\(\*\(int \*\)&int_str_array->field_0x10 \+ (0x[0-9a-f]+|\d+)\)\)",
    re.S,
)
# a <STRING/> with no text is still an entry, and skipping one shifts every
# index after it -- this install has eight of them.
STRING = re.compile(r"<STRING\b[^>]*?(?:/>|>(.*?)</STRING>)", re.S)


def strings(install):
    path = os.path.join(install, "Data", "internal_strings.xml")
    with open(path, encoding="utf-8", errors="replace") as fh:
        return [m.group(1) or "" for m in STRING.finditer(fh.read())]


def commands(install, decomp):
    """[(index, name, help)], in command-index order."""
    table = strings(install)
    src = open(os.path.join(decomp, INIT_CMDS)).read()
    slots = {int(dst, 0) if dst else 0: int(off, 0)
             for dst, off in ASSIGN.findall(src)}
    if sorted(slots) != list(range(0, max(slots) + 1, STR_STRIDE)):
        sys.exit("init_cmds: slots are not a contiguous run -- the export moved")
    out = []
    for i in range((max(slots) // STR_STRIDE + 1) // 2):
        name = table[slots[i * CMD_STRIDE] // STR_STRIDE]
        text = table[slots[i * CMD_STRIDE + STR_STRIDE] // STR_STRIDE]
        out.append((i, name, text))
    return out


def reachable(decomp):
    """(console_only, chat_reachable) -- sets of command indices."""
    lines = open(os.path.join(decomp, RUN_CMD)).read().split("\n")
    label = guard = None
    for n, line in enumerate(lines):
        m = CHAT_GOTO.search(line)
        if m:
            label, guard = m.group(1), n
            break
    if label is None:
        sys.exit("run_cmd: no `if (from_chat != 0) goto` -- the export moved")
    split = next(n for n, line in enumerate(lines) if line.startswith(label + ":"))
    case = re.compile(r"^  case (0x[0-9a-f]+|\d+):")
    def cases(lo, hi):
        return {m.group(1) for m in
                (case.match(line) for line in lines[lo:hi]) if m}
    to_int = lambda s: {int(v, 0) for v in s}
    return to_int(cases(guard, split)), to_int(cases(split, len(lines)))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("what", nargs="?", default="table",
                    choices=("table", "chat", "console"))
    ap.add_argument("--install", default=os.environ.get("RON_INSTALL"))
    ap.add_argument("--decomp", default=os.path.expanduser("~/ghidra-projects/decomp"))
    args = ap.parse_args()
    if not args.install:
        sys.exit("no install: pass --install or set RON_INSTALL")

    cmds = commands(args.install, args.decomp)
    console_only, chat = reachable(args.decomp)
    both = console_only & chat
    if both:
        sys.exit(f"run_cmd: {len(both)} commands in both switches -- the export moved")
    if len(console_only | chat) != len(cmds):
        print(f"warning: {len(cmds)} commands, {len(console_only | chat)} case labels",
              file=sys.stderr)

    want = {"table": None, "chat": chat, "console": console_only}[args.what]
    for i, name, text in cmds:
        if want is not None and i not in want:
            continue
        how = "chat" if i in chat else "cons"
        print(f"{i:4} {how}  {name:<16} {text[:96]}")


if __name__ == "__main__":
    main()
