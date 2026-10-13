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

  cell    the function reads the pointer in a global cell (`--cell VA`, in
          the image's .data) and an integer at `--off` of the record it
          names; the rows are the field shape's, `--port` over `v: i32`.
  vcall   the function calls a virtual slot of `this` (`--slot OFF`, with
          `--nargs N` stack arguments; `--tail` when it jumps into the slot
          after pushing them, as `is_city` does) and answers from it; the rows vary
          the slot's answer through `tools/emu/hooks.py`'s Harness, which
          records what the call was handed, and `--port` is over `v: i32`.
  outptr  the function stores its answer through the out-pointer it is
          handed on the stack (`[esp+4]`), reading one integer at
          `this+OFF`; the rows are the field shape's, `--port` over `v`.

  fwd     the function hands its work to one or two direct callees and
          answers a constant or the last callee's answer
          (`--call ADDR:NARGS:ECX:ARGS`, repeatable; ECX is `this` or
          `this+0xNN`, ARGS a comma list of `this`, `p0` or an integer;
          `--own N` stack arguments of its own, `--ret` one of `none`, an
          integer or `last`). The callees are hooked
          (`tools/emu/hooks.py`); a row is `<this> <p0> <answer> -> <eax> |
          <ecx,args…>;…` and the test holds the calls — which callee, with
          what `ecx` and arguments, in what order — to the spec.

  walk    the function hands the walker it was given (`[esp+4]`) two pointers
          into `this` through the walker's first vtable slot: `walk_data`
          (`--a0`, `--a1`, each `this`, `this+0xNN` or an absolute address); the callback is hooked.

A fixed-offset `lea eax, [ecx-0x70]` is `const` with `--port '_this - 0x70'`;
a `--mask` is `field` with the mask in the port.

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
from sweeplib import OBJ, TYPE, REC, OUT, cell, setup  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402,F401

ENTRY = 0x{va:08x}
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)
'''

SCRIPT_TAIL = '''

