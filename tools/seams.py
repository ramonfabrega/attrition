#!/usr/bin/env python3
"""seams.py — what this crate and its documents say they left out, by name.

    python3 tools/seams.py think_scout do_idle group           # by name
    python3 tools/seams.py --item 1214                         # the queue's chain
    python3 tools/seams.py --chain 'Unit::think_scout+0x941 < Unit::think+0x7da'
    python3 tools/seams.py --doors                             # seams a built door outlived

A word's cause has been written down before it was a word, more than once:
a `SEAM` in `crates/sim` or a "not modelled" row in a specification, under
a name the reader of the word's frame did not think to grep. In one tranche
(the twentieth pass, parked 1146, 1159, 1193): `soft_collision`'s
`TRADE_ROUTE` seam held East Indies at 6151, ORDERS §4.4's "not modelled"
region check held it at 6321, and `World::set_blocked_at`'s road clearing
stood as a seam four days after its door was built and held it at 7512.

Given the names on a word's draw chain and its parted fields, this prints:

- every **live** `SEAM` (not struck through) whose comment names one of
  them, or that sits inside a function of that name — flagged `unscanned`
  when it cites the disk for an absence and names no `scan:`;
- every line of a specification that says something is left out — "not
  modelled", "not built", "unbuilt", "simplif…", "not carried", "stub" —
  in a paragraph that names one of them.

`--doors` lists the live seams that name, as missing, a function this crate
now carries: a seam that left an arm out because its door was not built
outlives the door in silence.

It reads the tree and prints; it decides nothing. A name matches as a whole
word, case-blind, with `Class::` stripped: `Unit::think_scout` is
`think_scout`.
"""
import argparse
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SIM = ROOT / 'crates/sim/src'
DOCS = ROOT / 'docs'
QUEUE = DOCS / 'QUEUE.md'

# What a seam says when it cites the disk for an absence: docs_guard's list.
ABSENCES = ('no capture', 'no run', 'never entered', 'no dump', 'not on disk',
            'has not been captured', 'no trace')
LEFT_OUT = re.compile(
    r'not (?:yet )?model+ed|not (?:yet )?built|unbuilt|simplif|not carried|does not carry'
    r'|\bstub\b|left out|not wired|unwired|\ba seam\b'
    # The spellings a gap took in the twenty-fifth and twenty-sixth
    # tranches, each one a word the scan did not print (parked 1482, 1560,
    # 1521, 1561): a block "not loaded", a stub that "reads 1" or "reads
    # empty", an arm argued "unreachable", a function "never asked for",
    # a branch "never reached" or that "no capture reaches", a field
    # "written only by" one writer, a branch "reached through" the fog.
    r'|not loaded|reads (?:0|1|empty|nothing|none)\b|unreachable|not reachable'
    r'|never (?:asked|reached|entered|taken|runs?)\b|no (?:capture|run|trace) reaches'
    r'|\bonly by\b|reached (?:only )?through', re.I)
MISSING = re.compile(
    r'not (?:yet )?built|unbuilt|not carried|does not carry|is not here|until .{0,40}? is built'
    r'|has no\b|not (?:yet )?model+ed', re.I)
# Documents that are not specifications: the story, the ledger, the rules.
NOT_A_SPEC = ('JOURNAL.md', 'QUEUE.md', 'PARKED.md', 'DECISIONS.md', 'RUNS.md')


def bare(name):
    """`Unit::think_scout+0x941` → `think_scout`."""
    name = re.sub(r'\+0x[0-9a-f]+$', '', name.strip().strip('`'))
    return name.split('::')[-1].split('@')[0]


def chain_names(text):
    """The function names of a draw chain as the queue writes one."""
    return [bare(m) for m in re.findall(r'[A-Za-z_][A-Za-z0-9_]*::[A-Za-z_][A-Za-z0-9_]*', text)]


