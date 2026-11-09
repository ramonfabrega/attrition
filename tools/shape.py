#!/usr/bin/env python3
"""shape.py — the structure track's number, counted on the tree.

    python3 tools/shape.py            # the queue's `Structure:` line
    python3 tools/shape.py --exact    # the same six counts, unrounded
    python3 tools/shape.py --json

`docs/STRUCTURE.md` §2 names six counts, each of which may only fall, and
the queue's `Structure:` line carries them in this order:

    flat · test-tails · sweep-lines · names · fields · largest

Each is defined in its own function's docstring, exactly as §2 defines it
and as the seventy-fourth pass's inventories measured it at `db6c12985`
(their scripts are quoted where the definition is theirs). Two counts are
printed in thousands to one decimal (`65.2k`); `--exact` prints every
count whole, and `rondata::diff::floors::STRUCTURE` pins the exact ones,
so the line cannot hide a move that changed a count by less than a
rounding step.

The seventy-fifth pass (2026-11-09) ruled on two of them: `largest`
counts lines of code, so the rows inside an item-level table literal are
left out (`--json` prints them as `tables`, information only, never on
the line), and `flat` counts loose root files, so a `foo.rs` that is the
module file of a sibling `foo/` is not one.

It reads the tree and prints; it decides nothing.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SIM = ROOT / 'crates/sim/src'
RONDATA = ROOT / 'crates/rondata/src'

ORDER = ('flat', 'test-tails', 'sweep-lines', 'names', 'fields', 'largest')
# Printed by `--json` as information; not a count of §2, not on the line.
INFORMATION = ('tables',)
THOUSANDS = ('test-tails', 'sweep-lines')


def lines(path):
    """A file's line count as `wc -l` gives it: newline characters, plus a
    last line that has none."""
    text = path.read_text()
    return text.count('\n') + (0 if text.endswith('\n') or not text else 1)


def in_sweep(path, root):
    return 'sweep' in path.relative_to(root).parts


def flat(sim=SIM):
    """`.rs` files in the flat root of `crates/sim/src` — `lib.rs` and
    every module that has not moved into a directory. Target ≤ 12. A
    `foo.rs` beside a `foo/` directory is that directory's module file,
    its front page, and not a loose file (the seventy-fifth pass's
    ruling: the `mod.rs` → `foo.rs` + `foo/` layout does not move it)."""
    return sum(1 for p in sim.glob('*.rs') if not p.with_suffix('').is_dir())


def test_mounted(sim=SIM):
    """The files a module declares only under `#[cfg(test)]`, relative to
    `sim`, and the directories whose every file is test code: `mod x;` or
    `#[path = "f.rs"] mod x;` after the attribute, in any file outside
    `sweep/`. A child of `lib.rs` or a `mod.rs` resolves beside it, a child
    of `foo.rs` in `foo/` (so `foo.rs`'s `#[cfg(test)] mod tests;` is
    `foo/tests.rs`, the shape §5's third move writes), a `#[path]` beside
    the declaring file. A module's directory goes with it."""
    files, dirs = set(), set()
    for host in sim.rglob('*.rs'):
        if in_sweep(host, sim):
            continue
        here = host.parent
        base = here if host.name in ('lib.rs', 'mod.rs') else here / host.stem
        held = host.read_text().split('\n')
        for i, line in enumerate(held):
            if line.strip() != '#[cfg(test)]':
                continue
            j, path = i + 1, None
            while j < len(held) and held[j].strip().startswith('#['):
                m = re.match(r'\s*#\[path\s*=\s*"([^"]+)"\]', held[j])
                if m:
                    path = m.group(1)
                j += 1
            m = j < len(held) and re.match(
                r'\s*(?:pub(?:\([a-z]+\))?\s+)?mod\s+(\w+)\s*;', held[j])
            if not m:
                continue
            if path:
                files.add((here / path).resolve().relative_to(sim.resolve()))
                continue
            name = m.group(1)
            for cand in (base / f'{name}.rs', base / name / 'mod.rs'):
                if cand.exists():
                    files.add(cand.relative_to(sim))
            if (base / name).is_dir():
                dirs.add((base / name).relative_to(sim))
    return files, dirs


def tail_blocks(text):
    """`(first, last)` line indices of each `#[cfg(test)]` braced `mod`
    block: the attribute, any attributes or comments between, the `mod x {`
    line, and every line to the brace that closes it (counted by braces per
    line, as the inventory's `scratch/tails.py` counted them)."""
    held = text.split('\n')
    out = []
    i = 0
    while i < len(held):
        if held[i].strip() == '#[cfg(test)]':
            j = i + 1
            while j < len(held) and (held[j].strip().startswith('#[')
                                     or held[j].strip().startswith('//')):
                j += 1
            if j < len(held) and re.match(r'\s*(pub(\([a-z]+\))? )?mod \w+\s*\{', held[j]):
                depth, k = 0, j
                while k < len(held):
                    depth += held[k].count('{') - held[k].count('}')
                    if depth == 0 and k > j:
                        break
                    k += 1
                out.append((i, k))
                i = k
        i += 1
    return out


def test_tails(sim=SIM):
    """Lines of inline `#[cfg(test)] mod … { … }` tails inside the
    production files of `crates/sim/src`: every `.rs` file there but those
    `lib.rs` (or a `mod.rs`) mounts only under `#[cfg(test)]` and their
    directories — `sweep/`, the whole test files and the guards, whose
    tails are tests of a test. (The inventory's `scratch/tails.py` counted
    the guards' tails too: 65,172 at `db6c12985`, 214 lines more than
    this.) Target 0 — in sibling `tests.rs` files or the crate's
    `tests/`."""
    files, dirs = test_mounted(sim)
    total = 0
    for p in sorted(sim.rglob('*.rs')):
        rel = p.relative_to(sim)
        if rel in files or any(d in rel.parents for d in dirs):
            continue
        total += sum(k - i + 1 for i, k in tail_blocks(p.read_text()))
    return total


def sweep_lines(sim=SIM):
    """Lines of sweep code under `crates/sim/src`: every `.rs` file under a
    `sweep/` directory there. Target ≤ 60k, outside `src/`."""
    return sum(lines(p) for p in sim.rglob('*.rs') if in_sweep(p, sim))


PUB_CONST = re.compile(r'^\s*pub\s+(?:const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:')


def names(roots=(SIM, RONDATA)):
    """`pub` constant names defined more than once: `pub const NAME:` or
    `pub static NAME:` (plain `pub`, never `pub(crate)`) in
    `crates/sim/src` and `crates/rondata/src`, outside `sweep/`, counted
    once per name that has two or more definitions. Target 15 (the
    per-module ones)."""
    seen = {}
    for root in roots:
        for p in sorted(root.rglob('*.rs')):
            if in_sweep(p, root):
                continue
            for line in p.read_text().split('\n'):
                m = PUB_CONST.match(line)
                if m:
                    seen[m.group(1)] = seen.get(m.group(1), 0) + 1
    return sum(1 for n in seen.values() if n > 1)


FIELD = re.compile(r'^    (?:pub(?:\([a-z]+\))?\s+)?([a-z_][a-z0-9_]*)\s*:')


def fields(sim=SIM):
    """Fields of `Sim`: the lines of `pub struct Sim {` indented exactly
    four spaces that open `name:` (attributes and doc comments are not
    fields), wherever under `crates/sim/src` the struct lives. Target
    ≤ 24."""
    for p in sorted(sim.rglob('*.rs')):
        if in_sweep(p, sim):
            continue
        held = p.read_text().split('\n')
        for i, line in enumerate(held):
            if line.startswith('pub struct Sim {'):
                n = 0
                for body in held[i + 1:]:
                    if body.startswith('}'):
                        return n
                    if FIELD.match(body):
                        n += 1
    raise SystemExit('shape.py: no `pub struct Sim {` under crates/sim/src')


TABLE_OPEN = re.compile(
    r'^(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(?:mut\s+)?'
    r'[A-Za-z_][A-Za-z0-9_]*\s*:.*=\s*&?\[\s*$')
TABLE_CLOSE = re.compile(r'^\];\s*$')


def table_rows(text):
    """Lines strictly inside an item-level `const`/`static` array literal:
    after a column-0 line that opens `= &[` or `= [` (`pub const X: … =
    &[`, `const X: … = [`, `static …`), up to the column-0 `];` that
    closes it. The two bracket lines are not rows; a row's own braces
    and brackets (tuples, struct literals) are the row's. An opener with
    no closer leaves nothing out."""
    held = text.split('\n')
    rows, i = 0, 0
    while i < len(held):
        if TABLE_OPEN.match(held[i]):
            for k in range(i + 1, len(held)):
                if TABLE_CLOSE.match(held[k]):
                    rows += k - i - 1
                    i = k
                    break
        i += 1
    return rows


def production(sim=SIM):
    """The production files of `crates/sim/src`: every `.rs` file there but
    those `lib.rs` (or a `mod.rs`) mounts only under `#[cfg(test)]` and
    their directories (`sweep/`, the whole test files, the guards)."""
    files, dirs = test_mounted(sim)
    for p in sorted(sim.rglob('*.rs')):
        rel = p.relative_to(sim)
        if rel in files or any(d in rel.parents for d in dirs):
            continue
        yield p


def largest(sim=SIM):
    """Lines of code of the largest production file of `crates/sim/src`
    (see `production`): its `wc -l` lines less its `table_rows`. A file's
    own test tail counts: it moves out in §5's third step. The rows of a
    table literal do not (the seventy-fifth pass's ruling: the target is
    readable code, a pinned table is data, and a 4,700-row table split
    into files of 1,000 rows helps nobody). Target ≤ 4,000."""
    return max((lines(p) - table_rows(p.read_text()) for p in production(sim)),
               default=0)


def largest_file(sim=SIM):
    """The file `largest` measured, relative to `sim`."""
    best = max(production(sim),
               key=lambda p: lines(p) - table_rows(p.read_text()))
    return best.relative_to(sim)


def tables(sim=SIM):
    """The table rows `largest` leaves out, summed over every production
    file: information, not one of §2's counts."""
    return sum(table_rows(p.read_text()) for p in production(sim))


def measure(sim=SIM, rondata=RONDATA):
    return {
        'flat': flat(sim),
        'test-tails': test_tails(sim),
        'sweep-lines': sweep_lines(sim),
        'names': names((sim, rondata)),
        'fields': fields(sim),
        'largest': largest(sim),
        'tables': tables(sim),
    }


def thousands(n):
    """`n` in thousands to one decimal, half up, in integers (so
    `rondata`'s twin rounds the same way): `65172` → `65.2k`,
    `65000` → `65k`."""
    tenths = (n + 50) // 100
    whole, frac = divmod(tenths, 10)
    return f'{whole}k' if frac == 0 else f'{whole}.{frac}k'


def line(counts, exact=False):
    parts = []
    for key in ORDER:
        n = counts[key]
        parts.append(f'{key} {n if exact or key not in THOUSANDS else thousands(n)}')
    return 'Structure: ' + ' · '.join(parts)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--exact', action='store_true', help='every count whole')
    ap.add_argument('--json', action='store_true')
    args = ap.parse_args(argv)
    counts = measure()
    if args.json:
        print(json.dumps(counts))
    else:
        print(line(counts, args.exact))
    return 0


if __name__ == '__main__':
    sys.exit(main())