def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    for row in rows():
{write}
        r = {call}
        print(" ".join(str(x) for x in row) + f" -> {{r}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

VCALL_TAIL = '''

def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{}})
    for row in rows():
        h.calls.clear()
        vt = h.vtable({{0x{slot:x}: ("slot", {nargs}, row[0], {tail})}})
        obj = h.alloc(0x400)
        h.w32(obj, vt)
        r = h.call(ENTRY, (), ecx=obj)
        handed = ",".join(str(a) for name, ecx, args, _ in h.calls for a in args)
        print(" ".join(str(x) for x in row) + f" -> {{r}}" + (f"  # slot handed {{handed}}" if handed else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

FWD_TAIL = '''

CALLS = {calls}
OWN = {own}


def rows():
    rng = random.Random({item})
    return [(rng.randrange(0x1000, 0x7FFF0000) & ~3, rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(140)]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {{a: (f"{{a:x}}", n) for a, n, _, _ in CALLS}})
    for this, p0, ans, p1, p2 in rows():
        h.calls.clear()
        for i, (a, n, _, _) in enumerate(CALLS):
            h.answers[f"{{a:x}}"] = (ans + i) & 0xFFFFFFFF
        r = h.call(ENTRY, (p0, p1, p2)[:OWN], ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{{this}} {{p0}} {{ans}} {{p1}} {{p2}} -> {{r}} | {{seen}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

FWD_TEST = '''//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, fwd).

/// `{name}@{va:08x}`, run by `tools/emu/sweep_{snake}.py` under unicorn
/// (tools/emu) with its callees hooked: {doc}
#[test]
fn the_emulated_original_agrees_on_every_row() {{
    let Some(rows) = super::emu_rows("sweep_{snake}.py") else {{
        return;
    }};
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, _p0, ans, _p1, _p2) = (a[0], a[1], a[2], a[3], a[4]);
        let _ = (this, _p0, ans, _p1, _p2);
        let calls: Vec<Vec<i64>> = {calls};
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "{short} — calls, row `{{line}}`");
{ret}
        n += 1;
    }}
    assert!(n >= 100, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
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
ROWS['cell'] = ROWS['outptr'] = ROWS['vcall'] = ROWS['field']

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
BODIES['cell'] = BODIES['outptr'] = BODIES['vcall'] = BODIES['field']


def _fwd_expr(tok):
    tok = tok.strip()
    if tok == 'this':
        return 'this'
    if tok.startswith('this+'):
        return f'this + {int(tok[5:], 0):#x}'
    if tok == 'p0':
        return '_p0'
    if tok in ('p1', 'p2'):
        return '_' + tok
    return str(int(tok, 0))


def generate_fwd(spec, *, item, calls, own=0, ret='none'):
    name, va = parse(spec)
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    parsed, exprs = [], []
    for c in calls:
        addr, nargs, ecx, args = c.split(':', 3)
        parsed.append((int(addr, 16), int(nargs), ecx, args))
        exprs.append('vec![' + ', '.join([_fwd_expr(ecx)] + [_fwd_expr(t) for t in args.split(',') if t.strip()]) + ']')
    if ret == 'none':
        retcode = '        let _ = eax;'
    elif ret == 'last':
        last = f'(ans + {len(calls) - 1})' if len(calls) > 1 else 'ans'
        retcode = (f'        let want_eax = {last} as i32;\n'
                   f'        assert_eq!(eax.trim().parse::<i64>().unwrap(), i64::from(want_eax), "{short} — eax, row `{{line}}`");')
    else:
        retcode = (f'        assert_eq!(eax.trim().parse::<i64>().unwrap(), {int(ret, 0)}, '
                   f'"{short} — eax, row `{{line}}`");')
    cs = [(a, n, e, g) for a, n, e, g in parsed]
    script = (SCRIPT_HEAD.format(snake=sn, name=name, va=va, item=item,
                                 what=f'the object; calls {len(calls)} direct callee(s) and returns {ret}')
              + ROWS['const'].split('def rows')[0]
              + FWD_TAIL.format(calls=repr(cs), own=own, item=item)).replace('\n\n\n\ndef main', '\n\n\ndef main')
    doc = (f'{len(calls)} hooked callee(s), `ecx` and arguments held to the spec, `eax` {ret}.')
    test = FWD_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc,
                           calls='vec![' + ', '.join(exprs) + ']', ret=retcode)
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


WALK_SCRIPT = '''#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py, walk).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function calls
the walker's callback (the first word of the walker's vtable) with `ecx` = the walker
and two stack arguments, `{a0}` and `{a1}` of the object. One row per input,
`<this> <walker> -> <result> | <ecx,arg0,arg1>`.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x{va:08x}


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{}})
    rng = random.Random({item})
    for _ in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        ans = rng.randint(-0x80000000, 0x7FFFFFFF)
        vt = h.vtable({{0: ("walk", 2, ans & 0xFFFFFFFF)}})
        walker = h.alloc(16)
        h.w32(walker, vt)
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{{this}} {{walker}} -> {{r}} | {{seen}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

WALK_TEST = '''//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, walk).

/// `{name}@{va:08x}`, run by `tools/emu/sweep_{snake}.py` under unicorn
/// (tools/emu) with the walker's callback hooked: it is entered once, `ecx` the walker,
/// with `{a0}` and `{a1}` of the object, in that order.
#[test]
fn the_emulated_original_agrees_on_every_row() {{
    let Some(rows) = super::emu_rows("sweep_{snake}.py") else {{
        return;
    }};
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (_eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, walker) = (a[0], a[1]);
        let want = format!("{{walker}},{{}},{{}}", {e0}, {e1});
        assert_eq!(seen.trim(), want, "{short} — calls, row `{{line}}`");
        n += 1;
    }}
    assert!(n >= 100, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
'''


def _walk_expr(tok):
    tok = tok.strip()
    if tok == 'this':
        return 'this'
    if tok.startswith('this+'):
        return f'this + {int(tok[5:], 0):#x}'
    return f'{int(tok, 0):#x}'  # an absolute address: PathFinder::walk_data hands two globals (item 1916)


def generate_walk(spec, *, item, a0, a1):
    name, va = parse(spec)
    sn = snake(name)
    short = name.rsplit('::', 1)[-1]
    script = WALK_SCRIPT.format(snake=sn, name=name, va=va, item=item, a0=a0, a1=a1)
    test = WALK_TEST.format(snake=sn, name=name, va=va, item=item, short=short, a0=a0, a1=a1,
                            e0=_walk_expr(a0), e1=_walk_expr(a1))
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


def generate(shape, spec, *, item, port=None, off=None, via=None, cell=None, slot=None, nargs=0, tail=False):
    """`(script path, script text, test path, test text, mod line)`."""
    name, va = parse(spec)
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    port = port or f'todo!("the port of {name}")'
    call = 'signed(m.call(ENTRY, (), ecx={ecx}))'
    head = ''
    if shape == 'cell':
        if cell is None:
            raise ValueError('cell: --cell VA names the global cell')
        off = 0 if off is None else off
        what = f'unused; the pointer in the global cell 0x{cell:08x}, the int at +0x{off:x} of its record'
        head = f'CELL = 0x{cell:08x}\n'
        write = (f'        cell(uc, CELL, REC)\n'
                 f'        uc.mem_write(REC + 0x{off:x}, struct.pack("<i", row[0]))')
        ecx = 'OBJ'
        doc = f'the signed-32 edges and two hundred random words at the record\'s field, against `{port}`.'
    elif shape == 'outptr':
        off = 0 if off is None else off
        what = f'the object; reads the int at this+0x{off:x}, stores through the out pointer `[esp+4]`'
        write = f'        uc.mem_write(OBJ + 0x{off:x}, struct.pack("<i", row[0]))'
        ecx = 'OBJ'
        call = 'signed((m.call(ENTRY, (OUT,), ecx=OBJ), struct.unpack("<I", uc.mem_read(OUT, 4))[0])[1])'
        doc = f'the signed-32 edges and two hundred random words at the field, the out word against `{port}`.'
    elif shape == 'vcall':
        if slot is None:
            raise ValueError('vcall: --slot OFF names the vtable slot')
        what = (f'the object; {"jumps into" if tail else "calls"} slot +0x{slot:x} of its vtable '
                f'with {nargs} stack argument(s)')
        write = ecx = ''
        doc = f'the slot\'s answer over the signed-32 edges and two hundred random words, against `{port}`.'
    elif shape == 'const':
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
        raise ValueError(f'no such shape: {shape}; one of const, field, range, cell, vcall, outptr')
    if shape == 'vcall':
        tail = VCALL_TAIL.format(slot=slot, nargs=nargs, tail=tail)
    else:
        tail = SCRIPT_TAIL.format(write=write, call=call.format(ecx=ecx))
    script = (SCRIPT_HEAD.format(snake=sn, name=name, va=va, item=item, what=what) + head
              + ROWS[shape].format(item=item) + tail)
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
    ap.add_argument('shape', choices=('const', 'field', 'range', 'cell', 'vcall', 'outptr', 'fwd', 'walk'))
    ap.add_argument('--call', action='append', default=[], help='fwd: ADDR:NARGS:ECX:ARGS, in call order')
    ap.add_argument('--a0', default='this', help='walk: the first callback argument, `this` or `this+0xNN`')
    ap.add_argument('--a1', default='this', help='walk: the second callback argument')
    ap.add_argument('--own', type=int, default=0, help='fwd: stack arguments the function itself takes')
    ap.add_argument('--ret', default='none', help='fwd: none, an integer, or last (the last callee\'s answer)')
    ap.add_argument('function', help='`Class::method@00xxxxxx`, as the export names it')
    ap.add_argument('--item', type=int, required=True)
    ap.add_argument('--port', help="the port's Rust expression (`v`, `t` or `_this` in scope)")
    ap.add_argument('--off', type=lambda s: int(s, 0), help='the field offset (field; range: default 0x4)')
    ap.add_argument('--via', type=lambda s: int(s, 0), help='field: the pointer at this+VIA holds the record')
    ap.add_argument('--cell', type=lambda s: int(s, 0), help='cell: the global cell that holds the record pointer')
    ap.add_argument('--slot', type=lambda s: int(s, 0), help='vcall: the vtable slot the function calls')
    ap.add_argument('--nargs', type=int, default=0, help='vcall: stack arguments the slot takes')
    ap.add_argument('--tail', action='store_true', help='vcall: the slot is entered by a jmp, not a call')
    ap.add_argument('--write-mod', action='store_true', help='append the `mod` line to sweep/mod.rs')
    ap.add_argument('--dry-run', action='store_true')
    args = ap.parse_args(argv)
    try:
        if args.shape == 'walk':
            sp, script, tp, test, mod = generate_walk(args.function, item=args.item, a0=args.a0, a1=args.a1)
        elif args.shape == 'fwd':
            sp, script, tp, test, mod = generate_fwd(args.function, item=args.item, calls=args.call,
                                                     own=args.own, ret=args.ret)
        else:
            sp, script, tp, test, mod = generate(args.shape, args.function, item=args.item,
                                                 port=args.port, off=args.off, via=args.via,
                                                 cell=args.cell, slot=args.slot, nargs=args.nargs, tail=args.tail)
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