def item_names(queue, item):
    a = queue.index('## The queue\n')
    m = re.search(rf'^{item}\. \*\*.*?(?=^\d{{1,4}}\. \*\*|^## |\Z)', queue[a:], re.S | re.M)
    if not m:
        raise ValueError(f'docs/QUEUE.md books no item {item}')
    # The chain's functions, and not the parted keys beside them: `group`,
    # `order` and `kind` are in every seam of the tree. A name that is one
    # common word (`think`) is left to be passed by hand.
    names = [n for n in chain_names(m.group(0)) if '_' in n or len(n) > 8]
    # The original's own upper-case names — a type (`OILPLATFORM`), a
    # character class (`CHAR_FARM`), a queue mode (`QUEUE_LAST`) — name
    # the paragraph a field's name misses (parked 1560, 1552's cause).
    names += [n for n in re.findall(r'(?<![A-Za-z0-9_])[A-Z][A-Z0-9_]{3,}(?![A-Za-z0-9_])', m.group(0))
              if '_' in n or len(n) > 4]
    return list(dict.fromkeys(names))


def matcher(names):
    names = [n for n in dict.fromkeys(bare(n) for n in names) if n]
    if not names:
        return None
    return re.compile(r'(?<![A-Za-z0-9_])(' + '|'.join(re.escape(n) for n in names)
                      + r')(?![A-Za-z0-9_])', re.I)


def seams(text):
    """`(line, enclosing fn, the seam's text, struck)` for every `SEAM` of a
    source text. A comment block is read whole and split on the word."""
    lines = text.split('\n')
    out = []
    fn = ''
    i = 0
    while i < len(lines):
        s = lines[i].lstrip()
        m = re.match(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)', s)
        if m:
            fn = m.group(1)
        if not s.startswith('//'):
            i += 1
            continue
        start = i
        block = []
        while i < len(lines) and lines[i].lstrip().startswith('//'):
            block.append(lines[i].lstrip().lstrip('/!').strip())
            i += 1
        # A doc comment belongs to the function under it.
        under = ''
        j = i
        while j < len(lines) and (lines[j].lstrip().startswith('#[') or not lines[j].strip()):
            j += 1
        if j < len(lines):
            m = re.match(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)', lines[j].lstrip())
            if m:
                under = m.group(1)
        joined = ' '.join(block)
        # A plural block (`SEAMS:`) is a seam too (parked 1240).
        for m in re.finditer(r'(~~)?SEAMS?\b', joined):
            nxt = re.search(r'SEAMS?\b', joined[m.end():])
            end = m.end() + nxt.start() if nxt else len(joined)
            one = joined[m.start():end].strip()
            struck = bool(m.group(1))
            out.append((start + 1, under or fn, one, struck))
    return out


def left_out(text):
    """`(line, enclosing fn, the comment)` for every comment block of a source
    text that says something is not modelled and carries no `SEAM` (parked
    1253, three reaches): `find_friends`' enhancer arm said so in a plain
    comment, and `seams.py find_friends enhancer` answered "0 live seams".
    A comment is read as the seam it should have been."""
    lines = text.split('\n')
    out = []
    fn = ''
    i = 0
    while i < len(lines):
        s = lines[i].lstrip()
        m = re.match(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)', s)
        if m:
            fn = m.group(1)
        if not s.startswith('//'):
            i += 1
            continue
        start = i
        block = []
        while i < len(lines) and lines[i].lstrip().startswith('//'):
            block.append(lines[i].lstrip().lstrip('/!').strip())
            i += 1
        under = ''
        j = i
        while j < len(lines) and (lines[j].lstrip().startswith('#[') or not lines[j].strip()):
            j += 1
        if j < len(lines):
            m = re.match(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)', lines[j].lstrip())
            if m:
                under = m.group(1)
        joined = re.sub(r'~~.*?~~', '', ' '.join(block))
        if re.search(r'SEAMS?\b', joined) or not LEFT_OUT.search(joined):
            continue
        out.append((start + 1, under or fn, joined.strip()))
    return out


FN_LINE = re.compile(r'(?:pub(?:\([a-z]+\))? )?(?:const )?fn ([a-z0-9_]+)')
DECL_LINE = re.compile(r'(?:pub(?:\([a-z]+\))? )?(?:struct|enum|union) [A-Za-z0-9_]+')


