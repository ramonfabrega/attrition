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
          `--via VIA` calls the slot of the object at `this+VIA` instead (the
          callee's `ecx` is held to it), with `--own N` stack arguments of the
          function's own that a `--tail` slot is handed unchanged (item 1993).

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

  mix     the function makes several of those in order: `--step` repeated,
          `call:ADDR:NARGS:ECX:ARGS`, `walk:A0:A1` (the walker's slot 0),
          `walk4:OFF` (slot 4, handed `**--glob`'s +0x10 word plus OFF); a `gated/`
          step runs only when `[walker+--gate] == 0` (item 1921, Loop 1890).

  walk    the function hands the walker it was given (`[esp+4]`) two pointers
          into `this` through the walker's first vtable slot: `walk_data`
          (`--a0`, `--a1`, each `this`, `this+0xNN` or an absolute address); the callback is hooked.

  obj     the function reads and/or writes fixed offsets of `this` (the
          `deref` and `store` the sweeps wrote by hand, parked 1933, 1943):
          `--win BYTES` of random object, `--bias OFF:V,V,..` (repeatable)
          pins a 32-bit word at OFF to one of the listed values per row,
          `--own N` stack arguments drawn from the edges and random words;
          a row is `<args…> | <before hex> -> <eax> | <after hex>` and the
          test hands `--port` (a Rust block over `a: &[i32]` and
          `m: &mut [u8]`, with `rd(m, off) -> i32` and `wr(m, off, v)`, whose
          value is eax) the same bytes and holds both the answer and every
          byte after to the original's. `--ret none` ignores eax.

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

VCALL_VIA_TAIL = '''

OWN = {own}


def rows():
    rng = random.Random({item})
    vs = list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]
    return [(v, *(rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(OWN))) for v in vs]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {{}})
    for row in rows():
        h.calls.clear()
        vt = h.vtable({{0x{slot:x}: ("slot", {nargs}, row[0], {tail})}})
        inner = h.alloc(0x400)
        h.w32(inner, vt)
        obj = h.alloc(0x400)
        h.w32(obj + 0x{via:x}, inner)
        r = h.call(ENTRY, row[1:], ecx=obj)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        assert [(c[1], c[2]) for c in h.calls] == [(inner, list(row[1:{nargs_or_zero}]))], (row, h.calls)
        print(" ".join(str(x) for x in row) + f" -> {{r}}")
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
        # `ecx` is read unsigned off the register; a stack argument is read signed (item 1932)
        ecx_expr = 'i64::from(_p0 as i32 as u32)' if ecx.strip() == 'p0' else _fwd_expr(ecx)
        exprs.append('vec![' + ', '.join([ecx_expr] + [_fwd_expr(t) for t in args.split(',') if t.strip()]) + ']')
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
        let _ = this;
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


MIX_SCRIPT = """#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
\"\"\"sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py, mix).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, one stack argument: the walker. The function makes
{nsteps} step(s) in order — direct callees (hooked), the walker's callback (slot 0, two
arguments) and its slot 4 (one argument{glob}){gate}.
One row per input, `<this> <walker> <g> <gate> -> <result> | <ecx,args…>;…`.
\"\"\"
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x{va:08x}
CALLS = {calls}
GLOBAL = {glob_cell}
GATE = {gate_off}


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{a: (f"{{a:x}}", n) for a, n, _, _ in CALLS}})
    rng = random.Random({item})
    vt = h.vtable({{0: ("walk", 2, 7), 4: ("walk4", 1, 9)}})
    for i in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        g = rng.randrange(0x1000, 0x7FFF0000) & ~3
        gv = 0 if i % 2 == 0 else rng.randint(1, 0x7FFFFFFF)
        walker = h.alloc(32)
        h.w32(walker, vt)
        if GATE is not None:
            h.w32(walker + GATE, gv)
        if GLOBAL is not None:
            rec = h.alloc(0x40)
            h.w32(rec + 0x10, g)
            h.m.uc.mem_write(GLOBAL, struct.pack("<I", rec))
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        print(f"{{this}} {{walker}} {{g}} {{gv}} -> {{r}} | {{seen}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
"""

MIX_TEST = """//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, mix).

/// `{name}@{va:08x}`, run by `tools/emu/sweep_{snake}.py` under unicorn
/// (tools/emu) with every callee and the walker's callbacks hooked: {doc}
#[test]
#[allow(clippy::vec_init_then_push)]
fn the_emulated_original_agrees_on_every_row() {{
    let Some(rows) = super::emu_rows("sweep_{snake}.py") else {{
        return;
    }};
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> -> <result>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (_eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let (this, walker, g, gate) = (a[0], a[1], a[2], a[3]);
        let _ = (this, g, gate);
        let mut calls: Vec<Vec<i64>> = Vec::new();
{steps}
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "{short} — calls, row `{{line}}`");
        n += 1;
    }}
    assert!(n >= 100, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
"""


def _mix_expr(tok):
    tok = tok.strip()
    if tok in ('p0', 'walker'):
        return 'walker'
    if tok == 'this':
        return 'this'
    if tok.startswith('this+'):
        return f'this + {int(tok[5:], 0):#x}'
    return str(int(tok, 0))


def generate_mix(spec, *, item, steps, glob=None, gate=None):
    """`steps`: `call:ADDR:NARGS:ECX:ARGS`, `walk:A0:A1` (the walker's slot 0),
    `walk4:OFF` (its slot 4, handed `**glob + 0x10` pointer's word + OFF),
    each optionally prefixed `gated/` — run only when `[walker+gate] == 0`."""
    name, va = parse(spec)
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    calls, code = [], []
    for st in steps:
        gated = st.startswith('gated/')
        st = st.removeprefix('gated/')
        kind, rest = st.split(':', 1)
        if kind == 'call':
            addr, nargs, ecx, args = rest.split(':', 3)
            calls.append((int(addr, 16), int(nargs), ecx, args))
            vec = ', '.join([_mix_expr(ecx)] + [_mix_expr(t) for t in args.split(',') if t.strip()])
        elif kind == 'walk':
            a0, a1 = rest.split(':')
            vec = ', '.join(['walker', _mix_expr(a0), _mix_expr(a1)])
        elif kind == 'walk4':
            vec = f'walker, g + {int(rest, 0):#x}'
        else:
            raise ValueError(f'a step is call:, walk: or walk4:, not `{st}`')
        push = f'calls.push(vec![{vec}]);'
        code.append(('        if gate == 0 { ' + push + ' }') if gated else ('        ' + push))
    script = MIX_SCRIPT.format(snake=sn, name=name, va=va, item=item, nsteps=len(steps),
                               glob=', from the global cell' if glob else '',
                               gate=f', the last gated on [walker+{gate:#x}] == 0' if gate is not None else '',
                               calls=repr(calls), glob_cell=hex(glob) if glob else 'None',
                               gate_off=hex(gate) if gate is not None else 'None')
    doc = f'{len(steps)} step(s) in order, `ecx` and arguments held to the spec.'
    test = MIX_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc,
                           steps='\n'.join(code))
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


OBJ_SCRIPT = '''#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py, obj).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a {win}-byte object of random bytes ({biasdoc}), {own} stack argument(s).
One row per input, `<args…> | <before hex> -> <eax> | <after hex>`; the test holds every byte.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE, signed  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x{va:08x}
WIN = {win}
OWN = {own}
BIAS = {bias!r}
ARGSET = {argset!r}
ARGFROM = {argfrom!r}
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def main(argv):
    from hooks import Harness
    rng = random.Random({item})
    h = Harness(argv[1], {{}})
    for i in range(220):
        this = h.alloc(WIN + 0x20)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        for off, vals in BIAS:
            v = rng.choice(vals) if rng.random() < 0.9 else rng.randint(-0x80000000, 0x7FFFFFFF)
            mem[off:off + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")
        args = [rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(OWN)]
        for idx, vals in ARGSET:
            args[idx] = rng.choice(vals) if rng.random() < 0.9 else args[idx]
        for idx, off, size in ARGFROM:
            if rng.random() < 0.45:
                args[idx] = int.from_bytes(mem[off:off + size], "little", signed=True)
        h.m.uc.mem_write(this, bytes(mem))
        guard = bytes(h.m.uc.mem_read(this + WIN, 0x20))
        r = signed(h.call(ENTRY, tuple(args), ecx=this))
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        assert bytes(h.m.uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        after = bytes(h.m.uc.mem_read(this, WIN))
        print(" ".join(str(a) for a in args) + f" | {{bytes(mem).hex()}} -> {{r}} | {{after.hex()}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

OBJ_TEST = '''//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, obj).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {{
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {{
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}}

fn unhex(s: &str) -> Vec<u8> {{
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}}

/// `{name}@{va:08x}`, run by `tools/emu/sweep_{snake}.py` under unicorn
/// (tools/emu): {doc}
#[test]
fn the_emulated_original_agrees_on_every_row() {{
    let Some(rows) = super::emu_rows("sweep_{snake}.py") else {{
        return;
    }};
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <before> -> <eax> | <after>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut m = unhex(before.trim());
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {{
            let (a, m) = (&a[..], &mut m[..]);
            let _ = (&a, &m);
            {port}
        }};
{ret}
        assert_eq!(m, unhex(after.trim()), "{short} — bytes, row `{{line}}`");
        n += 1;
    }}
    assert!(n >= 200, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
'''


def generate_obj(spec, *, item, win, bias, own=0, ret='eax', port=None, argset=(), argfrom=()):
    name, va = parse(spec)
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    parsed = []
    for b in bias:
        off, vals = b.split(':', 1)
        parsed.append((int(off, 0), [int(v, 0) for v in vals.split(',')]))
    pset = [(int(a.split(':', 1)[0]), [int(v, 0) for v in a.split(':', 1)[1].split(',')]) for a in argset]
    pfrom = [tuple(int(x, 0) for x in a.split(':')) for a in argfrom]
    port = port or f'todo!("the port of {name}")'
    biasdoc = ', '.join(f'+{o:#x} in {v}' for o, v in parsed) or 'no bias'
    script = OBJ_SCRIPT.format(snake=sn, name=name, va=va, item=item, win=win, own=own, bias=parsed, biasdoc=biasdoc, argset=pset, argfrom=pfrom)
    if ret == 'none':
        retcode = '        let _ = (got, want_eax);'
    else:
        retcode = f'        assert_eq!(got, want_eax, "{short} — eax, row `{{line}}`");'
    doc = f'{win} random bytes of object, {own} argument(s) over the edges and random words, `eax` and every byte after held to the port.'
    test = OBJ_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc, port=port, ret=retcode)
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


def generate(shape, spec, *, item, port=None, off=None, via=None, cell=None, slot=None, nargs=0, tail=False, own=0):
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
    if shape == 'vcall' and via is not None:
        tail = VCALL_VIA_TAIL.format(slot=slot, nargs=nargs, tail=tail, via=via, own=own, item=item,
                                     nargs_or_zero=1 + nargs)
        what = what.replace('the object;', f'the object, whose callee object is at this+0x{via:x};')
    elif shape == 'vcall':
        tail = VCALL_TAIL.format(slot=slot, nargs=nargs, tail=tail)
    else:
        tail = SCRIPT_TAIL.format(write=write, call=call.format(ecx=ecx))
    rows_src = '' if (shape == 'vcall' and via is not None) else ROWS[shape].format(item=item)
    script = (SCRIPT_HEAD.format(snake=sn, name=name, va=va, item=item, what=what) + head
              + rows_src + tail)
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
    ap.add_argument('shape', choices=('const', 'field', 'range', 'cell', 'vcall', 'outptr', 'fwd', 'walk', 'mix', 'obj'))
    ap.add_argument('--call', action='append', default=[], help='fwd: ADDR:NARGS:ECX:ARGS, in call order')
    ap.add_argument('--a0', default='this', help='walk: the first callback argument, `this` or `this+0xNN`')
    ap.add_argument('--a1', default='this', help='walk: the second callback argument')
    ap.add_argument('--step', action='append', default=[], help='mix: call:ADDR:NARGS:ECX:ARGS | walk:A0:A1 | walk4:OFF, in order; `gated/` prefix')
    ap.add_argument('--glob', type=lambda s: int(s, 0), help='mix: the global cell (the walk4 argument is *(*cell+0x10)+OFF)')
    ap.add_argument('--gate', type=lambda s: int(s, 0), help='mix: gated steps run when [walker+GATE] == 0')
    ap.add_argument('--own', type=int, default=0, help='fwd, obj: stack arguments the function itself takes')
    ap.add_argument('--argset', action='append', default=[], help='obj: IDX:V,V,.. — argument IDX drawn from the list (90%%)')
    ap.add_argument('--argfrom', action='append', default=[], help='obj: IDX:OFF:SIZE — argument IDX copies the signed SIZE-byte word at OFF (45%%)')
    ap.add_argument('--win', type=lambda s: int(s, 0), default=0x400, help='obj: bytes of random object')
    ap.add_argument('--bias', action='append', default=[], help='obj: OFF:V,V,.. — a 32-bit word at OFF drawn from the list')
    ap.add_argument('--ret', default='none', help='fwd: none, an integer, or last (the last callee\'s answer)')
    ap.add_argument('function', help='`Class::method@00xxxxxx`, as the export names it')
    ap.add_argument('--item', type=int, required=True)
    ap.add_argument('--port', help="the port's Rust expression (`v`, `t` or `_this` in scope)")
    ap.add_argument('--off', type=lambda s: int(s, 0), help='the field offset (field; range: default 0x4)')
    ap.add_argument('--via', type=lambda s: int(s, 0), help='field: the pointer at this+VIA holds the record; vcall: the object whose vtable is called')
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
        elif args.shape == 'mix':
            sp, script, tp, test, mod = generate_mix(args.function, item=args.item, steps=args.step,
                                                    glob=args.glob, gate=args.gate)
        elif args.shape == 'obj':
            sp, script, tp, test, mod = generate_obj(args.function, item=args.item, win=args.win, bias=args.bias,
                                                    own=args.own, ret='none' if args.ret == 'none' else 'eax', port=args.port,
                                                    argset=args.argset, argfrom=args.argfrom)
        elif args.shape == 'fwd':
            sp, script, tp, test, mod = generate_fwd(args.function, item=args.item, calls=args.call,
                                                     own=args.own, ret=args.ret)
        else:
            sp, script, tp, test, mod = generate(args.shape, args.function, item=args.item,
                                                 port=args.port, off=args.off, via=args.via,
                                                 cell=args.cell, slot=args.slot, nargs=args.nargs, tail=args.tail, own=args.own)
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
