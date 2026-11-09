#!/usr/bin/env python3
"""brief.py — compose a worker's brief from the frame and the queue.

    python3 tools/brief.py 1133 --kind residue --runs run423,run424 \\
        --section 'docs/AI.md §87' --read docs/journal/2026-09-28-item-1066.md \\
        --note-file notes-1133.md > brief-1133.md

A brief is four things, and this writes three of them:

- **the frame** (`tools/brief/frame.md`), which the steering pass writes and
  the brief checklist is held to — what every brief of a kind says;
- **the item, in the queue's own words**, quoted from `docs/QUEUE.md` and
  never retold, with the other open items as the other lanes' words;
- **what is reserved**: run numbers, section numbers and a park slot, the things two
  lanes can take in silence. Code is not fenced (`docs/DECISIONS.md` 55);
- **the commander's own note** (`--note-file`), which is where its judgement
  goes: what this item alone needs, in its own words, as short as it can be.

Kinds: `residue` — a word on the AI track, a pair's or the third map's;
`chapter` — the rules track's; `sweep` — a batch of never-backed functions;
`reading` — the never-entered functions a document cites, read into claims
that each name their falsifying capture (DECISIONS 84); `coverage` — a
pinned lobby's `cover=1` trace; `structure` — a `docs/STRUCTURE.md` §5
move, gated whole; `render` — a `docs/RENDERER.md` §6 item (Loop 4022). A refusal is a `ValueError` and exit 2: an
item the queue does not book has no brief, and a `--tests` filter that matches no test in the
workspace, or only `#[ignore]` ones, has no lane gate (Loop 3818, 3227); one whose every match is
in `crates/sim` is noted on stderr, since a lane that touches no sim path drops it (Loop 3823).

Why (the nineteenth pass, 2026-09-29): a brief was the previous brief,
copied from the job's scratch directory and edited, and it had grown from
4,700 characters to 11,000 in ten days. `tools/explore/test_brief.py`.
"""
import argparse
import functools
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FRAME = ROOT / 'tools/brief/frame.md'
# The frame's size, in characters, and it may only fall (parked 1226, the
# twenty-first pass): a row added by a pass is paid for by one struck.
# 12,959 was the twentieth pass's; the twenty-first left it under 13,000.
FRAME_CEILING = 13_000
QUEUE = ROOT / 'docs/QUEUE.md'

# The frame's sections a kind of brief carries, in the order they are read.
KINDS = {
    'residue': ('opening', 'booked', 'lanes', 'before-reading', 'reading',
                'capture', 'build', 'landing'),
    'chapter': ('opening', 'booked', 'lanes', 'before-reading', 'reading',
                'chapter', 'capture', 'build', 'landing'),
    # A sweep batch (DECISIONS 63 (iv), parked 1629): no word, no booking
    # paragraph — its section says how a fanned batch is cut and merged.
    'sweep': ('opening', 'lanes', 'sweep', 'landing'),
    # A reading lane (DECISIONS 84): the decompile, for the functions its
    # document cites and no trace enters (`blind::NEVER`), into claims each
    # naming the capture that would falsify it. No capture, no packet, no
    # build rows: its section says what a claim carries.
    'reading': ('opening', 'booked', 'lanes', 'reading-item', 'landing'),
    # A coverage lane (the forty-eighth pass, Loop 3416): one pinned lobby
    # retaken at `cover=1` on the click-free pool, checked against its `cover=0`
    # twin and pinned into `blind::TRACES`, so the functions a reading's
    # captures-owed table names leave `blind::NEVER`. No word moves.
    'coverage': ('opening', 'lanes', 'capture', 'coverage-item', 'landing'),
    # A structure move (Loop 4022; `docs/STRUCTURE.md` §1, §5): no behaviour
    # change, every floor unchanged, gated whole; its number is `shape.py`'s.
    'structure': ('opening', 'booked', 'lanes', 'structure', 'landing'),
    # A render item (Loop 4022; `docs/RENDERER.md` §5.4, §6): K of M, the
    # oracle's rungs, and the render tests' preconditions (4018's Loop line).
    'render': ('opening', 'booked', 'lanes', 'render', 'landing'),
}
TRACKS = {'residue': 'the AI track', 'chapter': 'the rules track', 'sweep': 'the sweep track',
          'reading': 'the reading track', 'coverage': 'the coverage track',
          'structure': 'the structure track', 'render': 'the render track'}