def field_functions(text, field):
    """`{function: 'w' | 'r'}` for every function of a source text that
    writes or reads a field by that name (parked 1340, six reaches in one
    tranche, with 1337): the seam that was the answer sat on the first
    parted field's writer or reader, never on the word's chain. A write
    is `.field =`, `.field +=` and its kin, or `field:` in a struct
    literal; any other `.field` is a read. A declaration (`pub field:
    Type,`) and a comment are neither."""
    f = re.escape(field)
    write = re.compile(rf'\.{f}\s*(?:=[^=]|[-+*/|&^]=|\.(?:push|insert|clear|set|retain|extend|remove|drain|sort|truncate)\b)')
    literal = re.compile(rf'(?<![A-Za-z0-9_.]){f}\s*:')
    read = re.compile(rf'\.{f}(?![A-Za-z0-9_])')
    out = {}
    fn = ''
    # A test module's writes are a fixture's, not the simulation's.
    live = text.split('#[cfg(test)]')[0]
    prev = ''
    # A struct's or enum's declaration ends a function and declares its
    # fields; a `field: value,` line inside one is a declaration, and the
    # same line inside a function is a struct literal's write (parked 1397:
    # `total_time` is written in four places, every one a literal that
    # spans lines, and the scan read each as a declaration).
    decl_indent = None
    for line in live.split('\n'):
        s = line.strip()
        indent = len(line) - len(line.lstrip())
        if decl_indent is not None:
            if s == '}' and indent == decl_indent:
                decl_indent = None
            continue
        if DECL_LINE.match(s):
            fn = ''
            if s.endswith('{'):
                decl_indent = indent
            continue
        m = FN_LINE.match(s)
        if m:
            # A `#[test]` outside a test module is a fixture too.
            fn = '' if prev.startswith('#[test]') else m.group(1)
            prev = s
            continue
        if s:
            prev = s
        if not fn or s.startswith('//'):
            continue
        code = s.split('//')[0]
        if write.search(code) or literal.search(code):
            out[fn] = 'w'
        elif read.search(code) and fn not in out:
            out[fn] = 'r'
    return out


def functions_naming(text, name):
    """The functions of a source text whose code names an identifier."""
    rx = re.compile(rf'(?<![A-Za-z0-9_]){re.escape(name)}(?![A-Za-z0-9_])')
    out, fn = [], ''
    for line in text.split('#[cfg(test)]')[0].split('\n'):
        s = line.strip()
        m = FN_LINE.match(s)
        if m:
            fn = m.group(1)
            continue
        if fn and not s.startswith('//') and rx.search(s.split('//')[0]) and fn not in out:
            out.append(fn)
    return out


def declared_fields(text, struct):
    """The field names a `struct` declaration of that name carries."""
    m = re.search(rf'(?:pub(?:\([a-z]+\))? )?struct {re.escape(struct)}\b[^{{;]*\{{(.*?)\n\}}', text, re.S)
    if not m:
        return []
    return re.findall(r'^\s*(?:pub(?:\([a-z]+\))? )?([a-z][a-z0-9_]*)\s*:', m.group(1), re.M)


def unwritten(struct, root=None):
    """`(fields, unwritten)`: the fields a struct declares, and those no
    non-test line of `crates/sim` writes (parked 1382, 1400 — three reaches:
    `combat::Side`'s, `Muster::library_cities`, `gather_stamp`). A field no
    one writes reads exactly like a modelled one, and nothing says "not
    modelled"."""
    root = root or SIM
    files = [f.read_text() for f in sorted(root.rglob('*.rs'))]
    fields = []
    for text in files:
        fields = declared_fields(text, struct)
        if fields:
            break
    silent = []
    for field in fields:
        if not any(k == 'w' for text in files for k in field_functions(text, field).values()):
            silent.append(field)
    return fields, silent


def unscanned(seam):
    low = seam.lower()
    return any(a in low for a in ABSENCES) and 'scan: `' not in low


def carried():
    """Every function name `crates/sim/src` defines."""
    out = set()
    for f in sorted(SIM.rglob('*.rs')):
        out.update(re.findall(r'\bfn ([a-z0-9_]+)', f.read_text()))
    return out


def snake(name):
    name = bare(name)
    return re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', name).lower()


