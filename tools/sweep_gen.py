#!/usr/bin/env python3
"""sweep_gen.py — a sweep's two files from one line, for the shapes the
sweep lanes wrote by hand four times (parked 1706; CLAUDE.md, "a probe
shape reached for a third time graduates into `tools/`").

    python3 tools/sweep_gen.py --item 1736 const 'FollowOrder::get_type@00486c20' \\
        --port 'i64::from(crate::orders::Kind::Follow as i32)'
    python3 tools/sweep_gen.py --item 1736 field 'ObjectData::has_range@00646c10' \\
        --via 0x18 --off 0x1fc --port 'i64::from(crate::combat::has_range(v))'
    python3 tools/sweep_gen.py --item 1736 range 'TypeData::is_bonus_type@004705b0' \\
        --port 'crate::type_index::is_bonus_type(t)'

Three shapes, each a `__thiscall` on `ecx`:

  const   the function reads nothing and answers a constant; the rows vary
          `this` to show it is unread, and the test holds the answer to
          `--port`, an `i64` expression.
  field   the function reads one integer at `this+OFF` — or, with `--via`,
          at `(*(this+VIA))+OFF`, the type record a unit points to; the
          rows are the signed 32-bit edges and two hundred random words,
          and the test holds `--port`, an `i64` expression over `v: i32`.
  range   the function tests the type record's index (`+0x4`, or `--off`)
          against a range; the rows are every index -2..=0x33f and the
          edges, and the test holds `--port`, a `bool` over `t: i32`.

Written: `tools/emu/sweep_<snake>.py` and `crates/sim/src/sweep/<snake>.rs`;
the `mod <snake>;` line for `sweep/mod.rs` is printed, or appended with
`--write-mod`. Without `--port` the test's port is `todo!()`, so the row
fails until the lane writes it — a sweep lands when the two sides agree,
never when a file exists. `--dry-run` prints both files instead.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EMU = ROOT / 'tools/emu'
SWEEP = ROOT / 'crates/sim/src/sweep'
NAME = re.compile(r'^([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_~][A-Za-z0-9_<>=!\[\]+\-*/%&|^]*)+)@(00[0-9a-f]{6})$')


def snake(qualified):
    """`TypeData::is_bonus_type` → `type_data_is_bonus_type`."""
    words = []
    for part in qualified.split('::'):
        part = re.sub(r'[^A-Za-z0-9_]+', '_', part).strip('_')
        words.append(re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', part).lower())
    return '_'.join(w for w in words if w)


def parse(spec):
    m = NAME.match(spec)
    if not m:
        raise ValueError(f'a function is `Class::method@00xxxxxx`, not `{spec}`')
    return m.group(1), int(m.group(2), 16)


SCRIPT_HEAD = '''#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, `ecx` = {what}.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x{va:08x}
OBJ, TYPE = 0x10000000, 0x10100000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)
'''

SCRIPT_TAIL = '''

def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
{write}
        r = signed(m.call(ENTRY, (), ecx={ecx}))
        print(" ".join(str(x) for x in row) + f" -> {{r}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

ROWS = {
    'const': '''

def rows():
    return [(OBJ + d,) for d in (0, 4, 0x100, 0x1FFFC, 0x8000)]
''',
    'field': '''

def rows():
    rng = random.Random({item})
    out = [(v,) for v in INT_EDGES]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF),) for _ in range(200)]
    return out
''',
    'range': '''

def rows():
    return [(i,) for i in range(-2, 0x340)] + [(-1000,), (0x7FFFFFFF,), (-0x80000000,), (0x10000,)]
''',
}

TEST = '''//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, {shape}).

/// `{name}@{va:08x}`, run by `tools/emu/sweep_{snake}.py` under unicorn
/// (tools/emu): {doc}
#[test]
fn the_emulated_original_agrees_on_every_row() {{
    let Some(rows) = super::emu_rows("sweep_{snake}.py") else {{
        return;
    }};
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (a, want) = super::row(line);
{body}
        n += 1;
    }}
    assert!(n >= {least}, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
'''

BODIES = {
    'const': ('''        let _this = a[0];
        let got: i64 = {port};
        assert_eq!(got, want, "{short} — row `{{line}}`");''', 5),
    'field': ('''        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        let got: i64 = {port};
        assert_eq!(got, want, "{short} — row `{{line}}`");''', 200),
    'range': ('''        let t = i32::try_from(a[0]).expect("an int");
        let got: bool = {port};
        assert_eq!(got, want != 0, "{short} — row `{{line}}`");''', 800),
}


def generate(shape, spec, *, item, port=None, off=None, via=None):
    """`(script path, script text, test path, test text, mod line)`."""
    name, va = parse(spec)
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    port = port or f'todo!("the port of {name}")'
    if shape == 'const':
        what = 'the object (never read)'
        write = '        pass'
        ecx = 'row[0]'
        doc = f'answers a constant whatever `this` is, held to `{port}`.'
    elif shape == 'field':
        off = 0 if off is None else off
        if via is None:
            what = f'the object; reads the int at this+0x{off:x}'
            write = f'        uc.mem_write(OBJ + 0x{off:x}, struct.pack("<i", row[0]))'
        else:
            what = f'the object; reads the int at type+0x{off:x}, the type at this+0x{via:x}'
            write = (f'        uc.mem_write(OBJ + 0x{via:x}, struct.pack("<I", TYPE))\n'
                     f'        uc.mem_write(TYPE + 0x{off:x}, struct.pack("<i", row[0]))')
        ecx = 'OBJ'
        doc = f'the signed-32 edges and two hundred random words at the field, against `{port}`.'
    elif shape == 'range':
        off = 4 if off is None else off
        what = f'the type record; a range test on `type` (+0x{off:x}), signed'
        write = f'        uc.mem_write(TYPE + 0x{off:x}, struct.pack("<i", row[0]))'
        ecx = 'TYPE'
        doc = f'every index -2..=0x33f and the edges, against `{port}`.'
    else:
        raise ValueError(f'no such shape: {shape}; one of const, field, range')
    script = (SCRIPT_HEAD.format(snake=sn, name=name, va=va, item=item, what=what)
              + ROWS[shape].format(item=item)
              + SCRIPT_TAIL.format(write=write, ecx=ecx))
    body, least = BODIES[shape]
    test = TEST.format(name=name, va=va, item=item, shape=shape, snake=sn, doc=doc,
                       body=body.format(port=port, short=short), least=least)
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


def shown(path):
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('shape', choices=('const', 'field', 'range'))
    ap.add_argument('function', help='`Class::method@00xxxxxx`, as the export names it')
    ap.add_argument('--item', type=int, required=True)
    ap.add_argument('--port', help="the port's Rust expression (`v`, `t` or `_this` in scope)")
    ap.add_argument('--off', type=lambda s: int(s, 0), help='the field offset (field; range: default 0x4)')
    ap.add_argument('--via', type=lambda s: int(s, 0), help='field: the pointer at this+VIA holds the record')
    ap.add_argument('--write-mod', action='store_true', help='append the `mod` line to sweep/mod.rs')
    ap.add_argument('--dry-run', action='store_true')
    args = ap.parse_args(argv)
    try:
        sp, script, tp, test, mod = generate(args.shape, args.function, item=args.item,
                                             port=args.port, off=args.off, via=args.via)
    except ValueError as e:
        print(f'sweep_gen.py: {e}', file=sys.stderr)
        return 2
    if args.dry_run:
        print(f'# {sp}\n{script}\n# {tp}\n{test}\n# sweep/mod.rs\n{mod}')
        return 0
    for p in (sp, tp):
        if p.exists():
            print(f'sweep_gen.py: {p} exists; a sweep is generated once', file=sys.stderr)
            return 2
    sp.write_text(script)
    sp.chmod(0o755)
    tp.write_text(test)
    if args.write_mod:
        with open(SWEEP / 'mod.rs', 'a') as f:
            f.write(mod + '\n')
    print(f'{shown(sp)}\n{shown(tp)}\n{mod}'
          + ('' if args.write_mod else '  (add to crates/sim/src/sweep/mod.rs)'))
    return 0


if __name__ == '__main__':
    sys.exit(main())