# The lane gate's filters every word lane runs (parked 1623): a widening
# puts its window in the coverage driver, so `coverage::` is a word's own.
# And every lobby's `first_parting` (parked 1910, 2000, the thirty-first
# pass): 1937's fix moved Great Sahara and broke the coverage pair's first
# parting 4730 -> 4206, which took a capture to read; the three run in a
# dozen seconds together. A sweep moves no word and runs neither.
# And the held-out battery's floors (DECISIONS 77): a word lane's change can lower an unseen lobby.
# `blind::` in every kind (Loop 2989): the pinned never-executed list moves
# when a document or a sweep cites a function no trace enters, and 2976's
# lane gate (`--tests nations::`) left its re-pin to the booking gate; the
# pin is 0.4 s.
# And the audited guard in every word lane (Loop 3880): a melee or pair word
# that closes whole wants an `audited!` row or an `OWED` line, and 3862's and
# 3873's lane gates never ran it; the test alone, since `diff::audit::` also
# reads the long words' dumps and would add minutes. It reads the lane's own
# `AI_WORDS` pin beside the queue's line (Loop 3998: 3975's lane ran it green
# against the commander's unwritten `w744`), so `audit::` adds nothing here.
ALWAYS_TESTS = {'residue': ('coverage::', 'first_parting', 'battery_floors', 'blind::',
                            'audit::every_whole_scoreboard_word_is_audited'),
                # Loop 4029: `a_golden_widening` holds a closed chapter's
                # widening to its word and its walk's length — chapter 136
                # pinned `(599, 900)` for word 900 and its lane passed.
                'chapter': ('coverage::', 'first_parting', 'battery_floors', 'blind::',
                            'audit::every_whole_scoreboard_word_is_audited',
                            'a_golden_widening'),
                'sweep': ('sweep::', 'blind::'),
                # A reading moves no word and no sweep; its one pin is the
                # blind list it re-pins when it cites a never-entered
                # function (DECISIONS 84).
                'reading': ('blind::',),
                # A coverage capture moves the blind list alone; its word
                # check is `rngcmp.py` against the twin, a script, not a test.
                'coverage': ('blind::',),
                # A structure lane gates whole (`WHOLE`): it names no filter.
                'structure': (),
                # A render lane (the seventy-fifth pass's ruling): the render
                # crate's tests, `rondata`'s `free` and `artdata`, the coverage
                # and census pins. Never `render::`: cargo prints a crate's
                # tests crate-less (`camera::tests::…`), so that filter matches
                # nothing; the crate's modules are named instead.
                'render': ('free::', 'artdata::', 'bh3::', 'bha::', 'camera', 'chunk::', 'draw::',
                           'coverage::', 'census::', 'blind::')}
# The kinds whose lane gate names no filter of its own: no word moves.
NO_WORD = ('sweep', 'reading', 'coverage', 'structure', 'render')
# The kinds whose lane gate is `--lane --whole` (Loop 4022, the seventy-fifth
# pass): a structure move certifies no behaviour change by every test.
WHOLE = ('structure',)
# Rows of the checklist that are the commander's to follow in composing, or
# the pass's, and say nothing to a worker.
NOT_A_BRIEF_S = frozenset()
N = r'\d{1,4}(?!\d)'
# The seams print the brief carries whole; past it, a count (parked 1743).
SEAMS_LINES = 120


def lane_worktree(item):
    """Where ccc cuts item N's lane: `att-N` beside this worktree, or under
    the checkout's `.claude/worktrees` when this is the checkout."""
    base = ROOT.parent if ROOT.parent.name == 'worktrees' else ROOT / '.claude' / 'worktrees'
    return str(base / f'att-{item}')


def seams_print(code, text, item):
    """Loop 3537: a print that is a usage error (`seams.py --item 3529` exited
    2 on `no name given`, and the brief carried argparse's usage block as if
    it were the print, twice in one tranche) is one line that says so and
    names what the lane runs instead."""
    text = text.strip()
    if code != 0:
        last = [line for line in text.split('\n') if line.strip()]
        why = last[-1].strip() if last else 'no output'
        return (f'(seams.py --item {item} exited {code}: {why}. Run `python3 tools/seams.py <the '
                f'chain\'s function names>` or `--field <the first parted field>` yourself.)')
    return text or '(seams.py printed nothing)'


def run_seams(item):
    """`python3 tools/seams.py --item N` on this tree, its stdout and stderr."""
    out = subprocess.run([sys.executable, str(ROOT / 'tools/seams.py'), '--item', str(item)],
                         cwd=ROOT, capture_output=True, text=True, check=False)
    return seams_print(out.returncode, out.stdout + out.stderr, item)