def doors(root=SIM):
    """Live seams that name, as missing, a function this crate now carries."""
    have = carried()
    out = []
    for f in sorted(root.rglob('*.rs')):
        for line, fn, seam, struck in seams(f.read_text()):
            if struck or not MISSING.search(seam):
                continue
            live = re.sub(r'~~.*?~~', '', seam)
            named = {snake(n) for n in re.findall(r'`([A-Za-z_][A-Za-z0-9_:]*)(?:\(\))?`', live)}
            built = sorted(n for n in named if n in have and n != fn and len(n) > 5)
            if built:
                out.append((f.relative_to(ROOT), line, fn, built, seam))
    return out


def spec_rows(rx, docs=DOCS):
    out = []
    for f in sorted(docs.glob('*.md')):
        if f.name in NOT_A_SPEC:
            continue
        section = ''
        para = []
        start = 0
        # A bold lead-in that ends in a colon and says something is left
        # out heads the list under it (parked 1253, ROADS §9.4): the names
        # sit in the bullets, which are their own paragraph.
        lead = ''
        for n, line in enumerate(f.read_text().split('\n') + [''], 1):
            if line.startswith('#'):
                section = line.lstrip('# ').strip()
                lead = ''
            if line.strip():
                if not para:
                    start = n
                para.append(line.strip())
                continue
            text = re.sub(r'~~.*?~~', '', ' '.join(para))
            is_list = bool(para) and re.match(r'(?:[-*]|\d+\.) ', para[0]) is not None
            para = []
            if not text:
                continue
            if is_list and lead:
                text = lead + ' ' + text
            if LEFT_OUT.search(text) and rx.search(text):
                out.append((f.relative_to(ROOT), start, section, text))
            lead = text if (not is_list and LEFT_OUT.search(text) and text.rstrip('*').endswith(':')) else ''
    return out


EXPORT = Path(os.path.expanduser('~/ghidra-projects/decomp'))
CALL = re.compile(r'\b([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_~][A-Za-z0-9_]*)*)\s*\(')


def export_index(export=EXPORT):
    """`{name: path}` from the decompile export's `INDEX.tsv`
    (`addr<TAB>name<TAB>file`); None when the export is not on this box."""
    index = export / 'INDEX.tsv'
    if not index.exists():
        return None
    out = {}
    for line in index.read_text().splitlines():
        parts = line.split('\t')
        if len(parts) == 3:
            out[parts[1]] = export / parts[2]
    return out


def callees(name, export=EXPORT):
    """The functions `name`'s decompiled body calls, by the export's
    names — the names a seam about a gate the writer calls is written in
    (parked 1560: `find_goody_box` → `goody_item_is_seen`; six landings
    of two tranches sat a step past `--field`'s reach)."""
    index = export_index(export)
    if index is None or name not in index:
        return None
    body = index[name].read_text()
    return sorted({c for c in CALL.findall(body) if c in index and c != name})


def callers(name, export=EXPORT):
    """The functions whose decompiled bodies call `name` (1586's
    `Group::normalize`: its callers nobody counted)."""
    index = export_index(export)
    if index is None or name not in index:
        return None
    needle = re.compile(r'\b' + re.escape(name) + r'\s*\(')
    return sorted(n for n, path in index.items() if n != name and needle.search(path.read_text()))