def sections(frame):
    """`{name: body}` for every `## name` of the frame."""
    out = {}
    for m in re.finditer(r'^## ([a-z-]+)\n(.*?)(?=^## |\Z)', frame, re.S | re.M):
        out[m.group(1)] = m.group(2).strip('\n')
    return out


def open_items_text(queue):
    """`{number: the item's text as written}` for the queue's open items."""
    a = queue.index('## The queue\n')
    b = queue.find('\n## ', a + 1)
    body = queue[a:b if b > 0 else len(queue)]
    out = {}
    for m in re.finditer(rf'^({N})\. \*\*.*?(?=^{N}\. \*\*|\Z)', body, re.S | re.M):
        out[int(m.group(1))] = m.group(0).rstrip('\n')
    return out


def open_items(queue):
    """`[(number, headline)]`, the bold opening of each open item."""
    out = []
    for n, text in open_items_text(queue).items():
        head = re.search(r'\*\*(.*?)\*\*', text, re.S)
        out.append((n, ' '.join(head.group(1).split()) if head else ''))
    return out


# A Rust source with its comments, strings and char literals blanked, so a
# brace or a `fn` inside one is not read. The lookaheads are for speed alone.
RUST_NOISE = re.compile(r'(?=[/"\'br])(?://[^\n]*|/\*(?:[^*]|\*(?!/))*\*/|b?r(#*)"[\s\S]*?"\1'
                        r'|b?"(?:\\[\s\S]|[^"\\])*"'
                        r"|b?'(?:\\(?:u\{[0-9a-fA-F]*\}|x..|.)|[^'\\\n])')")
RUST_TOKEN = re.compile(r'(?=[m{}#f])(?:\bmod\s+([A-Za-z_]\w*)\s*\{|(\{)|(\})|#\s*\[\s*(test|ignore)\b'
                        r'|\bfn\s+([A-Za-z_]\w*))')
PATH_MOD = re.compile(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_]\w*)\s*;')
# A `macro_rules!` whose arm repeats `$($NAME:ident ...)` and writes
# `#[test] fn $NAME` (`diff::audit`'s `audited!`): each entry of its
# invocations is a test, named by the ident that opens the entry.
TEST_MACRO = re.compile(r'macro_rules!\s*([A-Za-z_]\w*)\s*\{\s*\(\s*\$\(\s*\$(\w+)\s*:\s*ident'
                        r'(?:(?!macro_rules!)[\s\S])*?#\s*\[\s*test\s*\]\s*fn\s+\$\2\b')


def module_of(path, crate):
    """The module path cargo prints a test file's tests under, crate left out:
    `src/a/b.rs` and `src/a/b/mod.rs` are `a::b`; `lib.rs`, `main.rs`, a
    `src/bin/` file and a `tests/` file are their binary's root."""
    rel = path.relative_to(crate)
    parts = list(rel.with_suffix('').parts)
    if parts[0] == 'tests' or parts[:2] == ['src', 'bin']:
        return []
    parts = parts[1:]
    if parts and parts[-1] in ('lib', 'main', 'mod'):
        parts = parts[:-1]
    return parts


def file_tests(text, prefix):
    """`{full test name: ignored?}` for a file's `#[test]` fns, named the way
    `cargo test -- --list` prints them: the file's module path, every inline
    `mod NAME {` enclosing the fn, the fn."""
    clean = RUST_NOISE.sub(' ', text)
    out, stack, depth, test, ignored = {}, [], 0, False, False
    for m in RUST_TOKEN.finditer(clean):
        if m.group(1):
            depth += 1
            stack.append((m.group(1), depth))
        elif m.group(2):
            depth += 1
        elif m.group(3):
            if stack and stack[-1][1] == depth:
                stack.pop()
            depth -= 1
        elif m.group(4):
            test, ignored = test or m.group(4) == 'test', ignored or m.group(4) == 'ignore'
        else:
            if test:
                name = '::'.join(prefix + [n for n, _ in stack] + [m.group(5)])
                out[name] = out.get(name, False) or ignored
            test = ignored = False
    for macro, _ in TEST_MACRO.findall(clean):
        for call in re.finditer(rf'\b{macro}!\s*\{{([^{{}}]*)\}}', clean):
            for entry in re.findall(r'(?:^|;)\s*([A-Za-z_]\w*)\s*:(?!:)', call.group(1)):
                out.setdefault('::'.join(prefix + [entry]), False)
    return out


@functools.lru_cache(maxsize=None)
def test_fns(crates=None):
    """`{full test name: ignored?}` over the workspace's sources — the names
    `cargo test -- --list` prints, which a lane gate's `--tests` filter
    matches as a substring, and whether each is `#[ignore]` (Loop 3227:
    three briefs named `battery_measure`, which is, and a lane gate exited
    2 on it). A `#[path = ...] mod NAME;` file is named under `NAME` too.
    Read from the source, not from `cargo test -- --list` (Loop 3818: 0.6 s
    against 2.6 s on a built release tree, and minutes on one cargo would
    rebuild); on 2026-11-04 it held all 5,303 names that list printed, the
    doctests aside, and 1,447 more (a `#[path]` file under its own path too,
    a debug-only test), which can only let a filter through."""
    crates = [ROOT / c for c in crates] if crates else sorted(p for p in (ROOT / 'crates').iterdir() if p.is_dir())
    out = {}
    for crate in crates:
        for path in sorted(list((crate / 'src').rglob('*.rs')) + list((crate / 'tests').glob('*.rs'))):
            text = path.read_text(errors='replace')
            mod = module_of(path, crate)
            for name, ign in file_tests(text, mod).items():
                out[name] = out.get(name, False) or ign
            for target, alias in PATH_MOD.findall(text):
                other = (path.parent / target).resolve()
                if other.is_file():
                    for name, ign in file_tests(other.read_text(errors='replace'), mod + [alias]).items():
                        out[name] = out.get(name, False) or ign
    return out


SIM_CRATE = 'sim'


def crate_tests():
    """`{crate: test_fns of that crate alone}` for every crate of the
    workspace — what says which crate's binaries a filter's matches run in
    (Loop 3823); `release_gate.py` reads the same."""
    return {p.name: test_fns((str(p.relative_to(ROOT)),))
            for p in sorted((ROOT / 'crates').iterdir()) if p.is_dir()}


def sim_only(tests, by_crate):
    """The filters whose every live match is a test of `crates/sim` — none
    elsewhere, at least one there (Loop 3823). A lane gate that leaves the
    sim crate to the booking gate (`sim_crate_left`, Loop 3653) runs none
    of their tests; one that matches nothing anywhere is not among them."""
    def live(crate, t):
        return any(t in n and not ignored for n, ignored in by_crate.get(crate, {}).items())
    return [t for t in tests if live(SIM_CRATE, t)
            and not any(live(c, t) for c in by_crate if c != SIM_CRATE)]


def filters(kind, tests):
    """The lane gate's `--tests` for a brief: what the commander names,
    and the kind's own always among them (1623: 1611's lane gate never ran
    the coverage pins its widening moved, and the booking gate was red).
    A filter that matches no test is refused (3818), and so is one whose
    every matching test is `#[ignore]` (3227)."""
    tests = [t for t in (tests or ()) if t]
    if kind in WHOLE:
        if tests:
            raise ValueError(f"a {kind} lane gates --lane --whole, every test: it names no --tests "
                             f"({' '.join(tests)}) (Loop 4022)")
        return []
    if kind not in NO_WORD and not tests:
        raise ValueError("a word's brief names its lane gate's filters: --tests FILTER [FILTER ...]")
    fns = test_fns()
    for t in tests:
        hits = [n for n in fns if t in n]
        if not hits:
            raise ValueError(f"--tests {t} matches no test in the workspace: the lane gate exits 2 on it "
                             "(Loop 3818); name a test or a module that has one")
        if all(fns[n] for n in hits):
            raise ValueError(f"--tests {t} matches only #[ignore] tests ({', '.join(sorted(hits))}): "
                             "the lane gate exits 2 on it; name a live test, or say the measurement is by hand")
    # Loop 3823: a note, not a refusal — 3819's first lane gate exited 2 on
    # `map_lakes` and `terrain_groups`, its branch touching no sim path; the
    # gate now drops such a filter there and names it.
    for t in sim_only(tests, crate_tests()) if tests else ():
        print(f'note: --tests {t} matches tests in crates/sim only; moot on a lane that touches '
              'no sim path (Loop 3823)', file=sys.stderr)
    return tests + [t for t in ALWAYS_TESTS[kind] if t not in tests]


def run_numbers(text):
    """The run numbers a text names: `run423`, `run 423`, `--runs 423`."""
    return set(re.findall(r'\brun ?(\d{3,4})\b', text))