def clip(text, n):
    return text if len(text) <= n else text[:n - 1].rstrip() + '…'


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('names', nargs='*', help='function, field or constant names')
    ap.add_argument('--item', type=int, help="read the names off a queue item's chain and keys")
    ap.add_argument('--chain', help='a draw chain as the queue writes one')
    ap.add_argument('--field', action='append', default=[],
                    help="a parted field: its writers and readers in `crates/sim` are "
                         "named too, writers first (parked 1340)")
    ap.add_argument('--doors', action='store_true',
                    help='list the live seams that name a function this crate now carries')
    ap.add_argument('--unwritten', metavar='STRUCT',
                    help='list the fields of a `crates/sim` struct that no non-test line writes')
    ap.add_argument('--callees', action='append', default=[], metavar='FN',
                    help="an export name (`Wall::process`): its callees' names are scanned too (parked 1560)")
    ap.add_argument('--callers', action='append', default=[], metavar='FN',
                    help="an export name: its callers' names are scanned too")
    ap.add_argument('--width', type=int, default=260)
    args = ap.parse_args()

    if args.doors:
        rows = doors()
        for f, line, fn, built, seam in rows:
            print(f'{f}:{line}  in `{fn}`  names {", ".join("`%s`" % b for b in built)}')
            print(f'    {clip(seam, args.width)}')
        print(f'{len(rows)} live seams name, as missing, a function this crate carries')
        return 0

    if args.unwritten:
        fields, silent = unwritten(args.unwritten)
        if not fields:
            print(f'no struct `{args.unwritten}` under crates/sim', file=sys.stderr)
            return 2
        print(f'`{args.unwritten}`: {len(fields)} fields, {len(silent)} written by no non-test line'
              + (': ' + ', '.join(silent) if silent else ''))
        return 0

    names = list(args.names)
    try:
        if args.item:
            names += item_names(QUEUE.read_text(), args.item)
    except ValueError as e:
        print(f'seams.py: {e}', file=sys.stderr)
        return 2
    if args.chain:
        names += chain_names(args.chain)
    for field in args.field:
        fns = {}
        for f in sorted(SIM.rglob('*.rs')):
            fns.update(field_functions(f.read_text(), field))
        writers = sorted(n for n, k in fns.items() if k == 'w')
        readers = sorted(n for n, k in fns.items() if k == 'r')
        print(f'field `{field}`: writers ' + (', '.join(writers) or 'none')
              + '; readers ' + (', '.join(readers) or 'none'))
        if not fns:
            # A variant or a constant is not a field (parked 1397's second
            # reach): say where the bare name occurs instead of "none".
            where = sorted({fn for f in sorted(SIM.rglob('*.rs'))
                            for fn in functions_naming(f.read_text(), field)})
            print(f'    no `.{field}`; the name occurs in ' + (', '.join(where) or 'no function'))
        names += [field] + writers + readers
    graph = [(f, 'callees', callees) for f in args.callees] + [(f, 'callers', callers) for f in args.callers]
    for fn, mode, found_by in graph:
        more = found_by(fn)
        if more is None:
            print(f'seams.py: `{fn}` is not in the export at {EXPORT / "INDEX.tsv"}', file=sys.stderr)
            return 2
        print(f'{mode} of `{fn}`: ' + (', '.join(more) or 'none'))
        names += [fn] + more
    rx = matcher(names)
    if rx is None:
        ap.error('no name given: pass names, --item, --chain, --callees or --callers')
    print('names: ' + ', '.join(dict.fromkeys(bare(n) for n in names)))

    found = 0
    for f in sorted(SIM.rglob('*.rs')):
        for line, fn, seam, struck in seams(f.read_text()):
            if struck:
                continue
            live = re.sub(r'~~.*?~~', '', seam)
            hit = rx.search(live) or (fn and rx.fullmatch(fn))
            if not hit:
                continue
            found += 1
            flag = '  unscanned' if unscanned(live) else ''
            print(f'\n{f.relative_to(ROOT)}:{line}  in `{fn}`{flag}')
            print(f'    {clip(live, args.width)}')
    print(f'\n{found} live seams name one of them, or sit in a function of that name')

    plain = 0
    for f in sorted(SIM.rglob('*.rs')):
        for line, fn, text in left_out(f.read_text()):
            if not (rx.search(text) or (fn and rx.fullmatch(fn))):
                continue
            plain += 1
            print(f'\n{f.relative_to(ROOT)}:{line}  in `{fn}`  (a comment, no SEAM)')
            print(f'    {clip(text, args.width)}')
    print(f'\n{plain} comments without the word say something is left out and name one of them')

    rows = spec_rows(rx)
    for f, line, section, text in rows:
        print(f'\n{f}:{line}  {clip(section, 70)}')
        print(f'    {clip(text, args.width)}')
    print(f'\n{len(rows)} paragraphs of a specification say something is left out and name one of them')
    return 0


if __name__ == '__main__':
    sys.exit(main())