def reserved_once(item, runs, note, briefs):
    """Loop 3176: a run number is reserved by `--runs` and nowhere else.
    A note that names a run the list does not carry is refused — two
    notes offered run 923 "if needed" and a third brief reserved it — and
    a run another item's brief on the box names is refused."""
    mine = {re.sub(r'\D', '', r) for r in runs}
    loose = run_numbers(note) - mine
    if loose:
        raise ValueError(f"the note names run {', '.join(sorted(loose))} and --runs does not: "
                         "a note's run goes through --runs (Loop 3176)")
    for name, text in sorted((briefs or {}).items()):
        if re.search(rf'(?<!\d){item}(?!\d)', name):
            continue  # the item's own earlier brief
        for line in text.splitlines():
            if 'Run numbers' not in line:
                continue
            taken = run_numbers(line) & mine
            if taken:
                raise ValueError(f"run{', run'.join(sorted(taken))} is reserved by {name}; "
                                 "a run number is reserved once (Loop 3176)")


LANE_FLAG = re.compile(r'(?<![\w-])--lane(?![\w-])')


def note_gates_run(note):
    """Loop 3489: `release_gate.py --lane` with neither `--tests` nor
    `--whole` is refused by the gate (exit 2), so a note that briefs one
    hands the lane a gate that cannot run — and a lane whose item moved no
    test ran the booking gate, which forgives nothing. A gate line is read
    as one inline code span (which may wrap) or, outside code, one line."""
    spans = re.findall(r'`([^`]+)`', note)
    spans += re.sub(r'`[^`]*`', ' ', note).splitlines()
    for span in spans:
        if LANE_FLAG.search(span) and '--tests' not in span and '--whole' not in span:
            line = ' '.join(span.split())
            raise ValueError(f"the note's gate `{line}` names --lane with neither --tests nor --whole, "
                             'and the gate refuses it (exit 2): a lane whose item moves no test gates '
                             'with --lane --whole (Loop 3489)')


def compose(item, kind, *, queue, frame, base, model, runs, sections: list, read, note, tests=(),
            park=None, seams=None, briefs=None):
    if kind not in KINDS:
        raise ValueError(f'no such kind of brief: {kind}; one of {sorted(KINDS)}')
    reserved_once(item, runs, note, briefs)
    note_gates_run(note)
    tests = filters(kind, tests)
    items = open_items_text(queue)
    if item not in items:
        raise ValueError(f'docs/QUEUE.md books no item {item}; its open items are {sorted(items)}')
    body = globals()['sections'](frame)
    missing = [n for n in KINDS[kind] if n not in body]
    if missing:
        raise ValueError(f'tools/brief/frame.md has no section {missing}')

    def fill(text):
        if kind in WHOLE:
            text = text.replace('--tests {tests}{widen}', '--whole')
        return (text.replace('{item}', str(item)).replace('{model}', model)
                .replace('{base}', base).replace('{track}', TRACKS[kind])
                .replace('{tests}', ' '.join(tests))
                .replace('{widen}', '' if kind == 'residue' else ' --no-widen'))

    others = [(n, h) for n, h in open_items(queue) if n != item]
    # The other sweep lanes' worktrees, for `census.py --unbacked --claimed`
    # (Loop 2926: two sweep lanes swept one function twice in a tranche,
    # and the commander's note of the other lane's names dropped a line).
    claims = [lane_worktree(n) for n, h in others if kind == 'sweep' and 'sweep' in h.lower()]
    out = [fill(body['opening']), '']
    out += ['## The item, as the queue books it', '', items[item], '']
    if 'booked' in KINDS[kind]:
        out += [fill(body['booked']), '']
    out += ['## Read first', '']
    out += [f'- `docs/QUEUE.md`: the opener, and item {item}.']
    out += [f'- `{r}`' if not r.startswith('`') else f'- {r}' for r in read]
    out += ['- `docs/audit/README.md`, "The brief checklist": the story behind any row '
            'below, by its number, when a row is not clear.', '']
    out += ['## The other lanes', '', fill(body['lanes']), '']
    out += [f'- **{n}**: {h}' for n, h in others] or ['- None is booked.']
    if claims:
        out += ['', '- **The other sweep lane\'s claims**: your batch is `python3 tools/census.py --unbacked '
                + ' '.join(f'--claimed {w}' for w in claims)
                + '`, re-read at every `ccc update` — a function a sweep test on that tree cites is `claimed` '
                  'and hidden, committed or not (Loop 2926: two batches of one tranche built one function twice).']
    out += ['', '## Reserved for you', '']
    out += ['- **Run numbers**: ' + (', '.join(runs) if runs else 'none; ask for one before a capture') + '.']
    out += ['- **Sections**: ' + ('; '.join(sections) if sections else
                                  'amend your documents\' sections in place; name a new '
                                  'section to me before you write it') + '.']
    out += ['- **Park heading**: ' + (
        f'`## Parked by the next landing, slot {park} — reserved`, at the end of `docs/PARKED.md`: '
        f'rename it `## Parked by item {item}, <the day you land> — <title>` and file your parked rows '
        'under it; never append a heading at the file\'s tail (parked 1812: three merges backed out there).'
        if park else 'ask me for a park slot before you park a row; never append a heading at '
                     '`docs/PARKED.md`\'s tail (parked 1812).')]
    if seams is not None and kind in ('residue', 'chapter'):
        lines = seams(item).rstrip('\n').split('\n')
        shown, rest = lines[:SEAMS_LINES], len(lines) - SEAMS_LINES
        out += ['', '## Seams, read for you', '',
                f'`python3 tools/seams.py --item {item}` on the base, so the frame\'s step 7 is on your '
                'desk before you read (parked 1743: the step ran in one journal of nine). Re-run it with '
                '`--field <the first parted field>` once you have one.', '', '```']
        out += shown
        if rest > 0:
            out += [f'… {rest} more lines; run it yourself for the rest']
        out += ['```']
    titles = {'before-reading': 'Before any reading', 'reading': 'Reading the original',
              'capture': 'A capture or a packet', 'chapter': 'A chapter', 'sweep': 'The sweep',
              'reading-item': 'A reading item', 'coverage-item': 'A coverage capture',
              'structure': 'A structure item', 'render': 'A render item',
              'build': 'Build', 'landing': 'Landing'}
    for name in KINDS[kind]:
        if name in ('opening', 'booked', 'lanes'):
            continue
        if name == 'landing' and note.strip():
            out += ['', '## This item', '', note.strip()]
        out += ['', f'## {titles[name]}', '', fill(body[name])]
    return '\n'.join(out) + '\n'


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('item', type=int)
    ap.add_argument('--kind', choices=sorted(KINDS), required=True)
    ap.add_argument('--runs', default='', help='comma-separated run numbers reserved for the item')
    ap.add_argument('--section', action='append', default=[], dest='sections',
                    help='a section reserved for the item, repeatable')
    ap.add_argument('--read', action='append', default=[],
                    help='a document or journal to read first, repeatable')
    ap.add_argument('--note-file', type=Path, help="the commander's own note for this item")
    ap.add_argument('--model', default='Opus 5.5')
    ap.add_argument('--tests', nargs='+', default=[],
                    help="the lane gate's filters for the word; `coverage::` is added (1623); "
                         "a sweep or a reading names none; one that matches no test is refused (3818)")
    ap.add_argument('--park', help='the park slot letter reserved for the item (parked 1812)')
    ap.add_argument('--no-seams', action='store_true', help='leave out the seams.py print (parked 1743)')
    ap.add_argument('--briefs', type=Path, default=Path(os.environ['CLAUDE_JOB_DIR']) / 'tmp' if os.environ.get('CLAUDE_JOB_DIR') else None,
                    help="the directory of the other lanes' briefs (brief*.md), whose run numbers are taken (Loop 3176); "
                         "$CLAUDE_JOB_DIR/tmp when set")
    args = ap.parse_args()
    briefs = {f.name: f.read_text() for f in sorted(args.briefs.glob('brief*.md'))} if args.briefs and args.briefs.is_dir() else {}
    base = subprocess.run(['git', 'rev-parse', '--short=8', 'HEAD'], cwd=ROOT,
                          capture_output=True, text=True, check=True).stdout.strip()
    try:
        text = compose(args.item, args.kind, queue=QUEUE.read_text(), frame=FRAME.read_text(),
                       base=base, model=args.model,
                       runs=[r for r in args.runs.split(',') if r],
                       sections=args.sections, read=args.read,
                       note=args.note_file.read_text() if args.note_file else '',
                       tests=args.tests, park=args.park,
                       seams=None if args.no_seams else run_seams, briefs=briefs)
    except ValueError as e:
        print(f'brief.py: {e}', file=sys.stderr)
        sys.exit(2)
    sys.stdout.write(text)


if __name__ == '__main__':
    main()
