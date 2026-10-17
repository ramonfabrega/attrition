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
          `--base ADDR` (item 2072, parked 2021) is the slot compared with
          the base implementation's address: a row's mode writes either
          ADDR (never entered) or a stub into the slot; on the base arm the
          function inlines the base — a field test of `this`, or, with
          `--base-via VIA --base-slot OFF`, a call of slot OFF of the
          object at `this+VIA` handed the same arguments. `--handed V,..`
          are the constants it pushes; a row is `<mode> <v> | <object hex>
          -> <eax> | <calls>`, the test holds the calls (which object, what
          arguments) and `--port`, a block over `mode`, `v` and `m` (the
          object's bytes, with `rd`), every byte of the object unwritten.
          `--base ADDR --base-call OFF` (item 2112) is the base inlined as a
          call of the compared object's own slot OFF, handed `--handed`
          (`pN` forwards the function's own argument N), `--base-not`
          negating its answer; the other arm jumps into the slot with the
          `--own N` arguments, and `--via VIA` compares the object at
          `this+VIA` (`makes_roads` and both `can_capture`s inline `is_city`,
          `ObjectData::isnt` inlines `!is`). A row is `<mode> <v> <own…> ->
          <eax> | <calls>`; `--port` is a block over `mode`, `v` and `own`.

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

  obj, with windows (the thirty-first pass, Loop 1890's ninth to
          thirteenth reaches): `--cell VA[:SIZE]` (repeatable) points the
          global cell at VA at a fresh record of random bytes a row; `--at
          VA:SIZE` writes a table of random bytes at VA itself (the image's
          data); `--ptrarg IDX[:SIZE]` hands argument IDX a buffer of random
          bytes; `--stub slot:OFF:NARGS` answers `this`'s vtable slot and
          `--stub ADDR:NARGS` a direct callee, each with a value drawn per
          row and printed after the arguments. A row is then `<args…
          stubs…> | <this>;<c…>;<p…> -> <eax> | <this>;<c…>;<p…>` and the
          port's block sees `c: &mut [Vec<u8>]` (cells, then tables, in
          order) and `p: &mut [Vec<u8>]` beside `a` and `m`; every byte
          of every window is held.

          `--outarg IDX:K:VALS` (item 2325) makes stub IDX write a value
          drawn from VALS through its argument K — the caller's local,
          as `get_inside` writes a player index — printed after the
          answers and `o` in a place; `--stub-cdecl IDX` hooks a direct
          callee as `__cdecl`, its arguments left for the caller to pop
          (`memcpy`, an import). `--deref IDX:K:N` (item 2379) prints
          the N bytes stub IDX's argument K points at, read when it is
          called, after the record's columns: a caller's local the
          record names only by depth (`add_command`'s `Command`). K `@VA`
          (item 2513) reads the absolute cell VA, as it stands when the
          stub is called: a flag the function sets around its callee.

          `--ptrtab LOC:N:SIZE[:vt]` (item 2782, parked 2772): an array of N
          record pointers at LOC (an address expression, `m[0x4a]` the
          object's own byte allowed) and each record SIZE random bytes, the
          array window `p<K>` and the records the N after it; the tables'
          places come before every `--place`, so a later flag names
          `p<K+1+i>`; `vt` also lays the stubbed vtable at each record's
          offset 0. A stub's answers per call are `--seq IDX:N` with
          `--answer-range IDX:LO:HI` (no wild word: a stub whose answer
          sizes a loop or indexes a record needs it; `--tame` is registers
          only), laid `a[own + stubs + N0 + N1 + ..]` after the arguments
          and one drawn answer a stub.

          `--alias WIN+OFF:TARGET[+K]` (item 2797, parked 2791): the word at
          WIN+OFF laid as the address of window TARGET (plus K) half the
          rows, the other half left as drawn: a pointer that must name
          another window or its own (`check_last_connect`'s `*q == node`);
          the port cannot see addresses, so it recovers the target from a
          pointer array's entry, as that sweep's does.

  void    `obj` with no answer: an empty body (`ret N`), `--own N`, every
          byte of the object held and `esp` checked after the return
          (parked 1889).

A fixed-offset `lea eax, [ecx-0x70]` is `const` with `--port '_this - 0x70'`;
a `--mask` is `field` with the mask in the port. `vcall --own N` hands the
function N stack arguments of its own, which the port reads as `p[..]`
(1966's `has_stance_type` compares the slot's answer with its `p0`).

Written: `tools/emu/sweep_<snake>.py` and `crates/sim/src/sweep/<snake>.rs`;
the registration is printed, or inserted with `--write-mod`. New tests without
`crate::` references join one of four standalone alphabetical partitions; tests
using crate internals stay in `sweep/mod.rs`. Existing registrations stay owned
by their current target. `--regen` redraws a generated sweep in place (both files
rewritten, registrations left alone), so a starved draw is one line. The script
keeps the line that drew it (`# sweep_gen: [...]`, its last line), so
`--regen tools/emu/sweep_<snake>.py` — or `--regen <snake>` — redraws it
with no line at hand (item 2325, parked 2311); a whole line given with
`--regen` replaces the kept one; a `--regen` with no `--port` keeps the test's
hand-written port block between the generated halves (item 2368). Without `--port` the test's port is `todo!()`, so the row
fails until the lane writes it — a sweep lands when the two sides agree,
never when a file exists. `--dry-run` prints both files instead.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EMU = ROOT / 'tools/emu'
SWEEP = ROOT / 'crates/sim/src/sweep'
# A method `Class::name`, or a free function `name` (item 2346: `add_type_bucket2`).
NAME = re.compile(r'^([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_~][A-Za-z0-9_<>=!\[\]+\-*/%&|^]*)*)@(00[0-9a-f]{6})$')


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
        raise ValueError(f'a function is `Class::method@00xxxxxx` or `name@00xxxxxx`, not `{spec}`')
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

OWN = {own}


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{}})
    for row in rows():
        h.calls.clear()
        vt = h.vtable({{0x{slot:x}: ("slot", {nargs}, row[0], {tail})}})
        obj = h.alloc(0x400)
        h.w32(obj, vt)
        r = h.call(ENTRY, row[1:], ecx=obj)
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
# A `vtordisp` thunk (`sub ecx, [ecx-4]; jmp target`, item 2362) reads the word
# before `this`: `this` is laid in the heap with a drawn displacement there.
VTORDISP = {vtordisp}
# `--record-order` (item 2672, parked 2666): each call's token is headed `sK:`, K the
# callee's place in CALLS, so two callees on one `this` are ordered.
ORDER = {order}
DISPS = (0, 4, 8, 0x10, 0x1c, 0x58, 0x60, 0x100, -4, -0x60)


def rows():
    rng = random.Random({item})
    return [(rng.randrange(0x1000, 0x7FFF0000) & ~3, rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF),
             rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(140)]


def main(argv):
    from hooks import Harness
    from callfn import STACK_BASE, STACK_SIZE
    h = Harness(argv[1], {{a: (f"{{a:x}}", n) for a, n, _, _ in CALLS}})
    drng = random.Random({item} + 1)
    for this, p0, ans, p1, p2 in rows():
        tail = ""
        if VTORDISP:
            disp = drng.choice(DISPS) if drng.random() < 0.6 else drng.randint(-0x80000000, 0x7FFFFFFF)
            buf = h.alloc(8)
            h.w32(buf, disp)
            this, tail = buf + 4, f" {{disp}}"
        h.calls.clear()
        for i, (a, n, _, _) in enumerate(CALLS):
            h.answers[f"{{a:x}}"] = (ans + i) & 0xFFFFFFFF
        r = h.call(ENTRY, (p0, p1, p2)[:OWN], ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        idx = {{f"{{a:x}}": k for k, (a, _, _, _) in enumerate(CALLS)}}
        seen = ";".join((f"s{{idx[nm]}}:" if ORDER else "") + ",".join(str(x) for x in (ecx, *args))
                        for nm, ecx, args, _ in h.calls)
        print(f"{{this}} {{p0}} {{ans}} {{p1}} {{p2}}{{tail}} -> {{r}} | {{seen}}")
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
{disp}        let calls: Vec<Vec<i64>> = {calls};
        let want: String = calls
            .iter()
            .enumerate()
            .map(|(k, c)| {{
                let _ = k;
                let t = c.iter().map(i64::to_string).collect::<Vec<_>>().join(",");
                {wrap}
            }})
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

# `vcall --own N`: the slot's answer, then N arguments of the function's own
# (half drawn from the slot's answer, so an equality arm is reached).
VCALL_OWN_ROWS = '''

def rows():
    rng = random.Random({item})
    vs = list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]
    out = []
    for v in vs:
        own = [v if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range({own})]
        out.append((v, *own))
    return out
'''

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
BODIES['cell'] = BODIES['outptr'] = BODIES['field']
BODIES['vcall'] = ('''        let v = i32::try_from(a[0]).expect("a 32-bit integer");
        #[allow(unused_variables)]
        let p: &[i64] = &a[1..];
        let got: i64 = {port};
        assert_eq!(got, want, "{short} — row `{{line}}`");''', 200)


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
    if tok == 'vtordisp':
        # `this` less the displacement the thunk reads at `this-4` (item 2362)
        return 'i64::from((this as u32).wrapping_sub(disp as u32))'
    return str(int(tok, 0))


def generate_fwd(spec, *, item, calls, own=0, ret='none', order=False):
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
    vtordisp = any(e.strip() == 'vtordisp' for _, _, e, _ in parsed)
    script = (SCRIPT_HEAD.format(snake=sn, name=name, va=va, item=item,
                                 what=f'the object; calls {len(calls)} direct callee(s) and returns {ret}')
              + ROWS['const'].split('def rows')[0]
              + FWD_TAIL.format(calls=repr(cs), own=own, item=item, vtordisp=vtordisp, order=order)).replace('\n\n\n\ndef main', '\n\n\ndef main')
    doc = (f'{len(calls)} hooked callee(s), `ecx` and arguments held to the spec, `eax` {ret}.')
    if vtordisp:
        doc += (' `this` is laid in the heap with a drawn displacement at `this-4` (the row\'s sixth'
                ' figure), and the callee\'s `ecx` is `this` less it.')
    if order:
        doc += ' Each call is headed `sK:`, K its callee\'s place in the spec, so the order is held.'
    test = FWD_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc,
                           wrap='format!("s{k}:{t}")' if order else 't',
                           disp='        let disp = a[5];\n' if vtordisp else '',
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
ROWS = {rows}
# `--count-only` (item 2672, parked 2667): a walker called a quarter-million times
# is one record, `<count> <first> <last> <sum a0> <sum a1>` (sums mod 2**32).
COUNT_ONLY = {count_only}


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{}})
    rng = random.Random({item})
    for _ in range(ROWS):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        ans = rng.randint(-0x80000000, 0x7FFFFFFF)
        vt = h.vtable({{0: ("walk", 2, ans & 0xFFFFFFFF)}})
        walker = h.alloc(16)
        h.w32(walker, vt)
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this, max_insns=200_000_000 if COUNT_ONLY else 200_000)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        if COUNT_ONLY:
            fmt = lambda c: ",".join(str(x) for x in (c[1], *c[2]))
            seen = " ".join([str(len(h.calls)), fmt(h.calls[0]), fmt(h.calls[-1]),
                             str(sum(c[2][0] & 0xFFFFFFFF for c in h.calls) & 0xFFFFFFFF),
                             str(sum(c[2][1] & 0xFFFFFFFF for c in h.calls) & 0xFFFFFFFF)])
        else:
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
{want}
        assert_eq!(seen.trim(), want, "{short} — calls, row `{{line}}`");
        n += 1;
    }}
    assert!(n >= {least}, "only {{n}} rows");
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


WALK_WANT = '''        let want = format!("{{walker}},{{}},{{}}", {e0}, {e1});'''
WALK_WANT_COUNT = '''        let pairs: Vec<(i64, i64)> = {port}.collect();
        let fmt = |p: &(i64, i64)| format!("{{walker}},{{}},{{}}", p.0, p.1);
        let sum = |f: fn(&(i64, i64)) -> i64| pairs.iter().map(|p| f(p) as u32 as u64).sum::<u64>() as u32;
        let want = format!(
            "{{}} {{}} {{}} {{}} {{}}",
            pairs.len(),
            fmt(&pairs[0]),
            fmt(pairs.last().unwrap()),
            sum(|p| p.0),
            sum(|p| p.1)
        );'''


def generate_walk(spec, *, item, a0, a1, count_only=False, rows=None, port=None):
    name, va = parse(spec)
    sn = snake(name)
    short = name.rsplit('::', 1)[-1]
    if count_only and not port:
        raise ValueError('--count-only wants --port, an iterator of the walker\'s `(a0, a1)` pairs (`walker`, `this` in scope)')
    rows = rows if rows is not None else (3 if count_only else 140)
    script = WALK_SCRIPT.format(snake=sn, name=name, va=va, item=item, a0=a0, a1=a1, rows=rows, count_only=count_only)
    want = (WALK_WANT_COUNT.format(port=port) if count_only
            else WALK_WANT.format(e0=_walk_expr(a0), e1=_walk_expr(a1)))
    test = WALK_TEST.format(snake=sn, name=name, va=va, item=item, short=short, a0=a0, a1=a1,
                            want=want, least=rows if count_only else 100)
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
One row per input, `<this> <walker> <g> <gate> -> <result> | <ecx,args…>;…` (the gate column is
the leftover `ecx` of the step before when a step names `stale`).
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
GATE_THIS = {gate_this}
GATE_AT = {gate_at}
LEN_AT = {len_at}
VBASE = {vbase}
STALE = {stale}
STALE_COL = {stale_col}
VBASE2 = {vbase2}
BIT_AT = {bit_at}
REC = {rec}
STORE = {store}
ELEMS = {elems}


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{a: (f"{{a:x}}", n) for a, n, _, _ in CALLS}})
    rng = random.Random({item})
    vt = h.vtable({{0: ("walk", 2, 7), 4: ("walk4", 1, 9)}})
    for i in range(140):
        this = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if GATE_THIS or GATE_AT is not None or BIT_AT is not None or LEN_AT is not None or STORE is not None or (ELEMS and ELEMS[0] in ('list', 'count')):
            this = h.alloc(0x100)  # a written object: the gate is the short at [this]
        g = rng.randrange(0x1000, 0x7FFF0000) & ~3
        if VBASE is not None:
            # `this` sits inside a record; [this-VBASE] points at a record whose +4 is the displacement g
            this = h.alloc(0x200) + 0x100
            g = rng.randint(-0x100, 0x1000)  # the displacement, in the g column
            vb = h.alloc(0x20)
            h.w32(vb + 4, g)
            h.w32(this - VBASE, vb)
        d2 = 0
        if VBASE2 is not None:
            # a second displacement record at [this-VBASE2], its +4 the `d2` column (parked 2668, 2674)
            d2 = rng.randint(-0x100, 0x1000)
            vb2 = h.alloc(0x20)
            h.w32(vb2 + 4, d2)
            h.w32(this - VBASE2, vb2)
        gv = 0 if i % 2 == 0 else rng.randint(1, 0x7FFFFFFF)
        if STALE:
            # the `ecx` a callback or a direct callee leaves (parked 1922, 2498, 2563), printed in the gate column
            sv = g  # a walker word is the gate: the leftover is the `g` column, the gate column stays the gate
            if STALE_COL:
                sv = rng.randrange(0x1000, 0x7FFF0000) & ~3  # its own column (parked 2673), printed after the gate
            elif GATE is None:
                gv = sv = rng.randrange(0x1000, 0x7FFF0000) & ~3
            h.answers["walk"] = (7, sv)
            h.answers["walk4"] = (9, sv)
            for a, _, _, _ in CALLS:
                h.answers[f"{{a:x}}"] = (0, sv)
        walker = h.alloc(32)
        h.w32(walker, vt)
        if GATE is not None:
            h.w32(walker + GATE, gv)
        if GATE_AT is not None:
            # a bit at another offset: the gate column is the masked byte (0 on even rows)
            off, mask = GATE_AT
            v = rng.randrange(256)
            v = v & ~mask & 0xFF if i % 2 == 0 else v | mask
            h.w8(this + off, v)
            gv = v & mask
        elif LEN_AT is not None:
            # a walk length read from memory: the sign-extended short, the gate column
            gv = 0 if i % 3 == 0 else rng.choice([-1, 1, rng.randint(-0x8000, 0x7FFF)]) or 1
            h.w16(this + LEN_AT, gv)
        elif GATE_THIS:
            gv = gv & 0xFFFF if i % 4 != 3 else 0
            h.w16(this, gv - 0x10000 if gv & 0x8000 else gv)
        if GLOBAL is not None:
            rec = h.alloc(0x40)
            if REC:
                g = rec  # the record base the cell holds is the g column
            h.w32(rec + 0x10, g)
            h.m.uc.mem_write(GLOBAL, struct.pack("<I", rec))
        el = ""
        if ELEMS and ELEMS[0] == 'table':
            # the elements: pointer words of the table *(*cell+0x10), one per STEP bytes in LO..HI
            _, lo, hi, stp = ELEMS
            tbl = h.alloc(hi + 0x10)
            h.w32(rec + 0x10, tbl)
            elems = [rng.randrange(0x1000, 0x7FFF0000) & ~3 for _ in range(lo, hi, stp)]
            for k, e in enumerate(elems):
                h.w32(tbl + lo + k * stp, e)
            el = "".join(f" {{e}}" for e in elems)
        elif ELEMS and ELEMS[0] == 'count':
            # a count at this+CNT and a record array at [this+PTR], STRIDE bytes a record (parked 2689)
            _, cnt, ptr, stp = ELEMS
            count = rng.choice([0, 1, 2, 3, 5, -1])
            base = h.alloc(max(count, 1) * stp + 0x10)
            h.w32(this + cnt, count)
            h.w32(this + ptr, base)
            elems = [base + k * stp for k in range(max(count, 0))]
            el = f" {{count}}" + "".join(f" {{e}}" for e in elems)
        elif ELEMS:
            # a linked list: the head at this+HEAD, each node's next at +NEXT, ended by null
            _, head, nxt = ELEMS
            elems = [h.alloc(0x40) for _ in range(rng.choice([0, 1, 2, 3, 5]))]
            for k, e in enumerate(elems):
                h.w32(e + nxt, elems[k + 1] if k + 1 < len(elems) else 0)
            h.w32(this + head, elems[0] if elems else 0)
            el = "".join(f" {{e}}" for e in elems)
        sw = f" {{sv}}" if STALE and STALE_COL else ""
        if VBASE2 is not None:
            sw += f" {{d2}}"
        if BIT_AT is not None:
            # a second gate: a bit of `this`, its masked byte the last extra column (parked 2669, 2675)
            boff, bmask = BIT_AT
            bv = rng.randrange(256)
            bv = bv & ~bmask & 0xFF if i % 3 == 0 else bv | bmask
            h.w8(this + boff, bv)
            sw += f" {{bv & bmask}}"
        if STORE is not None:
            off, val, size, woff = STORE
            w = 0 if i % 3 == 0 else rng.randint(1, 0x7FFFFFFF)
            h.w32(walker + woff, w)
            sw += f" {{w}}"
            h.m.uc.mem_write(this, bytes(rng.randrange(256) for _ in range(0x100)))
            pre = bytes(h.m.uc.mem_read(this, 0x100))
        h.calls.clear()
        r = h.call(ENTRY, (walker,), ecx=this)
        esp = h.m.uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        seen = ";".join(",".join(str(x) for x in (ecx, *args)) for _, ecx, args, _ in h.calls)
        after = ""
        if STORE is not None:
            post = bytes(h.m.uc.mem_read(this, 0x100))
            changed = [k for k in range(0x100) if post[k] != pre[k]]
            assert all(off <= k < off + size for k in changed), f"wrote outside the store: {{changed}}"
            after = " | " + post[off:off + size].hex() + " " + pre[off:off + size].hex()
        print(f"{{this}} {{walker}} {{g}} {{gv}}{{sw}}{{el}} -> {{r}} | {{seen}}{{after}}")
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
        let (eax, seen) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let eax: i64 = eax.trim().parse().expect("an integer");
        let (this, walker, g, gate) = (a[0], a[1], a[2], a[3]);
{store_w}{store_split}{elems_src}        let _ = eax;
        let stale = {stale_src}; // the slot-4 callee's leftover `ecx` when a step names it
        let _ = (this, g, gate, stale);
{d2_src}        let mut calls: Vec<Vec<i64>> = Vec::new();
{steps}
        let want: String = calls
            .iter()
            .map(|c| c.iter().map(i64::to_string).collect::<Vec<_>>().join(","))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(seen.trim(), want, "{short} — calls, row `{{line}}`");
{retcheck}{storecheck}        n += 1;
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
    if tok == 'stale':
        return 'stale'  # the slot-4 callee's leftover ecx (parked 2498)
    if tok == 'stale|this':
        return 'if gate != 0 { g } else { this }'  # a gated callee's leftover, else the entry ecx (parked 2568)
    if tok.startswith('this') and tok.endswith('+d2'):
        k = int(tok[4:-3], 0)  # the second vbase displacement (parked 2668, 2674)
        return f'this {"-" if k < 0 else "+"} {abs(k):#x} + d2'
    if tok.startswith('this') and ('-' in tok or tok.endswith('+d')):
        # `this-0x18+d`: an offset, and the vbase displacement `d` (the `g` column)
        body, disp = (tok[4:-2], ' + g') if tok.endswith('+d') else (tok[4:], '')
        k = int(body, 0)
        return f'this {"-" if k < 0 else "+"} {abs(k):#x}{disp}'
    if tok.startswith('this+') and tok.endswith('+len'):
        # `this+0x12+len`: the sign-extended short read from memory (the gate column)
        return f'this + {int(tok[5:-4], 0):#x} + gate'
    if tok == 'rec':
        return 'g'  # the record the global cell holds (parked 2584): its base is the g column
    if tok.startswith('rec+'):
        return f'g + {int(tok[4:], 0):#x}'
    if tok == 'g':
        return 'g'  # the word the global cell's record holds at +0x10
    if tok.startswith('g+'):
        return f'g + {int(tok[2:], 0):#x}'
    if tok == 'count':
        return 'count'  # the queue count read from `this` (parked 2689)
    if tok == 'elem':
        return 'e'  # the element of an `each/` step (parked 2585, 2589)
    if tok.startswith('elem+'):
        return f'e + {int(tok[5:], 0):#x}'
    if tok.startswith('this+'):
        return f'this + {int(tok[5:], 0):#x}'
    return str(int(tok, 0))


def generate_mix(spec, *, item, steps, glob=None, gate=None, gate_this=False, vbase=None, len_at=None,
                 gate_ne=False, ret_arg=False, store=None, elems=None, stale_col=False, vbase2=None, bit_at=None):
    """`steps`: `call:ADDR:NARGS:ECX:ARGS`, `walk:A0:A1` (the walker's slot 0),
    `walk4:OFF` (its slot 4, handed `**glob + 0x10` pointer's word + OFF),
    each optionally prefixed `gated/` — run only when `[walker+gate] == 0`."""
    name, va = parse(spec)
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    sv = store.split(':') if store else None
    calls, code = [], []
    ev = None
    if elems:
        ep = elems.split(':')
        if ep[0] == 'table' and len(ep) == 4:
            ev = ('table', int(ep[1], 0), int(ep[2], 0), int(ep[3], 0))
        elif ep[0] == 'list' and len(ep) == 3:
            ev = ('list', int(ep[1], 0), int(ep[2], 0))
        elif ep[0] == 'count' and len(ep) == 4:
            ev = ('count', int(ep[1], 0), int(ep[2], 0), int(ep[3], 0))
        else:
            raise ValueError(f'--elems is table:LO:HI:STEP, list:HEAD:NEXT or count:CNT:PTR:STRIDE, not {elems!r}')
    for st in steps:
        each = st.startswith('each/')
        st = st.removeprefix('each/')
        if each and ev is None:
            raise ValueError('an `each/` step needs --elems')
        conds = []
        while st.startswith(('gated/', 'bit/', 'zero/', 'nz/')):
            if st.startswith(('zero/', 'nz/')):
                if ev is None or ev[0] != 'count':
                    raise ValueError('a `zero/` or `nz/` step needs --elems count:')
                conds.append('count == 0' if st.startswith('zero/') else 'count != 0')
                st = st.split('/', 1)[1]
            elif st.startswith('bit/'):
                if not bit_at:
                    raise ValueError('a `bit/` step needs --bit-at')
                conds.append('bit != 0')
                st = st.removeprefix('bit/')
            else:
                conds.append('gate != 0' if (gate_this or gate_ne or len_at is not None) else 'gate == 0')
                st = st.removeprefix('gated/')
        gated = bool(conds)
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
        if each:
            push = f'for &e in elems {{ {push} }}'
        cond = ' && '.join(conds)
        code.append(('        if ' + cond + ' { ' + push + ' }') if gated else ('        ' + push))
    script = MIX_SCRIPT.format(snake=sn, name=name, va=va, item=item, nsteps=len(steps),
                               glob=', from the global cell' if glob else '',
                               gate=f', the last gated on [walker+{gate:#x}] {"!=" if gate_ne else "=="} 0' if gate is not None else '',
                               calls=repr(calls), glob_cell=hex(glob) if glob else 'None',
                               gate_off=hex(gate) if gate is not None else 'None',
                               gate_this=bool(gate_this) or len_at is not None,
                               gate_at=(f'({int(gate_this.split(":")[0], 0):#x}, {int(gate_this.split(":")[1], 0):#x})'
                                        if isinstance(gate_this, str) else 'None'),
                               len_at=hex(len_at) if len_at is not None else 'None',
                               vbase=hex(vbase) if vbase is not None else 'None',
                               stale=any('stale' in st.split(':', 1)[1] for st in steps),
                               stale_col=bool(stale_col),
                               vbase2=hex(vbase2) if vbase2 is not None else 'None',
                               bit_at=(f'({int(bit_at.split(":")[0], 0):#x}, {int(bit_at.split(":")[1], 0):#x})' if bit_at else 'None'),
                               rec=any('rec' in st.split(':', 1)[1] for st in steps),
                               elems=('(' + ', '.join(repr(x) if isinstance(x, str) else (f'{x:#x}' if i < len(ev) - 1 or ev[0] == 'list' else str(x)) for i, x in enumerate(ev)) + ')') if ev else 'None',
                               store=(f'({int(sv[0], 0):#x}, {int(sv[1], 0):#x}, {int(sv[2], 0)}, {int(sv[3], 0):#x})'
                                      if store else 'None'))
    doc = f'{len(steps)} step(s) in order, `ecx` and arguments held to the spec.'
    storecheck = ''
    if sv:
        cond = 'gate != 0' if (gate_this or gate_ne or len_at is not None) else 'gate == 0'
        mask = (1 << (8 * int(sv[2], 0))) - 1
        storecheck = (f'        let want_after = if {cond} && w != 0 {{ {int(sv[1], 0) & mask:#x} }} else {{ before }};\n'
                      f'        assert_eq!(after, want_after, "{short} — the store, row `{{line}}`");\n')
    elems_src = ''
    ecol = 4 + int(bool(stale_col)) + int(vbase2 is not None) + int(bool(bit_at))
    if ev and ev[0] == 'count':
        elems_src = (f'        let count = a[{ecol}];\n        let _ = count;\n'
                     f'        let elems = &a[{ecol + 1}..];\n'
                     f'        assert_eq!(elems.len() as i64, count.max(0), "{short} — one element a counted record, row `{{line}}`");\n')
    elif ev:
        elems_src = f'        let elems = &a[{ecol}..];\n'
        if ev[0] == 'table':
            elems_src += f'        assert_eq!(elems.len(), {(ev[2] - ev[1] + ev[3] - 1) // ev[3]}, "{short} — one element a table word, row `{{line}}`");\n'
    test = MIX_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc, elems_src=elems_src,
                           store_split=('        let (seen, tail) = seen.rsplit_once(" | ").expect("`<calls> | <after> <before>`");\n'
                                        '        let (after, before) = tail.split_once(\' \').expect("`<after> <before>`");\n'
                                        '        let rd = |h: &str| (0..h.len() / 2).rev().fold(0i64, |acc, k| acc << 8 | i64::from_str_radix(&h[2 * k..2 * k + 2], 16).expect("hex"));\n'
                                        '        let (after, before) = (rd(after), rd(before));\n') if sv else '',
                           store_w=f'        let w = a[{ecol}];\n' if sv else '',
                           d2_src=(f'        let d2 = a[{4 + int(bool(stale_col))}];\n        let _ = d2;\n' if vbase2 is not None else '')
                           + (f'        let bit = a[{4 + int(bool(stale_col)) + int(vbase2 is not None)}];\n        let _ = bit;\n' if bit_at else ''),
                           storecheck=storecheck,
                           steps='\n'.join(code), stale_src='a[4]' if stale_col else 'g' if gate is not None else 'gate',
                           retcheck='        assert_eq!(eax, walker, "{short} — the result is the out-pointer, row `{line}`");\n'.replace('{short}', short) if ret_arg else '')
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


VBASE_SCRIPT = '''#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
\"\"\"sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py, vcall --base).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a {win}-byte object of random bytes. It loads its own vtable slot +0x{slot:x} and compares
the pointer with the base implementation 0x{base:08x}: equal, it {basedoc}; different, it calls the slot.
Both arms are handed {handed!r}. A row's mode is 1 when the slot holds the base address (never entered), 0 when
it holds a stub; every called slot answers `v`, drawn per row.
Row: `<mode> <v> | <object hex> -> <eax> | <calls>`, a call `this|via,<args…>`.
\"\"\"
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x{va:08x}
BASE = 0x{base:08x}
SLOT = 0x{slot:x}
NARGS = {nargs}
VIA = {via!r}
VIA_SLOT = {via_slot!r}
WIN = {win}
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{}})
    uc = h.m.uc
    rng = random.Random({item})
    vt = h.vtable({{SLOT: ("slot", NARGS, 0)}})
    stub = h.r32(vt + SLOT)
    inner = None
    if VIA is not None:
        ivt = h.vtable({{VIA_SLOT: ("via", NARGS, 0)}})
        inner = h.alloc(0x40)
        h.w32(inner, ivt)
    obj = h.alloc(WIN + 0x20)
    for i in range(240):
        mode = i % 2
        v = rand32(rng)
        h.calls.clear()
        h.answers["slot"] = h.answers["via"] = v
        h.w32(vt + SLOT, BASE if mode else stub)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        mem[0:4] = vt.to_bytes(4, "little")
        if VIA is not None:
            mem[VIA:VIA + 4] = inner.to_bytes(4, "little")
        uc.mem_write(obj, bytes(mem))
        guard = bytes(uc.mem_read(obj + WIN, 0x20))
        r = h.call(ENTRY, (), ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        assert bytes(uc.mem_read(obj, WIN + 0x20)) == bytes(mem) + guard, "wrote the object"
        calls = ";".join(",".join([{{obj: "this", inner: "via"}}.get(ecx, "other"), *map(str, args)])
                         for _, ecx, args, _ in h.calls)
        print(f"{{mode}} {{v}} | {{bytes(mem).hex()}} -> {{r}} | {{calls}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

VBASE_TEST = '''//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, vcall --base).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {{
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
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
    let handed: &[i64] = &{handed_rs};
    let args: String = handed.iter().map(|h| format!(",{{h}}")).collect();
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<mode> <v> | <object> -> <eax> | <calls>`");
        let (head, obj) = lhs.split_once(" | ").expect("`<mode> <v> | <object>`");
        let (eax, calls) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let a: Vec<i64> = head.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (mode, v) = (a[0], i32::try_from(a[1]).expect("a 32-bit answer"));
        let m = unhex(obj.trim());
        let want_calls = match (mode, {via_some}) {{
            (0, _) => format!("this{{args}}"),
            (_, true) => format!("via{{args}}"),
            _ => String::new(),
        }};
        assert_eq!(calls.trim(), want_calls, "{short} — calls, row `{{line}}`");
        let got: i64 = {{
            let (mode, v, m) = (mode, v, &m[..]);
            let _ = (mode, v, m);
            {port}
        }};
        assert_eq!(got, eax.trim().parse::<i64>().expect("an integer"), "{short} — eax, row `{{line}}`");
        n += 1;
    }}
    assert!(n >= 200, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
'''


def generate_vbase(spec, *, item, slot, base, handed=(), via=None, via_slot=None, win=0x200, port=None):
    """`vcall --base ADDR` (item 2072, parked 2021): a slot compared with the base's address."""
    name, va = parse(spec)
    if slot is None:
        raise ValueError('vcall --base: --slot OFF names the vtable slot')
    if (via is None) != (via_slot is None):
        raise ValueError('vcall --base: --base-via and --base-slot go together')
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    handed = [int(h, 0) if isinstance(h, str) else h for h in handed]
    port = port or f'todo!("the port of {name}")'
    basedoc = (f'calls slot +0x{via_slot:x} of the object at this+0x{via:x}' if via is not None
               else 'inlines the base on the object\'s own fields')
    script = VBASE_SCRIPT.format(snake=sn, name=name, va=va, item=item, base=base, slot=slot, nargs=len(handed),
                                 via=via, via_slot=via_slot, win=win, handed=handed, basedoc=basedoc)
    doc = (f'slot +0x{slot:x} against the base 0x{base:08x}, a row each arm, the slots\' answer over the '
           f'signed-32 edges and random words, {win} random bytes of object; the calls and `eax` held.')
    test = VBASE_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc, port=port,
                             handed_rs='[' + ', '.join(str(h) for h in handed) + ']',
                             via_some='true' if via is not None else 'false')
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'

VCALLBASE_SCRIPT = """#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
\"\"\"sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py, vcall --base --base-call).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, {own} stack argument(s) of its own. The object {objdoc} has its vtable's slot +0x{slot:x} compared
with the base implementation 0x{base:08x}: equal, it calls that object's own slot +0x{bslot:x} handed {handed!r}
(`pN` the function's own argument N){notdoc}; different, it jumps into slot +0x{slot:x} with its own arguments, whose
answer is the answer. A row's mode is 1 when the slot holds the base address (never entered), 0 when it holds a
stub; both slots answer `v`, drawn per row, zero a quarter of the time.
Row: `<mode> <v> <own…> -> <eax> | <calls>`, a call `slot|base,this|rec|other,<args…>`; the {win} bytes of
`this` (and the 0x40 of the record) are random and held unwritten.
\"\"\"
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402

ENTRY = 0x{va:08x}
BASE = 0x{base:08x}
SLOT = 0x{slot:x}
BSLOT = 0x{bslot:x}
OWN = {own}
HANDED = {handed!r}
VIA = {via!r}
WIN = {win}
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.4 else rng.randint(-0x80000000, 0x7FFFFFFF)


def main(argv):
    from hooks import Harness
    h = Harness(argv[1], {{}})
    uc = h.m.uc
    rng = random.Random({item})
    vt = h.vtable({{SLOT: ("slot", OWN, 0), BSLOT: ("base", len(HANDED), 0)}})
    stub = h.r32(vt + SLOT)
    obj = h.alloc(WIN + 0x20)
    rec = h.alloc(0x60) if VIA is not None else obj
    for i in range(240):
        mode = i % 2
        v = 0 if rng.random() < 0.25 else rand32(rng)
        own = [rand32(rng) for _ in range(OWN)]
        h.calls.clear()
        h.answers["slot"] = h.answers["base"] = v
        h.w32(vt + SLOT, BASE if mode else stub)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        if VIA is None:
            mem[0:4] = vt.to_bytes(4, "little")
        else:
            mem[VIA:VIA + 4] = rec.to_bytes(4, "little")
            rmem = bytearray(rng.randrange(256) for _ in range(0x40))
            rmem[0:4] = vt.to_bytes(4, "little")
            uc.mem_write(rec, bytes(rmem))
        uc.mem_write(obj, bytes(mem))
        before = bytes(uc.mem_read(obj, WIN + 0x20)) + bytes(uc.mem_read(rec, 0x40))
        r = h.call(ENTRY, own, ecx=obj)
        esp = uc.reg_read(UC_X86_REG_ESP)
        assert esp == STACK_BASE + STACK_SIZE - 0x100, f"esp {{esp:x}}: wrong pop"
        assert bytes(uc.mem_read(obj, WIN + 0x20)) + bytes(uc.mem_read(rec, 0x40)) == before, "wrote memory"
        who = {{obj: "this", rec: "rec"}} if VIA is not None else {{obj: "this"}}
        calls = ";".join(",".join([name, who.get(ecx, "other"), *map(str, args)]) for name, ecx, args, _ in h.calls)
        print(" ".join(map(str, [mode, v, *own])) + f" -> {{r}} | {{calls}}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
"""

VCALLBASE_TEST = """//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, vcall --base --base-call).

/// `{name}@{va:08x}`, run by `tools/emu/sweep_{snake}.py` under unicorn
/// (tools/emu): {doc}
#[test]
fn the_emulated_original_agrees_on_every_row() {{
    let Some(rows) = super::emu_rows("sweep_{snake}.py") else {{
        return;
    }};
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {{
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<mode> <v> <own…> -> <eax> | <calls>`");
        let (eax, calls) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let a: Vec<i64> = lhs.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let (mode, v, own) = (a[0], i32::try_from(a[1]).expect("a 32-bit answer"), &a[2..]);
        let handed: Vec<String> = [{handed_rs}].iter().map(|t: &&str| match t.strip_prefix('p') {{
            Some(i) => own[i.parse::<usize>().expect("pN")].to_string(),
            None => t.to_string(),
        }}).collect();
        let own_s: String = own.iter().map(|x| format!(",{{x}}")).collect();
        let want_calls = if mode == 1 {{
            format!("base,{obj}{{}}", handed.iter().map(|h| format!(",{{h}}")).collect::<String>())
        }} else {{
            format!("slot,{obj}{{own_s}}")
        }};
        assert_eq!(calls.trim(), want_calls, "{short} — calls, row `{{line}}`");
        let got: i64 = {{
            let (mode, v, own) = (mode, v, own);
            let _ = (mode, v, own);
            {port}
        }};
        assert_eq!(got, eax.trim().parse::<i64>().expect("an integer"), "{short} — eax, row `{{line}}`");
        n += 1;
    }}
    assert!(n >= 200, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
"""


def generate_vcallbase(spec, *, item, slot, base, base_slot, handed=(), own=0, via=None, negate=False, win=0x200,
                       port=None):
    """`vcall --base ADDR --base-call OFF` (parked 2095's sibling, item 2112): the slot compared with the base's
    address, whose body the function inlines as a call of the same object's slot OFF handed constants or its own
    arguments (`pN`), optionally negated; the other arm jumps into the slot with the function's own arguments."""
    name, va = parse(spec)
    if slot is None or base_slot is None:
        raise ValueError('vcall --base --base-call: --slot OFF and --base-call OFF name the two slots')
    handed = [str(h) for h in handed]
    for t in handed:
        if t.startswith('p'):
            if not t[1:].isdigit() or int(t[1:]) >= own:
                raise ValueError(f'vcall --base-call: {t} names no argument of the function\'s {own}')
        else:
            int(t, 0)
    handed = [t if t.startswith('p') else str(int(t, 0)) for t in handed]
    short = name.rsplit('::', 1)[-1]
    sn = snake(name)
    port = port or f'todo!("the port of {name}")'
    objdoc = f'at `this+0x{via:x}`' if via is not None else '`this`'
    notdoc = ', and answers that slot\'s answer `== 0`' if negate else ''
    script = VCALLBASE_SCRIPT.format(snake=sn, name=name, va=va, item=item, base=base, slot=slot, bslot=base_slot,
                                     own=own, handed=handed, via=via, win=win, objdoc=objdoc, notdoc=notdoc)
    doc = (f'slot +0x{slot:x} of {objdoc} against the base 0x{base:08x}, a row each arm: the base arm calls slot '
           f'+0x{base_slot:x} handed `{",".join(handed)}`{notdoc}, the other jumps into the slot with the {own} own '
           f'argument(s); the slots\' answer over the signed-32 edges and random words, every byte held.')
    test = VCALLBASE_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc, port=port,
                                 handed_rs=', '.join(f'"{t}"' for t in handed),
                                 obj='rec' if via is not None else 'this')
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
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
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
    for i in range({rows}):
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
        r = h.call(ENTRY, tuple(args), ecx=this)  # Harness.call signs eax; signed() again would double a negative
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
    assert!(n >= {floor}, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
'''


OBJ_WIN_SCRIPT = '''#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_{snake}.py — `{name}@{va:08x}` under unicorn (item {item}, tools/sweep_gen.py, obj with windows).

    uv run tools/emu/sweep_{snake}.py <install>/riseofnations.exe

`__thiscall`, `ecx` = a {win}-byte object of random bytes ({biasdoc}), {own} stack argument(s);
{windoc}.
One row per input, `<args… stubs…> | <this>;<c…>;<p…> -> <eax> | <this>;<c…>;<p…>` (hex); the stubs'
answers follow the arguments, and the test holds every byte of every window.
"""
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import STACK_BASE, STACK_SIZE  # noqa: E402
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402
from sweeplib import link_value  # noqa: E402
from sweeplib import draw_reg  # noqa: E402

ENTRY = 0x{va:08x}
WIN = {win}
OWN = {own}
BIAS = {bias!r}
ARGSET = {argset!r}
ARGFROM = {argfrom!r}
CELLS = {cells!r}
AT = {at!r}
PTRARGS = {ptrargs!r}
PNULLS = {pnulls!r}
STUBS = {stubs!r}
PLACES = {places!r}
NULLS = {nulls!r}
WBIAS = {wbias!r}
FILLS = {fills!r}
ARGRANGE = {argrange!r}
SLOTBASE = {slotbase!r}
ECXINT = {ecxint!r}
EDX = {edx!r}
EDXWIN = {edxwin!r}
# `--tame` (item 2346): a register drawn wild this share of the rows, a random word.
WILD = {wild!r}
ANSWERS = {answers!r}
VTAT = {vtat!r}
VTATHALF = {vtathalf!r}
ALIAS = {alias!r}
ANSWERWIN = {answerwin!r}
CDECL = {cdecl!r}
ARGEQ = {argeq!r}
SEQ = {seq!r}
ANSRANGE = {ansrange!r}
CELLTHIS = {cellthis!r}
VTALSO = {vtalso!r}
SPARSE = {sparse!r}
GTAB = {gtab!r}
RECORD = {record!r}
RECORDORDER = {recordorder!r}
RECORDEDX = {recordedx!r}
ROWSPLIT = {rowsplit!r}
LINKS = {links!r}
OUTARG = {outarg!r}
CDECLSTUBS = {cdeclstubs!r}
STUBWRITE = {stubwrite!r}
ECXANSWER = {ecxanswer!r}
RING = {ring!r}
DEREF = {deref!r}
RET = {ret!r}
I32 = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 3, 4, 7, 15, 16, 255, 0x3F800000)


def rand32(rng):
    return rng.choice(I32) if rng.random() < 0.5 else rng.randint(-0x80000000, 0x7FFFFFFF)


def i32(w, off):
    return int.from_bytes(w[off:off + 4], "little", signed=True)


def i16(w, off):
    return int.from_bytes(w[off:off + 2], "little", signed=True)


def i8(w, off):
    return int.from_bytes(w[off:off + 1], "little", signed=True)


def u16(w, off):
    return int.from_bytes(w[off:off + 2], "little")


def u8(w, off):
    return w[off]


def token(v, spans):
    """A recorded value: a pointer into a window by the window's name and
    offset (`m+0x0`, `p0+0x10`), a local by its depth (`sp-0x14`), anything
    else as a signed word (item 2281)."""
    v &= 0xFFFFFFFF
    for name, at, size in spans:
        if at <= v <= at + size:
            return f"{{name}}+{{v - at:#x}}"
    # A caller's local handed to a callee: its depth under the caller's
    # `esp`, which the harness holds every row (item 2336).
    top = STACK_BASE + STACK_SIZE - 0x100
    if top - 0x1000 <= v < top:
        return f"sp-{{top - v:#x}}"
    return str(v - (1 << 32) if v & 0x80000000 else v)


def by_row(calls, base, stride, n, spans):
    """`--record IDX@S` (item 2309): a stub's calls `(ecx, args)` split by
    the row of a sparse table their `ecx` lies in — one token a row, `ecx`
    written `r+OFF` within it, `-` for a row not called — and a last token
    for the calls whose `ecx` lies outside the table."""
    cols = [[] for _ in range(n + 1)]
    for ecx_, args_ in calls:
        k = ((ecx_ & 0xFFFFFFFF) - base) // stride
        if 0 <= k < n:
            head = f"r+{{(ecx_ & 0xFFFFFFFF) - base - k * stride:#x}}"
        else:
            k, head = n, token(ecx_, spans)
        cols[k].append(",".join([head] + [token(v, spans) for v in args_]))
    return ["/".join(col) or "-" for col in cols]


def in_order(calls, record, names, spans, split=None):
    """`--record-order` (item 2341): the recorded stubs' calls as one token
    a call, in the order they were made, each headed by the callee's stub
    index (`s3:m+0x0`), so two argument-less callees on one `ecx` are
    ordered (`add_think_order`, parked 2318); `-` when none was called.
    `split` maps a stub index to its `--sparse` table `(base, stride, n)`
    (`--record IDX@S`, item 2421): that stub's `ecx` is written `rK+OFF`,
    K the table row, when it lies inside the table."""
    want = {{names[idx]: idx for idx in record}}
    split = split or {{}}

    def head(i, e):
        if i in split:
            base, stride, n = split[i]
            k = ((e & 0xFFFFFFFF) - base) // stride
            if 0 <= k < n:
                return f"r{{k}}+{{(e & 0xFFFFFFFF) - base - k * stride:#x}}"
        return token(e, spans)
    return [f"s{{want[n]}}:" + ",".join([head(want[n], e)] + [token(v, spans) for v in a])
            for n, e, a in calls if n in want] or ["-"]


def out_answer(inner, k, v, write):
    """`--outarg IDX:K:VALS` (item 2325): a stub that writes `v` through its
    argument K — the address of the caller's local, as `get_inside` writes
    the player index — unless that argument is null, then answers as
    `inner` would (a value, or a function of `ecx` and the arguments)."""
    def answer(ecx, args):
        if args[k]:
            write(args[k] & 0xFFFFFFFF, v)
        return inner(ecx, args) if callable(inner) else inner
    return answer


def win_write(inner, at, size, v, write, read=None):
    """`--stub-write IDX:WIN+OFF:SIZE:VAL` (item 2346): a stub that writes VAL
    at a window's word each call — a callee that unlinks the head of the
    container its caller loops over (`Object::kill_contents`) — then answers
    as `inner` would."""
    def answer(ecx, args):
        if v == "dec":
            # `dec` (item 2749, parked 2734): one less each call, a count a
            # callee (`remove_current`, `Array::remove`) lowers.
            w = (int.from_bytes(read(at, size), "little") - 1) & ((1 << 8 * size) - 1)
            write(at, w.to_bytes(size, "little"))
        else:
            write(at, (v & ((1 << 8 * size) - 1)).to_bytes(size, "little"))
        return inner(ecx, args) if callable(inner) else inner
    return answer


def read_arg(inner, k, n, read, sink, written=None):
    """`--deref IDX:K:N` (item 2379, parked 2348): a stub that reads the N
    bytes its argument K points at when it is called — a caller's local
    `Command`, gone once the call returns — into `sink` as hex (`null` for a
    null argument), then answers as `inner` would. `written` (item 2644,
    parked 2636) lists the byte indices the function wrote: every other byte
    of a local is stack residue and prints `00`."""
    def hexed(b):
        if written is not None:
            b = bytes(c if i in written else 0 for i, c in enumerate(b))
        return b.hex()

    def answer(ecx, args):
        if isinstance(k, tuple):  # `@VA` (item 2513): an absolute cell, read as it stands now
            sink.append(hexed(read(k[1], n)))
            return inner(ecx, args) if callable(inner) else inner
        a = args[k] & 0xFFFFFFFF
        sink.append(hexed(read(a, n)) if a else "null")
        return inner(ecx, args) if callable(inner) else inner
    return answer


def wdraw(rng, vals):
    """A bias's value: a set's member, or a range's — its own edges and the
    small integers inside it three rows in ten, so a boundary inside a wide
    range is drawn on purpose rather than by luck (Loop 2003, item 2181)."""
    if vals[0] == "set":
        return rng.choice(vals[1])
    lo, hi = vals[1], vals[2]
    if rng.random() < 0.3:
        return rng.choice([v for v in (lo, lo + 1, -1, 0, 1, 2, hi - 1, hi) if lo <= v <= hi])
    return rng.randint(lo, hi)


def seq_values(rng, n, pick, terms, rand32, span=None):
    """`--seq IDX:N[:T,T..]`'s answers a row: N drawn from the stub's
    `--answers` list (90%) or a random word, the last from the terminators
    T when given — answered past N, so a loop that runs while the stub
    answers inside a set ends (`Unit::repath`, item 2368, parked 2342)."""
    # `span` (`--answer-range` beside `--seq`, item 2421): every answer in LO..HI.
    vals = [rng.randint(*span) if span else rng.choice(pick) if pick and rng.random() < 0.9 else rand32(rng)
            for _ in range(n)]
    if terms:
        vals[-1] = rng.choice(terms)
    return vals


def shape(rng, name, buf):
    """The window `name`'s fills, then its biases, every row (parked 2113)."""
    for win, expr in FILLS:
        if win == name:
            for i in range(len(buf) // 4):
                buf[4 * i:4 * i + 4] = (eval(expr, {{"i": i}}) & 0xFFFFFFFF).to_bytes(4, "little")
    for win, off, vals in WBIAS:
        if win != name:
            continue
        for o in (range(0, len(buf) - 3, 4) if off is None else (off,)):
            v = wdraw(rng, vals)
            buf[o:o + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")


def main(argv):
    from hooks import Harness
    rng = random.Random({item})
    # `--stub-cdecl IDX`: the callee's arguments are the caller's to pop (item 2325).
    h = Harness(argv[1], {{a: (f"{{a:x}}", n, False, i in CDECLSTUBS) for i, (k, a, n) in enumerate(STUBS)
                          if k == "call"}})
    uc = h.m.uc
    slots = {{off: (f"slot{{off:x}}", n, 0) for k, off, n in STUBS if k == "slot"}}
    vt = h.vtable(slots) if slots else None
    names = [f"slot{{a:x}}" if k == "slot" else f"iat{{a:x}}" if k == "iat" else f"run{{a:x}}" if k == "run" else f"{{a:x}}" for k, a, _ in STUBS]
    # `--stub iat:VA:N`: a stub whose address is laid in an import's IAT cell,
    # which the emulator resolves to nothing, so `call [VA]` lands on it
    # (item 2336: `timeGetTime`, parked 2327).
    iats = [(a, h.stub(f"iat{{a:x}}", n, 0)) for k, a, n in STUBS if k == "iat"]
    # `--stub-cdecl IDX` on an `iat` stub (`malloc`: `call [VA]; add esp, 4`).
    for (_, sa), (i, (k, _, _)) in zip(iats, [e for e in enumerate(STUBS) if e[1][0] == "iat"]):
        if i in CDECLSTUBS:
            h.hooked[sa] = h.hooked[sa][:3] + (True,)
    stubbed = {{off: h.r32(vt + off) & 0xFFFFFFFF for off, _ in SLOTBASE}}
    base = h.brk
    for i in range({rows}):
        h.brk = base  # the bump heap is reused a row; {rows} rows of windows outgrow it
        this = h.alloc(WIN + 0x20)
        mem = bytearray(rng.randrange(256) for _ in range(WIN))
        for off, vals in BIAS:
            v = rng.choice(vals) if rng.random() < 0.9 else rng.randint(-0x80000000, 0x7FFFFFFF)
            mem[off:off + 4] = (v & 0xFFFFFFFF).to_bytes(4, "little")
        shape(rng, "m", mem)
        if vt is not None and VTAT is None:
            mem[0:4] = vt.to_bytes(4, "little")
        args = [rand32(rng) for _ in range(OWN)]
        for idx, vals in ARGSET:
            args[idx] = rng.choice(vals) if rng.random() < 0.9 else args[idx]
        for idx, off, size in ARGFROM:
            if rng.random() < 0.45:
                args[idx] = int.from_bytes(mem[off:off + size], "little", signed=True)
        for idx, lo, hi in ARGRANGE:
            args[idx] = rng.randint(lo, hi)
        wins = []
        for va, size in CELLS:
            rec = h.alloc(size + 0x20)
            buf = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(rec, bytes(buf))
            uc.mem_write(va, rec.to_bytes(4, "little"))
            wins.append((rec, buf))
        for va, size in AT:
            buf = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(va, bytes(buf))
            wins.append((va, buf))
        gelems = []
        for cnt, lo, hi, ptr, esize in GTAB:
            # A counted array behind image words (item 2709, parked 2699): the count
            # drawn from LO..HI at CNT, the array's pointer at PTR, one record of
            # ESIZE random bytes an element — the head CNT..PTR+4 and the array are
            # `c` windows, the records `p` windows after the pointer arguments'.
            n = rng.randint(lo, hi)
            recs = [h.alloc(esize + 0x20) for _ in range(max(n, 0))]
            arr = h.alloc(4 * max(n, 1) + 0x20)
            head = bytearray(rng.randrange(256) for _ in range(ptr + 4 - cnt))
            head[0:4] = (n & 0xFFFFFFFF).to_bytes(4, "little")
            head[ptr - cnt:] = arr.to_bytes(4, "little")
            body = bytearray(b"".join(r.to_bytes(4, "little") for r in recs) or bytes(4))
            wins.append((cnt, head))
            wins.append((arr, body))
            gelems += [(r, bytearray(rng.randrange(256) for _ in range(esize))) for r in recs]
        for idx, size in PTRARGS:
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            uc.mem_write(buf, bytes(b))
            args[idx] = buf
            # `--ptrarg-null IDX`: the argument laid as 0 a third of the rows,
            # its buffer still drawn so every later window keeps its index
            # (item 2271; a guarded null argument ran one arm).
            if idx in PNULLS and rng.random() < 1 / 3:
                args[idx] = 0
            wins.append((buf, b))
        wins += gelems
        nc = len(CELLS) + len(AT) + 2 * len(GTAB)
        c = [b for _, b in wins[:nc]]
        p = [b for _, b in wins[nc:]]
        for k, b in enumerate(c):
            shape(rng, f"c{{k}}", b)
        for k, b in enumerate(p):
            shape(rng, f"p{{k}}", b)
        tables = []
        for win, loc, stride, n, slices in SPARSE:
            # A table of N rows STRIDE apart of which the function reads a
            # few slices: the table zeroed, each row's slices `p` windows, the
            # pointer laid at LOC — or, `@`, the slices written at the image's
            # own table (parked 2201, item 2247: the leaders table).
            env = {{"a": args, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}}
            if win == "@":
                tbase = eval(loc, env) & 0xFFFFFFFF
            else:
                tbase = h.alloc(stride * n + 0x20)
                uc.mem_write(tbase, bytes(stride * n))
            tables.append((tbase, stride, n))
            for k in range(n):
                for off, size in slices:
                    b = bytearray(rng.randrange(256) for _ in range(size))
                    p.append(b)
                    wins.append((tbase + k * stride + off, b))
                    shape(rng, f"p{{len(p) - 1}}", b)
            if win != "@":
                ptr = tbase.to_bytes(4, "little")
                if win is None:
                    uc.mem_write(eval(loc, env) & 0xFFFFFFFF, ptr)
                else:
                    w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                    at = int(loc, 0)
                    w[at:at + 4] = ptr
        # The stubs' answers are drawn before the windows are placed, so a
        # placed table may be indexed by one (`s`, item 2160); `--answers`
        # draws stub IDX's from a list (parked 2141).
        answers = [rand32(rng) for _ in STUBS]
        for idx, vals in ANSWERS:
            answers[idx] = rng.choice(vals) if rng.random() < 0.9 else answers[idx]
        # `--answer-range`: an answer the function indexes a table by, held
        # to the range every row (item 2217).
        for idx, lo, hi in ANSRANGE:
            answers[idx] = rng.randint(lo, hi)
        # `--outarg`: the values stubs write through a pointer argument, drawn
        # with the answers so a place may index by one (`o`, item 2325).
        outs = [wdraw(rng, vals) for _, _, vals in OUTARG]
        k = -1
        for win, loc, expr, size in PLACES:
            # A table pointer laid as `buffer - (the offset the function adds)`,
            # so its read lands in a buffer of SIZE random bytes (parked 2113).
            k += 1
            env = {{"a": args, "s": answers, "o": outs, "m": mem, "c": c, "p": p,
                   "i32": i32, "i16": i16, "i8": i8, "u16": u16, "u8": u8}}
            off = eval(expr, env)
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{{len(p) - 1}}", b)
            ptr = ((buf - off) & 0xFFFFFFFF).to_bytes(4, "little")
            # `--place-null K`: place K's pointer laid as 0 a third of the
            # rows, the buffer still drawn so every later window keeps its
            # index (item 2253; a guarded null ran one arm).
            if k in NULLS and rng.random() < 1 / 3:
                ptr = bytes(4)
            at = eval(loc, env)
            if win is None:
                uc.mem_write(at & 0xFFFFFFFF, ptr)
            elif win[0] == "s":
                # A stub answers the placed buffer, a record a later place
                # may reach into (item 2241).
                answers[int(win[1:])] = (buf - off) & 0xFFFFFFFF if any(ptr) else 0
            else:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[at:at + 4] = ptr
        callwins = {{}}
        for idx, size in ANSWERWIN:
            # A stub that answers a record: the address of SIZE random bytes,
            # a new `p` window after the places (parked 2151, item 2170).
            buf = h.alloc(size + 0x20)
            b = bytearray(rng.randrange(256) for _ in range(size))
            p.append(b)
            wins.append((buf, b))
            shape(rng, f"p{{len(p) - 1}}", b)
            answers[idx] = buf
            callwins.setdefault(idx, []).append(buf)
        if VTAT is not None:
            # The stubbed vtable laid in a window: a slot of an object the
            # function reaches through a table, not of `this` (item 2170).
            win, off = VTAT
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off in VTALSO:
            # The stubbed vtable laid in a window as well, `this` keeping it:
            # a record the function reaches whose slots it asks too (item 2217).
            w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
            w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off in VTATHALF:
            # The stubbed vtable laid half the rows at a record the function
            # compares with a class's vtable, the other half left as drawn
            # (a `--wbias` of the class's address; item 2223).
            if rng.random() < 0.5:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = vt.to_bytes(4, "little")
        for win, off, tgt, plus in ALIAS:
            # `--alias`: the word laid as a window's own address (plus K) half the rows, the other half left as
            # drawn: a pointer that names another window, or its own (item 2797, parked 2791).
            if rng.random() < 0.5:
                at_ = this if tgt == "m" else wins[(0 if tgt[0] == "c" else nc) + int(tgt[1:])][0]
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                w[off:off + 4] = ((at_ + plus) & 0xFFFFFFFF).to_bytes(4, "little")
        for lo, hi, off, size, d, terms in LINKS:
            # A link field (`o_up`, a type's upgrade) of figures p(lo)..p(hi),
            # drawn from the figures on one side of each one's own index or a
            # terminator, so a walk through them ends (item 2276).
            n = hi - lo + 1
            for k in range(n):
                v = link_value(rng, n, k, d, terms)
                p[lo + k][off:off + size] = (v & ((1 << 8 * size) - 1)).to_bytes(size, "little")
        for idx, win, off in ARGEQ:
            # An argument the function compares with a word of a window laid
            # after the arguments were drawn: the word copied (item 2200).
            if rng.random() < 0.45:
                w = mem if win == "m" else (c if win[0] == "c" else p)[int(win[1:])]
                args[idx] = i32(w, off)
        # `--ring` (item 2400, parked 2369): a circular doubly linked list
        # laid in the `c` windows LO..HI at BASE + STRIDE k. The head is
        # the word at HWIN+HOFF; each node's `prev` is its predecessor on
        # the `next` walk from the head (the head's, the last node), so a
        # walk stopping at the head ends on the wrap; a fifth of the rows
        # then point one later node's `prev` at the head, an early stop.
        for hwin, hoff, lo_, hi_, noff, poff, base_, stride_ in RING:
            hw = mem if hwin == "m" else (c if hwin[0] == "c" else p)[int(hwin[1:])]
            head = i32(hw, hoff) & 0xFFFFFFFF
            node = lambda a_: (a_ - base_) // stride_
            if head and base_ <= head < base_ + stride_ * (hi_ - lo_ + 1) and (head - base_) % stride_ == 0:
                walk, seen = [node(head)], {{node(head)}}
                while True:
                    a_ = i32(c[lo_ + walk[-1]], noff) & 0xFFFFFFFF
                    if not (base_ <= a_ < base_ + stride_ * (hi_ - lo_ + 1)) or (a_ - base_) % stride_:
                        break
                    nxt = node(a_)
                    if nxt in seen:
                        break
                    walk.append(nxt)
                    seen.add(nxt)
                for j, k in enumerate(walk):
                    c[lo_ + k][poff:poff + 4] = (base_ + stride_ * walk[j - 1]).to_bytes(4, "little")
                if len(walk) > 1 and rng.random() < 0.2:
                    c[lo_ + rng.choice(walk[1:])][poff:poff + 4] = head.to_bytes(4, "little")
        for a_, b in wins:
            uc.mem_write(a_, bytes(b))
        for cell, stub in iats:
            uc.mem_write(cell, stub.to_bytes(4, "little"))
        based = []
        for off, addr in SLOTBASE:
            on = rng.random() < 0.5
            h.w32(vt + off, addr if on else stubbed[off])
            based.append(int(on))
        for name, v in zip(names, answers):
            h.answers[name] = v & 0xFFFFFFFF
        # A stub with several `--answer-win` windows answers one a call, in
        # call order, the last repeated past them (parked 2623, item 2633).
        for idx, bufs in callwins.items():
            if len(bufs) > 1:
                turn = [0]

                def answer(ecx, args, bufs=bufs, turn=turn):
                    v = bufs[min(turn[0], len(bufs) - 1)]
                    turn[0] += 1
                    return v
                h.answers[names[idx]] = answer
        # A callee called more than once a row answers each call in turn:
        # N values drawn a row (from its `--answers` list when it has one),
        # the last repeated past N — from the terminators when given (item
        # 2368) — printed after the answers (item 2217).
        seqs = []
        for idx, n, terms in SEQ:
            vals = seq_values(rng, n, dict(ANSWERS).get(idx), terms, rand32,
                              {{i: (lo, hi) for i, lo, hi in ANSRANGE}}.get(idx))
            seqs.append(vals)
            turn = [0]

            def answer(ecx, args, vals=vals, turn=turn):
                v = vals[min(turn[0], len(vals) - 1)]
                turn[0] += 1
                return v & 0xFFFFFFFF
            h.answers[names[idx]] = answer
        for (idx, k, _), v in zip(OUTARG, outs):
            h.answers[names[idx]] = out_answer(h.answers[names[idx]], k, v,
                                               lambda a_, v_: uc.mem_write(a_, (v_ & 0xFFFFFFFF).to_bytes(4, "little")))
        # `--ecx-answer IDX:OFF[:FLAGOFF]` (item 2400): a slot answering from
        # its own `ecx`: the word at `ecx+OFF`, or with FLAGOFF `ecx+OFF`
        # itself when the word at `ecx+FLAGOFF` is set, else null.
        for idx, off, flag in ECXANSWER:
            if flag is None:
                h.answers[names[idx]] = lambda ecx, args, off=off: h.r32(ecx + off) & 0xFFFFFFFF
            else:
                h.answers[names[idx]] = lambda ecx, args, off=off, flag=flag: (ecx + off) & 0xFFFFFFFF if h.r32(ecx + flag) else 0
        for idx, win, off, size, v in STUBWRITE:
            at = this if win == "m" else wins[(0 if win[0] == "c" else nc) + int(win[1:])][0]
            h.answers[names[idx]] = win_write(h.answers[names[idx]], at + off, size, v, uc.mem_write,
                                                      lambda a_, n_: bytes(uc.mem_read(a_, n_)))
        derefs = [[] for _ in DEREF]
        for (idx, k, n, wr), sink in zip(DEREF, derefs):
            h.answers[names[idx]] = read_arg(h.answers[names[idx]], k, n,
                                             lambda a_, n_: bytes(uc.mem_read(a_, n_)), sink, wr)
        uc.mem_write(this, bytes(mem))
        # A global cell that holds `this` itself: a method that reads the
        # global as well as its object sees one object (item 2217).
        for va in CELLTHIS:
            uc.mem_write(va, this.to_bytes(4, "little"))
        guard = bytes(uc.mem_read(this + WIN, 0x20))
        # `ecx` an integer rather than the object (parked 2141), the row's last.
        ecx = None if ECXINT is None else draw_reg(rng, ECXINT, WILD, rand32)
        # `--edx` (item 2346): an integer in `edx` beside `ecx`, after it on the row.
        edx = None if EDX is None else draw_reg(rng, EDX, WILD, rand32)
        # `--edx-win` (item 2801): `edx` the address of a window, not an integer.
        edxa = 0 if EDX is None else edx
        if EDXWIN is not None:
            edxa = this if EDXWIN == "m" else wins[(0 if EDXWIN[0] == "c" else nc) + int(EDXWIN[1:])][0]
        h.calls.clear()
        h.edxs.clear()
        r = h.call(ENTRY, tuple(args), ecx=this if ECXINT is None else ecx,
                   edx=edxa, ret=RET)  # Harness.call signs eax; signed() again would double a negative
        esp = uc.reg_read(UC_X86_REG_ESP)
        # A `__cdecl` function leaves its arguments for the caller (item 2181).
        want_esp = STACK_BASE + STACK_SIZE - 0x100 - (4 * len(args) if CDECL else 0)
        assert esp == want_esp, f"esp {{esp:x}}: wrong pop"
        assert bytes(uc.mem_read(this + WIN, 0x20)) == guard, "wrote past the window"
        before = ";".join([bytes(mem).hex()] + [bytes(b).hex() for _, b in wins])
        after = ";".join([bytes(uc.mem_read(this, WIN)).hex()]
                         + [bytes(uc.mem_read(a, len(b))).hex() for a, b in wins])
        shown = ([str(a) for a in args] + [str(v) for v in answers] + [str(v) for v in outs]
                 + [str(v) for s in seqs for v in s] + [str(f) for f in based]
                 + ([] if ECXINT is None else [str(ecx)]) + ([] if EDX is None else [str(edx)]))
        # `--record IDX`: what stub IDX was handed, each call `ecx,arg..`,
        # calls joined by `/`, `-` when it was not called, after ` # `
        # (item 2281: a tail target's `ecx` and argument, parked 2272).
        log = ""
        if RECORD or DEREF:
            # `this` past the window (`this + 0x28`, a member's address) is last,
            # so it never shadows a window the heap laid after the object (item 2464).
            spans = [("m", this, WIN)] + [(f"c{{k}}" if k < nc else f"p{{k - nc}}", a_, len(b))
                                          for k, (a_, b) in enumerate(wins)] + [("this", this, 0x1000)]
            cols = []
            if RECORDORDER:
                cols = in_order([(n_, e_, a_) for n_, e_, a_, _ in h.calls], RECORD, names, spans,
                                {{i: tables[t] for i, t in ROWSPLIT.items()}})
            for idx in ([] if RECORDORDER else RECORD):
                calls = [(ecx_, list(args_)) for name_, ecx_, args_, _ in h.calls if name_ == names[idx]]
                if RECORDEDX:
                    # `--record-edx` (item 2400): `ecx,edx,arg..` a call.
                    cols.append("/".join(",".join(token(v, spans) for v in [e, d] + list(a_))
                                         for (n_, e, a_, _), d in zip(h.calls, h.edxs) if n_ == names[idx]) or "-")
                elif idx in ROWSPLIT:
                    cols += by_row(calls, *tables[ROWSPLIT[idx]], spans)
                else:
                    cols.append("/".join(",".join(token(v, spans) for v in [e] + a_)
                                         for e, a_ in calls) or "-")
            # `--deref`: the bytes each call's argument pointed at (item 2379).
            cols += ["/".join(sink) or "-" for sink in derefs]
            log = " # " + " ".join(cols)
        print(" ".join(shown) + f" | {{before}} -> {{r}} | {{after}}" + log)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
'''

OBJ_WIN_TEST = '''//! `{name}@{va:08x}` under the emulator against the port (item {item}; tools/sweep_gen.py, obj with windows).

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
{logsplit}        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off({nc});
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
{logdecl}        let got: i64 = {{
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {port}
        }};
{ret}
{logcheck}        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "{short} — this's bytes, row `{{line}}`");
        for (i, w) in c.iter().enumerate() {{
            assert_eq!(w, &want[1 + i], "{short} — window c[{{i}}], row `{{line}}`");
        }}
        for (i, w) in p.iter().enumerate() {{
            assert_eq!(w, &want[{pbase} + i], "{short} — window p[{{i}}], row `{{line}}`");
        }}
        n += 1;
    }}
    assert!(n >= {floor}, "only {{n}} rows");
    eprintln!("{name}@{va:08x}: {{n}} rows agree");
}}
'''


def _window(spec, default):
    va, _, size = spec.partition(':')
    return int(va, 0), int(size, 0) if size else default


def _stub(spec):
    parts = spec.split(':')
    if parts[0] == 'slot' and len(parts) == 3:
        return ('slot', int(parts[1], 0), int(parts[2]))
    if parts[0] == 'iat' and len(parts) == 3:
        # An import called through its IAT cell (`call [VA]`): a stub's
        # address laid in the cell (item 2336, parked 2327).
        return ('iat', int(parts[1], 0), int(parts[2]))
    if parts[0] == 'run' and len(parts) == 3:
        # A callee run as the original's own code, not hooked (item 2731, parked
        # 2701, 2712, 2720): its writes to `this` are the original's, and its own
        # callees are hooked beside it by `--stub`.
        return ('run', int(parts[1], 0), int(parts[2]))
    if len(parts) == 2:
        return ('call', int(parts[0], 0), int(parts[1]))
    raise ValueError(f'a stub is slot:OFF:NARGS, iat:VA:NARGS, run:ADDR:NARGS or ADDR:NARGS, not {spec!r}')


def _place(spec):
    """`LOC:EXPR:SIZE` — LOC a window's `m+OFF`, `cN+OFF`, `pN+OFF`, or an
    address expression; EXPR the byte offset the function adds (parked 2113)."""
    loc, expr, size = spec.split(':')
    if re.fullmatch(r's\d+', loc):
        # A stub's answer laid as the placed buffer: a record the callee
        # answers, which a later place may reach into (item 2241).
        return (loc, '0', expr, int(size, 0))
    win, plus, off = loc.partition('+')
    if plus and re.fullmatch(r'm|c\d+|p\d+', win):
        return (win, off, expr, int(size, 0))
    if plus and re.fullmatch(r'[a-z]\d*', win):
        raise ValueError(f'a placed window is m, cN or pN, not {win!r}')
    return (None, loc, expr, int(size, 0))


def _placed(win, off):
    if win and win[0] == 's':
        return f'stub {win[1:]}\'s answer'
    return f'the word at `{win}+{off}`' if win else f'the word at `{off}`'


def _gtab(spec):
    """`CNT:LO..HI:PTR:SIZE` — a count word at the image address CNT drawn from
    LO..HI, the array's pointer at PTR (above CNT), SIZE random bytes an element
    (item 2709, parked 2699)."""
    m = re.fullmatch(r'(\w+):(-?\w+)\.\.(-?\w+):(\w+):(\w+)', spec)
    if not m:
        raise ValueError(f'--gtab is CNT:LO..HI:PTR:SIZE, not {spec!r}')
    cnt, lo, hi, ptr, size = (int(x, 0) for x in m.groups())
    if not cnt < ptr or not lo <= hi or size < 0x20 or hi > 64:
        raise ValueError(f'--gtab wants CNT < PTR, LO <= HI <= 64 and SIZE at least 0x20, not {spec!r}')
    return (cnt, lo, hi, ptr, size)


def _sparse(spec):
    """`LOC:STRIDE:N:OFF,SIZE[/OFF,SIZE..]` — LOC a window's `m+OFF`, `cN+OFF`,
    `pN+OFF`, an address expression, or `@VA` for a table in the image itself
    (parked 2201, item 2247)."""
    loc, stride, n, sl = spec.split(':')
    stride, n = int(stride, 0), int(n, 0)
    slices = sorted(tuple(int(x, 0) for x in one.split(',')) for one in sl.split('/'))
    end = 0
    for off, size in slices:
        if off < end or off + size > stride or size < 1:
            raise ValueError(f'--sparse slices overlap or outrun the stride {stride:#x}: {spec!r}')
        end = off + size
    if loc.startswith('@'):
        return ('@', loc[1:], stride, n, slices)
    win, plus, off = loc.partition('+')
    if plus and re.fullmatch(r'm|c\d+|p\d+', win):
        return (win, off, stride, n, slices)
    return (None, loc, stride, n, slices)


def _link(spec):
    """`pA..B+OFF:SIZE:DIR[:T,T..]` — figure k is window p(A+k); its link at
    OFF, SIZE bytes, drawn below (`<`) or above (`>`) k or a terminator,
    -1 by default (item 2276)."""
    loc, size, d, *terms = spec.split(':')
    m = re.fullmatch(r'p(\d+)\.\.(\d+)\+(\w+)', loc)
    if not m or int(m.group(1)) > int(m.group(2)):
        raise ValueError(f'--link names a run of figures pA..B+OFF, A <= B, not {loc!r}')
    if size not in ('1', '2', '4') or d not in ('<', '>') or len(terms) > 1:
        raise ValueError(f'--link is pA..B+OFF:1|2|4:<|>[:T,T..], not {spec!r}')
    terms = [int(t, 0) for t in terms[0].split(',')] if terms else [-1]
    return (int(m.group(1)), int(m.group(2)), int(m.group(3), 0), int(size), d, terms)


def _wbias(spec):
    """`WIN+OFF:V,V,..` or `WIN+OFF:LO..HI`, OFF `*` for every word, held every row."""
    loc, vals = spec.rsplit(':', 1)
    win, _, off = loc.partition('+')
    run = re.fullmatch(r'p(\d+)\.\.(\d+)', win)
    if run:
        # `pA..B+OFF`: every window of a run, a sparse table's rows (item 2247).
        return [w for k in range(int(run.group(1)), int(run.group(2)) + 1)
                for w in _wbias(f'p{k}+{off}:{vals}')]
    if not re.fullmatch(r'm|c\d+|p\d+', win):
        raise ValueError(f'a biased window is m, cN or pN, not {win!r}')
    o = None if off == '*' else int(off, 0)
    if '..' in vals:
        lo, hi = vals.split('..')
        return [(win, o, ('range', int(lo, 0), int(hi, 0)))]
    return [(win, o, ('set', [int(v, 0) for v in vals.split(',')]))]


def generate_obj(spec, *, item, win, bias, own=0, ret='eax', port=None, argset=(), argfrom=(),
                 cells=(), at=(), ptrargs=(), stubs=(), doc=None, places=(), wbias=(), fills=(),
                 argrange=(), slotbase=(), module=None, ecx_int=None, answers=(), vtat=None, answer_wins=(),
                 cdecl=False, argeq=(), seq=(), answer_range=(), cell_this=(), vtat_also=(), vtat_half=(), alias=(),
                 place_null=(), sparse=(), ptrarg_null=(), record=(), links=(), outarg=(), stub_cdecl=(),
                 rows=220, record_order=False, record_edx=False, edx=None, edx_win=None, tame=False, stub_write=(), deref=(), ecx_answer=(), ring=(), gtab=()):
    name, va = parse(spec)
    if rows < 200:
        raise ValueError('--rows N is at least the 200 rows the test asks for')
    floor = 200 if rows == 220 else rows
    rowdoc = '' if rows == 220 else f' {rows} rows (an arm few rows reach).'
    short = name.rsplit('::', 1)[-1]
    sn = module or snake(name)
    parsed = []
    for b in bias:
        off, vals = b.split(':', 1)
        parsed.append((int(off, 0), [int(v, 0) for v in vals.split(',')]))
    pset = [(int(a.split(':', 1)[0]), [int(v, 0) for v in a.split(':', 1)[1].split(',')]) for a in argset]
    pfrom = [tuple(int(x, 0) for x in a.split(':')) for a in argfrom]
    port = port or f'todo!("the port of {name}")'
    biasdoc = ', '.join(f'+{o:#x} in {v}' for o, v in parsed) or 'no bias'
    if ret == 'none':
        retcode = '        let _ = (got, want_eax);'
    elif ret == 'al':
        # A `bool` in `al` over an `eax` whose upper bytes are a pointer's
        # (`sete al`; item 2336, parked 2327): the low byte alone is held.
        retcode = f'        assert_eq!(got & 0xff, want_eax & 0xff, "{short} — al, row `{{line}}`");'
    elif ret in ('eax', 'xmm0'):
        retcode = f'        assert_eq!(got, want_eax, "{short} — eax, row `{{line}}`");'
    elif ret.startswith('arg:') and ret[4:].isdigit() and int(ret[4:]) < own:
        # An out-pointer returned (the inlined `String` constructors, parked 2594, item 2604):
        # eax is argument N, whatever the buffer's address; the stores are the windows' bytes after.
        retcode = (f'        assert_eq!(want_eax, i64::from(a[{int(ret[4:])}]), "{short} — eax is the out-pointer, row `{{line}}`");\n'
                   '        let _ = got;')
    else:
        raise ValueError(f'obj --ret is eax, xmm0, al, none or arg:N (N a stack argument), not {ret!r}')
    pcells = [_window(c, 0x400) for c in cells]
    pat = [_window(t, None) for t in at]
    if any(size is None for _, size in pat):
        raise ValueError('--at VA:SIZE names the table\'s size')
    pptr = [_window(p, 0x100) for p in ptrargs]
    if any(idx >= own for idx, _ in pptr):
        raise ValueError('--ptrarg IDX names one of the --own arguments')
    ppnull = [int(k, 0) for k in ptrarg_null]
    if any(k not in {idx for idx, _ in pptr} for k in ppnull):
        raise ValueError('--ptrarg-null IDX names one of the --ptrarg arguments')
    pstubs = [_stub(s) for s in stubs]
    pplaces = [_place(s) for s in places]
    pnull = [int(k, 0) for k in place_null]
    if any(not 0 <= k < len(pplaces) for k in pnull):
        raise ValueError('--place-null K names one of the --place pointers, by its order on the line')
    pwbias = [w for s in wbias for w in _wbias(s)]
    psparse = [_sparse(s) for s in sparse]
    pgtab = [_gtab(s) for s in gtab]
    plinks = [_link(s) for s in links]
    pfills = [tuple(f.split(':', 1)) for f in fills]
    prange = [tuple(int(x, 0) for x in r.split(':')) for r in argrange]
    pslot = [tuple(int(x, 0) for x in s.split(':')) for s in slotbase]
    pans = [(int(a.split(':', 1)[0]), [int(v, 0) for v in a.split(':', 1)[1].split(',')]) for a in answers]
    if any(not 0 <= idx < len(pstubs) for idx, _ in pans):
        raise ValueError('--answers IDX:V,.. names one of the --stub callees, by its order on the line')
    if any(('slot', off) not in {(k, a) for k, a, _ in pstubs} for off, _ in pslot):
        raise ValueError('--slotbase OFF:ADDR names a slot --stub slot:OFF:N stubs')
    if any(w and w[0] == 's' and not int(w[1:]) < len(pstubs) for w, _, _, _ in pplaces):
        raise ValueError('--place sN:EXPR:SIZE names one of the --stub callees, by its order on the line')
    pansw = [tuple(int(x, 0) for x in a.split(':')) for a in answer_wins]
    pcdecl = [int(k, 0) for k in stub_cdecl]
    if any(not 0 <= k < len(pstubs) or pstubs[k][0] not in ('call', 'iat') for k in pcdecl):
        raise ValueError('--stub-cdecl IDX names a direct callee among the --stub callees, by its order on the line')
    pout = []
    for o in outarg:
        idx, k, vals = o.split(':', 2)
        idx, k = int(idx, 0), int(k, 0)
        if not 0 <= idx < len(pstubs):
            raise ValueError('--outarg IDX:K:VALS names one of the --stub callees, by its order on the line')
        if not 0 <= k < pstubs[idx][2]:
            raise ValueError(f'--outarg {o}: stub {idx} takes {pstubs[idx][2]} argument(s); K names one')
        pout.append((idx, k, _wbias(f'm+0:{vals}')[0][2]))
    precord, prowsplit = [], {}
    for one in record:
        k, at, tab = str(one).partition('@')
        precord.append(int(k, 0))
        if at:
            prowsplit[int(k, 0)] = int(tab, 0)
    if any(not 0 <= k < len(pstubs) for k in precord):
        raise ValueError('--record IDX names one of the --stub callees, by its order on the line')
    if any(not 0 <= t < len(psparse) for t in prowsplit.values()):
        raise ValueError('--record IDX@S names a --sparse table S, by its order on the line (item 2309)')
    pderef = []
    for one in deref:
        i_, k_, n_, *w_ = str(one).split(':')
        idx, n = int(i_, 0), int(n_, 0)
        wr = None
        if w_:
            # `IDX:K:N:W` (item 2644, parked 2636): W the written byte ranges, `0,1-4`.
            got = set()
            for part in w_[0].split(','):
                lo, _, hi = part.partition('-')
                lo, hi = int(lo, 0), int(hi or lo, 0)
                if not 0 <= lo <= hi < n:
                    raise ValueError(f'--deref {one}: written bytes `{part}` outside 0..{n - 1}')
                got.update(range(lo, hi + 1))
            wr = tuple(sorted(got))
        if not 0 <= idx < len(pstubs):
            raise ValueError('--deref IDX:K:N names one of the --stub callees, by its order on the line')
        if k_.startswith('@'):
            # `--deref IDX:@VA:N` (item 2513, parked 2509): an absolute cell, read at call time.
            if n < 1:
                raise ValueError(f'--deref {one}: N >= 1')
            pderef.append((idx, ('@', int(k_[1:], 0)), n, wr))
            continue
        k = int(k_, 0)
        if not 0 <= k < pstubs[idx][2] or n < 1:
            raise ValueError(f'--deref {one}: stub {idx} takes {pstubs[idx][2]} argument(s); K names one, N >= 1')
        pderef.append((idx, k, n, wr))
    if record_edx and (not precord or prowsplit or record_order):
        raise ValueError('--record-edx adds `edx` to the --record stubs\' calls; a row split or an order keeps its own columns')
    if record_order and not precord:
        raise ValueError('--record-order orders the --record stubs\' calls')
    if any(not 0 <= idx < len(pstubs) for idx, _ in pansw):
        raise ValueError('--answer-win IDX:SIZE names one of the --stub callees, by its order on the line')
    peq = []
    for e in argeq:
        idx, loc = e.split(':', 1)
        ewin, plus, eoff = loc.partition('+')
        if not (plus and re.fullmatch(r'm|c\d+|p\d+', ewin)):
            raise ValueError(f'--argeq is IDX:m+OFF, IDX:cN+OFF or IDX:pN+OFF, not {e!r}')
        if not 0 <= int(idx) < own:
            raise ValueError('--argeq IDX names one of the --own arguments')
        peq.append((int(idx), ewin, int(eoff, 0)))
    pseq = []
    for q in seq:
        # `IDX:N[:T,T..]`: the last of the N answers drawn from the
        # terminators T and answered past N (item 2368, parked 2342).
        parts = q.split(':')
        if len(parts) not in (2, 3) or (len(parts) == 3 and not all(parts[2].split(','))):
            raise ValueError(f'--seq {q}: IDX:N or IDX:N:T,T..')
        pseq.append((int(parts[0], 0), int(parts[1], 0),
                     [int(t, 0) for t in parts[2].split(',')] if len(parts) == 3 else None))
    if any(not 0 <= idx < len(pstubs) or n < 1 for idx, n, _ in pseq):
        raise ValueError('--seq IDX:N names one of the --stub callees, by its order on the line, and N >= 1')
    if any(idx in {i for i, _ in pansw} for idx, _, _ in pseq):
        raise ValueError('--seq and --answer-win answer one stub two ways')
    pansr = [tuple(int(x, 0) for x in r.split(':')) for r in answer_range]
    if any(not 0 <= idx < len(pstubs) for idx, _, _ in pansr):
        raise ValueError('--answer-range IDX:LO:HI names one of the --stub callees, by its order on the line')
    pthis = [int(v, 0) for v in cell_this]
    if set(pthis) & {va for va, _ in pcells}:
        raise ValueError('--cell-this names a cell --cell-win gives a record of its own')
    palso = []
    for v in vtat_also:
        awin, plus, aoff = v.partition('+')
        if not (plus and re.fullmatch(r'm|c\d+|p\d+', awin)):
            raise ValueError(f'--vtat-also is m+OFF, cN+OFF or pN+OFF, not {v!r}')
        if not any(k == 'slot' for k, _, _ in pstubs):
            raise ValueError('--vtat-also lays the stubbed vtable: it needs a --stub slot:OFF:N')
        palso.append((awin, int(aoff, 0)))
    pvtat = None
    if vtat is not None:
        vwin, plus, voff = vtat.partition('+')
        if not (plus and re.fullmatch(r'm|c\d+|p\d+', vwin)):
            raise ValueError(f'--vtat is m+OFF, cN+OFF or pN+OFF, not {vtat!r}')
        if not any(k == 'slot' for k, _, _ in pstubs):
            raise ValueError('--vtat lays the stubbed vtable: it needs a --stub slot:OFF:N')
        pvtat = (vwin, int(voff, 0))
    phalf = []
    for vh in vtat_half:
        vwin, plus, voff = vh.partition('+')
        if not (plus and re.fullmatch(r'm|c\d+|p\d+', vwin)):
            raise ValueError(f'--vtat-half is m+OFF, cN+OFF or pN+OFF, not {vh!r}')
        if not any(k == 'slot' for k, _, _ in pstubs):
            raise ValueError('--vtat-half lays the stubbed vtable: it needs a --stub slot:OFF:N')
        phalf.append((vwin, int(voff, 0)))
    palias = []
    for al in alias:
        loc, colon, tgt = al.partition(':')
        awin, plus, aoff = loc.partition('+')
        twin, tplus, toff = tgt.partition('+')
        if not (colon and plus and re.fullmatch(r'm|c\d+|p\d+', awin) and re.fullmatch(r'm|c\d+|p\d+', twin)):
            raise ValueError(f'--alias is WIN+OFF:TARGET[+K] over m, cN or pN, not {al!r}')
        palias.append((awin, int(aoff, 0), twin, int(toff, 0) if tplus else 0))
    pecx = None
    if ecx_int is not None:
        if '..' in ecx_int:
            lo, hi = ecx_int.split('..')
            pecx = ('range', int(lo, 0), int(hi, 0))
        else:
            pecx = ('set', [int(v, 0) for v in ecx_int.split(',')])
    pwrite = []
    for spec in stub_write:
        m_ = re.fullmatch(r'(\d+):(m|c\d+|p\d+)\+(\w+):([124]):(-?\w+)', spec)
        if not m_:
            raise ValueError(f'--stub-write is IDX:WIN+OFF:SIZE:VAL, SIZE 1, 2 or 4, not {spec!r}')
        idx = int(m_.group(1))
        if idx >= len(stubs):
            raise ValueError(f'--stub-write names stub {idx}, and there are {len(stubs)}')
        pwrite.append((idx, m_.group(2), int(m_.group(3), 0), int(m_.group(4)),
                       'dec' if m_.group(5) == 'dec' else int(m_.group(5), 0)))
    pecxans = []
    for spec in ecx_answer:
        m_ = re.fullmatch(r'(\d+):(\w+)(?::(\w+))?', spec)
        if not m_ or int(m_.group(1)) >= len(stubs):
            raise ValueError(f'--ecx-answer is IDX:OFF[:FLAGOFF], IDX naming a --stub, not {spec!r}')
        pecxans.append((int(m_.group(1)), int(m_.group(2), 0), None if m_.group(3) is None else int(m_.group(3), 0)))
    pring = []
    for spec in ring:
        m_ = re.fullmatch(r'(m|c\d+|p\d+)\+(\w+):(\d+)\.\.(\d+):(\w+):(\w+):(\w+):(\w+)', spec)
        if not m_ or int(m_.group(3)) > int(m_.group(4)):
            raise ValueError(f'--ring is HWIN+HOFF:LO..HI:NEXT:PREV:BASE:STRIDE (the c windows LO..HI), not {spec!r}')
        g = m_.groups()
        pring.append((g[0], int(g[1], 0), int(g[2]), int(g[3]), int(g[4], 0), int(g[5], 0), int(g[6], 0), int(g[7], 0)))
    pedx = None
    if edx is not None:
        if '..' in edx:
            lo, hi = edx.split('..')
            pedx = ('range', int(lo, 0), int(hi, 0))
        else:
            pedx = ('set', [int(v, 0) for v in edx.split(',')])
    if edx_win is not None:
        if not re.fullmatch(r'm|c\d+|p\d+', edx_win):
            raise ValueError(f'--edx-win is m, cN or pN, not {edx_win!r}')
        if pedx is not None:
            raise ValueError('--edx-win and --edx both fill `edx`: name one')
    if tame and pecx is None and pedx is None:
        raise ValueError('--tame holds --ecx-int or --edx to its draw: name one')
    if not (pcells or pat or pptr or pstubs or pplaces or pwbias or pfills or prange or pslot or pansw or peq
            or pseq or pansr or pthis or palso or phalf or palias or edx_win or psparse or pgtab or plinks or pout or pcdecl or pwrite):
        if pecx is not None:
            raise ValueError('--ecx-int needs a window: the plain obj has no row for it')
        if pedx is not None:
            raise ValueError('--edx needs a window: the plain obj has no row for it')
        if cdecl:
            raise ValueError('--cdecl needs a window: the plain obj keeps the callee-pops check')
        script = OBJ_SCRIPT.format(snake=sn, name=name, va=va, item=item, win=win, own=own, bias=parsed,
                                   biasdoc=biasdoc, argset=pset, argfrom=pfrom, rows=rows)
        doc = (doc or f'{win} random bytes of object, {own} argument(s) over the edges and random words, `eax` and every byte after held to the port.') + rowdoc
        test = OBJ_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc, port=port, ret=retcode,
                               floor=floor)
        return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'
    windoc = '; '.join(
        [f'the global cell 0x{va:08x} points at {size} random bytes' for va, size in pcells]
        + [f'{size} random bytes at 0x{va:08x}' for va, size in pat]
        + [f'argument {idx} names {size} random bytes' + (' (null a third of the rows)' if idx in ppnull else '')
           for idx, size in pptr]
        + [f'vtable slot +0x{a:x} ({n} args) stubbed' if k == 'slot'
           else f'the import cell 0x{a:08x} holds a stub ({n} args)' if k == 'iat'
           else f'callee 0x{a:08x} ({n} args) run, not hooked' if k == 'run'
           else f'callee 0x{a:08x} ({n} args) stubbed' for k, a, n in pstubs]
        + [f'{_placed(w, o)} points at {n} random bytes less `{e}` (placed'
           + (', null a third of the rows)' if k in pnull else ')') for k, (w, o, e, n) in enumerate(pplaces)]
        + [f'slot +0x{o:x} laid as 0x{a:08x} or its stub, a flag after the answers' for o, a in pslot]
        + [f'stub {idx} answered from {vals} (90%)' for idx, vals in pans]
        + [f'stub {idx} answers the address of {n} random bytes, a `p` window after the places'
           for idx, n in pansw if [i for i, _ in pansw].count(idx) == 1]
        + [f'stub {idx} answers a window of its own to each call, in call order ({n} bytes, ...)'
           for k, (idx, n) in enumerate(pansw) if [i for i, _ in pansw].count(idx) > 1
           and idx not in [i for i, _ in pansw[:k]]]
        + ([f'the stubbed vtable laid at `{vtat}`'] if pvtat else [])
        + [f'the stubbed vtable laid at `{vh}` half the rows' for vh in vtat_half]
        + [f'the word at `{al.partition(":")[0]}` laid as the address of `{al.partition(":")[2]}` half the rows'
           for al in alias]
        + (['`__cdecl`: the caller pops the arguments'] if cdecl else [])
        + [f'argument {idx} copies `{w}+{o:#x}` (45%)' for idx, w, o in peq]
        + [f'stub {idx} answers {n} values in call order' + (f', the last from {t}' if t else '')
           + ', printed after the answers' for idx, n, t in pseq]
        + [f'stub {idx} answers {lo}..{hi} every row' for idx, lo, hi in pansr]
        + [f'stub {idx} records its `ecx`' + (', `edx`' if record_edx else '') + f' and arguments, a window\'s pointer by its name, after ` # `'
           + (f', split by the rows of sparse table {prowsplit[idx]} (a token a row, `ecx` `r+OFF`, '
              f'then the calls outside it)' if idx in prowsplit else '')
           for idx in precord]
        + (['the recorded calls one token a call, in call order, each headed `sIDX:` (the callee)']
           if record_order else [])
        + [(f'stub {idx} prints the {n} bytes at the cell 0x{k[1]:08x} when it is called, hex, after the record (`/` a call)'
            if isinstance(k, tuple) else
            f'stub {idx} prints the {n} bytes its argument {k} points at'
            + (f' (bytes {", ".join(map(str, wr))} written, the rest 00)' if wr is not None else '')
            + ', hex, after the record (`/` a call)')
           for idx, k, n, wr in pderef]
        + [f'stub {idx} writes a value in '
           + (f'{v[1]}..{v[2]}' if v[0] == 'range' else str(v[1]))
           + f' through its argument {k}, printed after the answers (`o` in a place)' for idx, k, v in pout]
        + [f'stub {k} is `__cdecl`: the caller pops its arguments' for k in pcdecl]
        + [f'stub {idx} lowers `{w}+{o:#x}` ({z} bytes) by one each call' if v == 'dec'
           else f'stub {idx} writes {v} ({z} bytes) at `{w}+{o:#x}` each call' for idx, w, o, z, v in pwrite]
        + [f'stub {idx} answers from its own `ecx`: the word at `+{o:#x}`'
           + (f', or `ecx+{o:#x}` when the word at `+{f:#x}` is set' if f is not None else '')
           for idx, o, f in pecxans]
        + [f'a circular list in `c{lo}..c{hi}` at {b:#x}+{st:#x}k, next `+{n:#x}`, prev `+{pv:#x}`, head the word at `{w}+{ho:#x}`'
           for w, ho, lo, hi, n, pv, b, st in pring]
        + [f'the global cell 0x{va:08x} holds `this`' for va in pthis]
        + [f'{n} rows {st:#x} apart at `{"@" if w == "@" else (w + "+" if w else "")}{lo}`, slices '
           + ', '.join(f'+{o:#x}:{z:#x}' for o, z in sl) + ' (`p` windows after the pointed buffers, before the places)'
           for w, lo, st, n, sl in psparse]
        + [f'a counted array behind image words: the count at 0x{c:08x} in {lo}..{hi}, the array\'s pointer at 0x{p:08x}, '
           f'{z:#x} random bytes an element (`c` windows head and array, `p` windows the elements after the pointer arguments\')'
           for c, lo, hi, p, z in pgtab]
        + [f'figures p{lo}..p{hi} link at +{o:#x} ({z} bytes) '
           + ('below' if d == '<' else 'above') + ' their own index or to ' + ', '.join(map(str, t))
           for lo, hi, o, z, d, t in plinks]
        + [f'the stubbed vtable laid at `{w}+{o:#x}` as well as at `this`'.replace('+0x0`', '+0`')
           for w, o in palso]
        + ([f'`ecx` an integer in {pecx[1]}..{pecx[2]}, not the object, the row\'s last'
            if pecx[0] == 'range' else f'`ecx` an integer in {pecx[1]}, not the object, the row\'s last']
           if pecx else [])
        + ([f'`edx` an integer in {pedx[1]}..{pedx[2]}, after `ecx` on the row'
            if pedx[0] == 'range' else f'`edx` an integer in {pedx[1]}, after `ecx` on the row']
           if pedx else [])
        + ([f'`edx` the address of window `{edx_win}`'] if edx_win else [])
        + (['`ecx` and `edx` held to their draw every row, no wild word'] if tame else [])) or 'no windows'
    script = OBJ_WIN_SCRIPT.format(snake=sn, name=name, va=va, item=item, win=win, own=own, bias=parsed,
                                   biasdoc=biasdoc, windoc=windoc, argset=pset, argfrom=pfrom,
                                   cells=pcells, at=pat, ptrargs=pptr, pnulls=ppnull, stubs=pstubs, places=pplaces, nulls=pnull,
                                   wbias=pwbias, fills=pfills, argrange=prange, slotbase=pslot, ecxint=pecx, edx=pedx, edxwin=edx_win, wild=0.0 if tame else 0.1,
                                   answers=pans, vtat=pvtat, answerwin=pansw, cdecl=cdecl, argeq=peq,
                                   seq=pseq, ansrange=pansr, cellthis=pthis, vtalso=palso, vtathalf=phalf, alias=palias,
                                   sparse=psparse, gtab=pgtab, record=precord, recordorder=record_order, recordedx=record_edx, rowsplit=prowsplit, links=plinks,
                                   outarg=pout, cdeclstubs=pcdecl, stubwrite=pwrite, deref=pderef, ecxanswer=pecxans, ring=pring, rows=rows,
                                   ret='xmm0' if ret == 'xmm0' else 'eax')
    doc = doc or (f'{win} random bytes of object, {own} argument(s), {windoc}; `eax` and every byte of '
                  f'every window after held to the port (`c` the cells and tables, `p` the pointed buffers, '
                  f'the stubs\' answers after the arguments).')
    doc += rowdoc
    if precord or pderef:
        logsplit = ('        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");\n'
                    '        let want_log: Vec<&str> = calls.split_whitespace().collect();\n')
        logdecl = '        let mut log: Vec<String> = Vec::new();\n'
        logcheck = (f'        assert_eq!(log, want_log, "{short} — the recorded calls, row `{{line}}`");\n')
    else:
        logsplit = logdecl = logcheck = ''
    test = OBJ_WIN_TEST.format(name=name, va=va, item=item, snake=sn, short=short, doc=doc, port=port,
                               ret=retcode, nc=len(pcells) + len(pat) + 2 * len(pgtab), pbase=1 + len(pcells) + len(pat) + 2 * len(pgtab),
                               logsplit=logsplit, logdecl=logdecl, logcheck=logcheck, floor=floor)
    return EMU / f'sweep_{sn}.py', script, SWEEP / f'{sn}.rs', test, f'mod {sn};'


def generate_void(spec, *, item, own=0, win=0x40):
    """An empty body (parked 1889): `obj` with no answer and every byte held."""
    return generate_obj(spec, item=item, win=win, bias=[], own=own, ret='none', port='0',
                        doc=f'an empty body: {win} random bytes of object and {own} argument(s), every byte held '
                            f'after the return and `esp` checked (parked 1889).')


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
        tail = VCALL_TAIL.format(slot=slot, nargs=nargs, tail=tail, own=own)
    else:
        tail = SCRIPT_TAIL.format(write=write, call=call.format(ecx=ecx))
    if shape == 'vcall' and via is not None:
        rows_src = ''
    elif shape == 'vcall' and own:
        rows_src = VCALL_OWN_ROWS.format(item=item, own=own)
    else:
        rows_src = ROWS[shape].format(item=item)
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


KEPT = '# sweep_gen: '
NOT_KEPT = ('--write-mod', '--regen', '--dry-run')


def kept_line(path):
    """The `sweep_gen.py` line a generated script carries (item 2325, parked
    2311), as an argument list, or None when the script keeps none."""
    for line in reversed(Path(path).read_text().splitlines()):
        if line.startswith(KEPT):
            return json.loads(line[len(KEPT):])
    return None


def _kept_for_regen(argv):
    """`--regen SCRIPT` (or its snake name): the line the script keeps, to
    be redrawn; None when `argv` is a whole line."""
    rest = [a for a in argv if a != '--regen']
    if '--regen' not in argv or len(rest) != 1 or rest[0].startswith('-'):
        return None
    path = Path(rest[0])
    if not path.exists():
        path = EMU / f'sweep_{rest[0]}.py'
    if not path.exists():
        raise ValueError(f'--regen {rest[0]}: no such script, nor tools/emu/sweep_{rest[0]}.py')
    line = kept_line(path)
    if line is None:
        raise ValueError(f'{shown(path)} keeps no `{KEPT.strip()}` line (drawn before item 2325); '
                         'give --regen the whole line')
    return line


def merged_with_kept(argv):
    """`--regen` given a whole line (Loop 2508): the kept line's options the
    new line does not set are kept under it — a window, a bias, a range —
    so a redraw that names one new flag does not drop the rest. The sweep
    is found by its function, the token the kept line carries verbatim."""
    if '--regen' not in argv:
        return argv
    fn = next((a for a in argv if '::' in a and '@' in a), None)
    kept = None
    # `--name M` (a function swept as several modules, `valid_filter`'s arms; item 2801): that module's script.
    named = argv[argv.index('--name') + 1] if '--name' in argv[:-1] else None
    for p in ([EMU / f'sweep_{named}.py'] if named else sorted(EMU.glob('sweep_*.py'))) if fn else ():
        if not p.exists():
            continue
        line = kept_line(p)
        if line and fn in line:
            kept = line
            break
    if kept is None:
        return argv
    new_opts = {a for a in argv if a.startswith('--')}
    extra, i = [], 0
    while i < len(kept):
        a = kept[i]
        if not a.startswith('--'):
            i += 1
            continue
        group = [a]
        i += 1
        while i < len(kept) and not kept[i].startswith('--'):
            group.append(kept[i])
            i += 1
        if a not in new_opts and a not in NOT_KEPT:
            extra += group
    return argv + extra


def kept_port(old, new, todo):
    """`--regen` with no `--port` (item 2368, parked 2342): the new test
    `new` with its `todo` port replaced by the block the old test `old`
    holds in the same place — the hand-written port between the generated
    halves — or None when `old` has no such place. The place is found by
    the line before the port and the line after it, which the template
    fixes whatever the draw."""
    at = new.find(todo)
    if at < 0:
        return None
    start = new.rfind('\n', 0, new.rfind('\n', 0, at)) + 1
    head = new[start:at]
    nl = new.find('\n', at + len(todo) + 1)
    tail = new[at + len(todo):nl if nl >= 0 else len(new)]
    i = old.find(head)
    if i < 0:
        return None
    i += len(head)
    j = old.find(tail, i)
    if j < 0:
        return None
    return new[:at] + old[i:j] + new[at + len(todo):]


SWEEP_FAMILIES = ('a_f', 'g_o', 'p_t', 'u_z')
REGISTRY_MARKER = '// Registered standalone sweeps.\n'
REGISTRATION = re.compile(r'#\[path = "([a-z][a-z0-9_]*)\.rs"\]\nmod \1;\n')


def sweep_family(name):
    return 'a_f' if name[0] <= 'f' else 'g_o' if name[0] <= 'o' else 'p_t' if name[0] <= 't' else 'u_z'


def standalone_names(text):
    if text.count(REGISTRY_MARKER) != 1:
        raise ValueError('standalone registry needs one registration marker')
    body = text.split(REGISTRY_MARKER)[1]
    names = REGISTRATION.findall(body)
    if REGISTRATION.sub('', body).strip() or len(set(names)) != len(names):
        raise ValueError('unexpected or duplicate standalone registration; refusing to discard content')
    return names


def registration(line):
    match = re.fullmatch(r'mod ([a-z][a-z0-9_]*);', line)
    if not match:
        raise ValueError(f'invalid module registration: {line!r}')
    name = match[1]
    return f'#[path = "{name}.rs"]\n{line}\n'


def insert_standalone(text, line):
    names = standalone_names(text)
    registration(line)  # Validate before extracting its name.
    name = line[4:-1]
    if name in names:
        raise ValueError(f'duplicate standalone module: {name}')
    header = text.split(REGISTRY_MARKER)[0] + REGISTRY_MARKER
    return header + ''.join(registration(f'mod {n};') for n in sorted([*names, name]))


def sweep_registry(line, test):
    """Keep existing ownership; route new independent tests to a small target.

    Conservative textual classification: even crate references in comments keep
    a new test in the library. Compilation remains the authority on privacy.
    """
    registration(line)
    main = SWEEP / 'mod.rs'
    name = line[4:-1]
    owners = [main] if main.exists() and line in main.read_text().splitlines() else []
    for family in SWEEP_FAMILIES:
        path = SWEEP / f'standalone_{family}.rs'
        if path.exists() and name in standalone_names(path.read_text()):
            owners.append(path)
    if len(owners) > 1:
        raise ValueError(f'module registered in multiple targets: {name}')
    if owners:
        return owners[0]
    candidate = SWEEP / f'standalone_{sweep_family(name)}.rs'
    return candidate if candidate.exists() and 'crate::' not in test else main


def validate_sweep_registrations(crate):
    """Every validly named sweep source has exactly one live Cargo registration."""
    from collections import Counter
    import tomllib
    crate = Path(crate)
    folder = crate / 'src/sweep'
    manifest = crate / 'Cargo.toml'
    if manifest.exists():
        config = tomllib.loads(manifest.read_text())
        if config.get('package', {}).get('autotests') is False:
            raise ValueError('automatic sweep test targets are disabled')
        if any(t.get('name') in {f'sweep_{f}' for f in SWEEP_FAMILIES} for t in config.get('test', [])):
            raise ValueError('sweep partitions require unmodified automatic Cargo test targets')
    main = re.findall(r'^mod ([a-z][a-z0-9_]*);$', (folder / 'mod.rs').read_text(), re.M)
    names = [name for name in main if name != 'support']
    for family in SWEEP_FAMILIES:
        path = folder / f'standalone_{family}.rs'
        selected = standalone_names(path.read_text())
        for name in selected:
            if sweep_family(name) != family:
                raise ValueError(f'{name} is in the wrong partition: {family}')
        entry = crate / 'tests' / f'sweep_{family}.rs'
        mount = f'#[path = "../src/sweep/standalone_{family}.rs"]\nmod sweep;'
        if mount not in entry.read_text():
            raise ValueError(f'partition is not mounted by {entry}')
        if any('crate::' in (folder / f'{name}.rs').read_text() for name in selected):
            if 'pub use sim::*;' not in entry.read_text().splitlines():
                raise ValueError(f'public API bridge missing from {entry}')
        names += selected
    excluded = {'mod', 'support', *(f'standalone_{f}' for f in SWEEP_FAMILIES)}
    sources = {p.stem for p in folder.glob('*.rs')
               if re.fullmatch('[a-z][a-z0-9_]*', p.stem) and p.stem not in excluded}
    counts = Counter(names)
    if set(counts) != sources or any(n != 1 for n in counts.values()):
        raise ValueError(f'sweep registration mismatch: missing={sorted(sources-set(counts))}, '
                         f'unknown={sorted(set(counts)-sources)}, '
                         f'duplicates={sorted(k for k,v in counts.items() if v != 1)}')


def insert_mod(text, line):
    """`text` with `line` (`mod name;`) placed in its last run of `mod` lines
    at the name's sorted place, or appended where the file has none (item
    2379, Loop 2003: two batches sorted the appended tail by hand)."""
    lines = text.splitlines()
    run = [i for i, ln in enumerate(lines) if re.fullmatch(r'mod \w+;', ln)]
    if not run:
        return text + ('' if text.endswith('\n') or not text else '\n') + line + '\n'
    start = end = run[-1]
    while start - 1 in run:
        start -= 1
    at = next((i for i in range(start, end + 1) if lines[i] > line), end + 1)
    lines.insert(at, line)
    return '\n'.join(lines) + '\n'


def fwd_gate_expand(spec, own):
    """`--fwd-gate OFF:MASK:VA[:T,T..]` (item 2443, parked 2433): a forwarder
    that tests `gamec`'s byte at OFF against MASK (set: it does nothing), then
    calls VA with the caller's stale `ecx` first, its `own` arguments in order
    and the constant tail T.. last. Returns the `obj` flags it stands for."""
    parts = spec.split(':')
    if len(parts) not in (3, 4):
        raise ValueError(f'--fwd-gate {spec}: want OFF:MASK:VA[:T,T..]')
    off, mask, va = int(parts[0], 0), int(parts[1], 0), int(parts[2], 0)
    tail = [t for t in parts[3].split(',') if t] if len(parts) == 4 else []
    vals = sorted({0, mask, mask * 2, mask * 3, 0xff & ~mask})
    args = [f'a[{i}]' for i in range(own)]
    fmt = ','.join(['m+0x0', 'm+0x0'] + ['{}'] * own + tail)
    call = f'log.push(format!("{fmt}"' + ''.join(f', {a}' for a in args) + '));'
    port = (f'{{ if c[0][{off:#x}] & {mask:#x} != 0 {{ log.push("-".into()); }} else {{ {call} }} 0 }}')
    return {'stub': [f'{va:#x}:{own + len(tail) + 1}'], 'cells': ['0xc061e8:0x900'],
            'wbias': [f'c0+{off:#x}:' + ','.join(f'{v:#x}' if v else '0' for v in vals)],
            'record': ['0'], 'port': port}


def cmd_gate_expand(spec, own):
    """`--cmd-gate OFF:MASK:CMD:SIZE:FIELD,..[:C[:PRE[:SOUND]]]` (item 2454,
    parked 2444): `issue_halt`'s shape -- `gamec`'s byte at OFF tested against
    MASK (set: nothing runs), `check_accept_issue` and `add_group` (two stubs
    answered 0/1), the command built in a local of SIZE bytes (CMD first, then
    each FIELD: `aN` the argument as a word, `bN` its low byte, `kV` the
    constant word V), `add_command` handed it. The `sp-` token is
    `C + 8 + 4*own`, C the local's `ebp` depth (default SIZE rounded up to
    four); PRE, `aN>=0,..`, are arguments tested before the check (a
    `siege_attack`); SOUND, `VA:ARG`, is a `SoundGlobal::play` call (one
    argument) made after the gate and before the check. An empty OFF is a
    `CommandManager` issuer: no gate, argument 0 the group (a buffer), handed
    to `add_group`. Returns the `obj` flags it stands for."""
    parts = spec.split(':')
    if not 5 <= len(parts) <= 9:
        raise ValueError(f'--cmd-gate {spec}: want OFF:MASK:CMD:SIZE:FIELD,..[:C[:PRE[:VA:ARG]]]')
    gated = parts[0] != ''
    off, mask = (int(parts[0], 0), int(parts[1], 0)) if gated else (0, 0)
    cmd, size = int(parts[2], 0), int(parts[3], 0)
    fields = [f for f in parts[4].split(',') if f]
    depth = int(parts[5], 0) if len(parts) > 5 and parts[5] else (size + 3) // 4 * 4
    pre = [g for g in parts[6].split(',') if g] if len(parts) > 6 else []
    sound = (int(parts[7], 0), int(parts[8], 0)) if len(parts) > 8 else None
    fmt, vals = f'{cmd:02x}', []
    for f in fields:
        if f[0] == 'a' and f[1:].isdigit() and int(f[1:]) < own:
            fmt += '{}'
            vals.append(f'le(a[{f[1:]}])')
        elif f[0] == 'b' and f[1:].isdigit() and int(f[1:]) < own:
            fmt += '{:02x}'
            vals.append(f'a[{f[1:]}] as u8')
        elif f[0] == 'k':
            fmt += '{}'
            vals.append('"%08x"' % int.from_bytes(int(f[1:], 0).to_bytes(4, 'little'), 'big'))
        else:
            raise ValueError(f'--cmd-gate {spec}: field {f!r} is aN, bN (N < --own) or kV')
    nst = 3 + (sound is not None)
    pushes = lambda n: ' '.join(['log.push("-".into());'] * n)
    cmp_ = []
    for g in pre:
        m = re.fullmatch(r'a(\d+)>=0', g)
        if not m or int(m.group(1)) >= own:
            raise ValueError(f'--cmd-gate {spec}: guard {g!r} is aN>=0')
        cmp_.append(f'a[{m.group(1)}] < 0')
    sp = f'sp-{depth + 8 + 4 * own:#x}'
    fmtcall = f'format!("{fmt}"' + ''.join(f', {v}' for v in vals) + ')'
    grp = '"15269768,m+0x0"' if gated else '"15269768,p0+0x0"'
    # The emulator prints its record in stub order and the deref last, so the
    # sound stub (index 3) stands before the arm's final push.
    snd = f'log.push("m+0x0,{sound[1]}".into()); ' if sound else ''
    dash = lambda n: ' '.join(['log.push("-".into());'] * n)
    arm3 = f'{dash(2)} {snd}{dash(1)}'
    arm2 = f'{dash(1)} {snd}{dash(1)}'
    body = (f'log.push("15269728".into()); if a[{own}] == 0 {{ {arm3} }} else {{ '
            f'log.push({grp}.into()); if a[{own + 1}] == 0 {{ {arm2} }} else {{ '
            f'log.push("15269768,{sp},{size}".into()); {snd}log.push({fmtcall}); }} }}')
    if pre:
        body = f'if {" || ".join(cmp_)} {{ {pushes(4)} }} else {{ {body} }}'
    le = ('let le = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>(); '
          if any(v.startswith('le(') for v in vals) else '')
    x = {'stub': ['0x940a70:0', '0x94bb60:1', '0x94bae0:2'], 'answers': ['0:0,1', '1:0,1'],
         'record': ['0', '1', '2'], 'deref': [f'2:0:{size}'], 'cells': [], 'wbias': [], 'ptrarg': []}
    if sound:
        x['stub'].append(f'{sound[0]:#x}:1')
        x['record'].append('3')
    if gated:
        vs = sorted({0, mask, mask * 2, mask * 3, 0xff & ~mask})
        x['cells'] = ['0xc061e8:0x900']
        x['wbias'] = [f'c0+{off:#x}:' + ','.join(f'{v:#x}' if v else '0' for v in vs)]
        port = f'{{ {le}if c[0][{off:#x}] & {mask:#x} != 0 {{ {pushes(nst + 1)} }} else {{ {body} }} 0 }}'
    else:
        x['ptrarg'] = ['0:0x20']
        port = f'{{ {le}{body} 0 }}'
    x['port'] = port
    return x


def cmd_obj_expand(spec, own):
    """`--cmd-obj CMD:SIZE:FIELD,..[:C]` (item 2469, parked 2466):
    `issue_come_out` / `issue_unqueue`'s shape -- argument 0 an object read
    through (`--ptrarg 0:0x40`), `check_accept_issue` answered 0/1, the
    command built in a local of SIZE bytes and handed to `add_command` (no
    group). FIELDs, in the order they lie: `zOFF` the object's byte widened
    to a dword, `sOFF` its signed word widened to a dword, `wOFF` its word's
    two bytes as they lie, `aN` an argument (N >= 1) as a dword, `kV` a
    constant dword. The `sp-` token is `C + 8 + 4*own`, C defaulting to SIZE
    rounded up to four. Returns the `obj` flags it stands for."""
    parts = spec.split(':')
    if not 3 <= len(parts) <= 4:
        raise ValueError(f'--cmd-obj {spec}: want CMD:SIZE:FIELD,..[:C]')
    cmd, size = int(parts[0], 0), int(parts[1], 0)
    depth = int(parts[3], 0) if len(parts) > 3 and parts[3] else (size + 3) // 4 * 4
    fmt, vals, pre = f'{cmd:02x}', [], ''
    for f in [f for f in parts[2].split(',') if f]:
        if f[0] == 'z' and f[1:]:
            off = int(f[1:], 0)
            fmt += '{:02x}000000'
            vals.append(f'p[0][{off:#x}]')
        elif f[0] == 's' and f[1:]:
            off = int(f[1:], 0)
            pre += (f'let w{off:#x} = i32::from(i16::from_le_bytes([p[0][{off:#x}], p[0][{off + 1:#x}]])); ')
            fmt += '{}'
            vals.append(f'le(w{off:#x})')
        elif f[0] == 'w' and f[1:]:
            off = int(f[1:], 0)
            fmt += '{:02x}{:02x}'
            vals += [f'p[0][{off:#x}]', f'p[0][{off + 1:#x}]']
        elif f[0] == 'a' and f[1:].isdigit() and 1 <= int(f[1:]) < own:
            fmt += '{}'
            vals.append(f'le(a[{f[1:]}])')
        elif f[0] == 'k' and f[1:]:
            fmt += '{}'
            vals.append('"%08x"' % int.from_bytes(int(f[1:], 0).to_bytes(4, 'little'), 'big'))
        else:
            raise ValueError(f'--cmd-obj {spec}: field {f!r} is zOFF, sOFF, wOFF, aN (1 <= N < --own) or kV')
    sp = f'sp-{depth + 8 + 4 * own:#x}'
    le = ('let le = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>(); ')
    call = f'format!("{fmt}"' + ''.join(f', {v}' for v in vals) + ')'
    port = (f'{{ {le}{pre}log.push("15269728".into()); if a[{own}] == 0 {{ log.push("-".into()); '
            f'log.push("-".into()); }} else {{ log.push("15269768,{sp},{size}".into()); log.push({call}); }} 0 }}')
    return {'stub': ['0x940a70:0', '0x94bae0:2'], 'answers': ['0:0,1'], 'record': ['0', '1'],
            'deref': [f'1:0:{size}'], 'ptrarg': ['0:0x40'], 'port': port}


def cmd_byte_expand(spec, own):
    """`--cmd-byte CMD[:SIZE]` (item 2481, parked 2472): `issue_recall`'s
    shape -- argument 0 a group id (a plain integer printed by name),
    `check_accept_issue` and `add_group` answered 0/1, a one-byte (SIZE) local
    holding CMD handed to `add_command`. The `sp-` token is
    `SIZE + 8 + 4*own`. Returns the `obj` flags it stands for."""
    parts = spec.split(':')
    if not parts[0] or len(parts) > 2:
        raise ValueError(f'--cmd-byte {spec}: want CMD[:SIZE]')
    cmd = int(parts[0], 0)
    size = int(parts[1], 0) if len(parts) > 1 else 1
    sp = f'sp-{size + 8 + 4 * own:#x}'
    dash = ' '.join(['log.push("-".into());'] * 3)
    two = ' '.join(['log.push("-".into());'] * 2)
    port = (f'{{ log.push("15269728".into()); if a[{own}] == 0 {{ {dash} }} else {{ '
            f'log.push(format!("15269768,{{}}", a[0])); if a[{own + 1}] == 0 {{ {two} }} else {{ '
            f'log.push("15269768,{sp},{size}".into()); log.push("{cmd:02x}".into()); }} }} 0 }}')
    return {'stub': ['0x940a70:0', '0x94bb60:1', '0x94bae0:2'], 'answers': ['0:0,1', '1:0,1'],
            'record': ['0', '1', '2'], 'deref': [f'2:0:{size}'], 'port': port}


def cmd_seh_expand(spec, own):
    """`--cmd-seh CMD:SIZE:FIELD,..:C:IGN:WIDE:NARROW:DW:DN` (item 2486,
    parked 2482): `issue_treaty`'s shape -- argument 0 a `LeaderOut` whose
    word at +8 names a leader, argument 1 the other; `check_accept_issue`
    answered 0/1; the command built in a local (CMD, then FIELDs: `w` the
    leader at +8, `aN` an argument as a dword) and handed to `add_command`
    when both leaders' flag bytes (the image's table, 0x6eec a row) are set;
    else, the byte at IGN clear, an SEH-framed feedback arm: a wide and a
    narrow `String` (constants WIDE, NARROW, locals DW and DN deep), an
    `Error::report` answered 0 (its two stale stack words print 0, as the
    emulator's zeroed stack leaves them -- a hand-held pair) and both strings closed. C is the command local's
    `ebp` depth. Returns the `obj` flags it stands for."""
    parts = spec.split(':')
    if len(parts) != 9:
        raise ValueError(f'--cmd-seh {spec}: want CMD:SIZE:FIELD,..:C:IGN:WIDE:NARROW:DW:DN')
    cmd, size = int(parts[0], 0), int(parts[1], 0)
    depth, ign, wide, narrow = int(parts[3], 0), int(parts[4], 0), int(parts[5], 0), int(parts[6], 0)
    dw, dn = int(parts[7], 0), int(parts[8], 0)
    fmt, vals = f'{cmd:02x}', []
    for f in [f for f in parts[2].split(',') if f]:
        if f == 'w':
            fmt += '{}'
            vals.append('le(who)')
        elif f[0] == 'a' and f[1:].isdigit() and 1 <= int(f[1:]) < own:
            fmt += '{}'
            vals.append(f'le(a[{f[1:]}])')
        else:
            raise ValueError(f'--cmd-seh {spec}: field {f!r} is w or aN (1 <= N < --own)')
    sp = lambda d: f'sp-{d + 8 + 4 * own:#x}'
    dash = lambda n: ' '.join(['log.push("-".into());'] * n)
    call = f'format!("{fmt}"' + ''.join(f', {v}' for v in vals) + ')'
    le = 'let le = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>(); '
    who = 'let who = i32::from_le_bytes([p[0][8], p[0][9], p[0][10], p[0][11]]); '
    arm = (f'{dash(1)} log.push("{sp(dw)},{wide}".into()); log.push("{sp(dn)},{narrow}".into()); '
           f'log.push("13161384,{sp(dn)},{sp(dn)},0,0,c0+{ign - 0xee12cc:#x},1".into()); '
           f'log.push("{sp(dn)}/{sp(dw)}".into()); {dash(1)}')
    body = (f'log.push("15269728".into()); if a[{own}] == 0 {{ {dash(6)} }} else if p[1 + who as usize][0] & 1 != 0 '
            f'&& p[1 + a[1] as usize][0] & 1 != 0 {{ log.push("15269768,{sp(depth)},{size}".into()); {dash(4)} '
            f'log.push({call}); }} else if c[0][{ign - 0xee12cc:#x}] != 0 {{ {dash(6)} }} else {{ {arm} }}')
    port = f'{{ {le}{who}{body} 0 }}'
    return {'stub': ['0x940a70:0', '0x94bae0:2', '0xa1edd0:1', '0xa1d660:1', '0xa2e550:6', '0xa1cf40:0'],
            'answers': ['0:0,1'], 'answer_range': ['4:0:0'], 'record': ['0', '1', '2', '3', '4', '5'],
            'deref': [f'1:0:{size}'], 'ptrarg': ['0:0x10'], 'wbias': ['p0+8:0..3', 'c0+0:0,0,0,0x01010101',
            'c0+4:0,0,0,0x01010101', 'c0+8:0,0,0,0x01010101']
            + [f'p{k}+0:1,1,1,3,0' for k in range(1, 5)], 'at': ['0xee12cc:12'],
            'sparse': ['@0xe3a390:0x6eec:4:0,4'], 'argrange': ['1:0:3'], 'port': port}


def chain_expand(spec, own, cell_index, pbase):
    """`--chain CELL:OFF:STRIDE:P:I:N:SIZE` (item 2495, parked 2484):
    `PathFinder::find_wpath`'s three-level walk -- the global cell CELL
    points at a record whose word at OFF + STRIDE * a[P] points at a table
    of N words, whose word at 4 * a[I] points at a SIZE-byte record. The
    cell's window (`c<cell_index>`) is laid wide enough for N rows, the table
    and the record are two places (`p<pbase>`, then the next), and both
    arguments are drawn inside 0..N-1. Returns the flags it stands for."""
    parts = spec.split(':')
    if len(parts) != 7:
        raise ValueError(f'--chain {spec}: want CELL:OFF:STRIDE:P:I:N:SIZE')
    cell, off, stride = int(parts[0], 0), int(parts[1], 0), int(parts[2], 0)
    pi, ii, n, size = int(parts[3], 0), int(parts[4], 0), int(parts[5], 0), int(parts[6], 0)
    if not (0 <= pi < own and 0 <= ii < own and pi != ii) or n < 1:
        raise ValueError(f'--chain {spec}: P and I name two of the --own arguments, N >= 1')
    wide = (off + stride * n + 4 + 0xf) & ~0xf
    return {'cells': [f'{cell:#x}:{wide:#x}'],
            'place': [f'c{cell_index}+{off:#x}+{stride:#x}*a[{pi}]:0:{4 * n}',
                      f'p{pbase}+4*a[{ii}]:0:{size:#x}'],
            'argrange': [f'{pi}:0:{n - 1}', f'{ii}:0:{n - 1}']}


def table_expand(spec, pbase, row_byte=None, row_links=()):
    """`--table VA:STRIDE:ROWS:COLS:SIZE:XOFF:VOFF` (item 2621, parked 2617):
    `WorldData::get_bottom`'s walk -- the image's own table at VA, ROWS rows
    STRIDE apart, each row's first word pointing at an array of COLS record
    pointers, each a SIZE-byte record whose i16 at XOFF is the next column
    (drawn below its own, or -1/-2, so the walk ends) and whose i16 at VOFF is
    the next row (drawn inside 0..ROWS-1; `-` leaves it alone). The row's array is place `p<pbase +
    r*(1+COLS)>` and its records the COLS after it. `row_byte` (`--row-byte OFF[:LOW,..]`, item 2719, parked
    2713) is a record's own row in the byte at OFF: the word holding it drawn as row<<shift | LOW, LOW
    default 0; each `row_links` entry (`--row-link OFF:SIZE:DIR[:T,..]`) is a `--link` over each row's
    records. Returns the flags it stands for."""
    parts = spec.split(':')
    if len(parts) != 7:
        raise ValueError(f'--table {spec}: want VA:STRIDE:ROWS:COLS:SIZE:XOFF:VOFF')
    va, stride, rows, cols, size, xoff = (int(x, 0) for x in parts[:6])
    voff = None if parts[6] == '-' else int(parts[6], 0)
    if rows < 1 or cols < 1 or max(xoff, voff or 0) + 4 > size:
        raise ValueError(f'--table {spec}: ROWS, COLS >= 1 and SIZE past both fields')
    rb = None
    if row_byte is not None:
        off, _, lows = row_byte.partition(':')
        rb = (int(off, 0), [int(v, 0) for v in lows.split(',')] if lows else [0])
        if rb[0] >= size or any(not 0 <= v < 1 << 8 * (rb[0] & 3) for v in rb[1]) or rows > 256:
            raise ValueError(f'--row-byte {row_byte}: OFF inside SIZE, LOW below the byte, ROWS <= 256')
    rl = []
    for r_ in row_links:
        o, sz, d, *t = r_.split(':')
        if sz not in ('1', '2', '4') or d not in ('<', '>') or len(t) > 1 or int(o, 0) + int(sz) > size:
            raise ValueError(f'--row-link is OFF:1|2|4:<|>[:T,T..] inside SIZE, not {r_!r}')
        rl.append((int(o, 0), sz, d, t))
    place, link, wbias = [], [], []
    for r in range(rows):
        t = pbase + r * (1 + cols)
        run = f'p{t + 1}..{t + cols}'
        place.append(f'{parts[0]}+{parts[1]}*{r}:0:{4 * cols}')
        place += [f'p{t}+{4 * c}:0:{parts[4]}' for c in range(cols)]
        link.append(f'{run}+{xoff:#x}:2:<:-1,-2')
        for o, sz, d, tm in rl:
            link.append(f'{run}+{o:#x}:{sz}:{d}' + (f':{tm[0]}' if tm else ''))
        if voff is not None:
            wbias.append(f'{run}+{voff:#x}:0..{rows - 1}')
        if rb:
            sh = 8 * (rb[0] & 3)
            wbias.append(f'{run}+{rb[0] & ~3:#x}:' + ','.join(f'{(r << sh) | v:#x}' for v in rb[1]))
    return {'place': place, 'link': link, 'wbias': wbias}


def ptrtab_expand(spec, pbase):
    """`--ptrtab LOC:N:SIZE[:vt]` (item 2782, parked 2772): an array of N record
    pointers at LOC (a place of 4N bytes, window `p<pbase>`) and each record a
    SIZE-byte place of that window, windows `p<pbase+1>`.. in order; `vt` also
    lays the stubbed vtable at each record's offset 0 (`--vtat-also`). Returns
    the flags it stands for."""
    parts = spec.split(':')
    if len(parts) not in (3, 4) or (len(parts) == 4 and parts[3] != 'vt'):
        raise ValueError(f'--ptrtab {spec}: want LOC:N:SIZE[:vt]')
    n, size = int(parts[1], 0), int(parts[2], 0)
    if n < 1 or size < 1:
        raise ValueError(f'--ptrtab {spec}: N and SIZE >= 1')
    place = [f'{parts[0]}:0:{4 * n}'] + [f'p{pbase}+{4 * k}:0:{parts[2]}' for k in range(n)]
    vt = [f'p{pbase + 1 + k}+0' for k in range(n)] if len(parts) == 4 else []
    return {'place': place, 'vtat_also': vt}


def jump_expand(spec, own):
    """`--jump IDX:N` (item 2615, parked 2607): a function whose body is one
    indexed jump over N arms (`jmp *TABLE(,%eax,4)`, `Setup::get_starting_citizens`)
    -- stack argument IDX is the index and is drawn in 0..N-1, since a wild one
    jumps into whatever bytes follow the table. Returns the flags it stands for."""
    parts = spec.split(':')
    if len(parts) != 2:
        raise ValueError(f'--jump {spec}: want IDX:N')
    idx, n = int(parts[0], 0), int(parts[1], 0)
    if not 0 <= idx < own or n < 1:
        raise ValueError(f'--jump {spec}: IDX names one of the --own arguments, N >= 1')
    return {'argrange': [f'{idx}:0:{n - 1}']}


def split_gate_this(argv):
    """A bare `--gate-this` (its optional value is OFF:MASK) becomes `--gate-bare`, so it
    never swallows the function that follows it (parked 2569)."""
    out = []
    for i, a in enumerate(argv):
        nxt = argv[i + 1] if i + 1 < len(argv) else None
        if a == '--gate-this' and (nxt is None or not re.fullmatch(r'0[xX][0-9a-fA-F]+:\w+|\d+:\w+', nxt)):
            out.append('--gate-bare')
        else:
            out.append(a)
    return out


def main(argv=None):
    argv = sys.argv[1:] if argv is None else list(argv)
    try:
        line = _kept_for_regen(argv)
    except ValueError as e:
        print(f'sweep_gen.py: {e}', file=sys.stderr)
        return 2
    if line is not None:
        argv = line + ['--regen']
    else:
        argv = merged_with_kept(argv)
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('shape', choices=('const', 'field', 'range', 'cell', 'vcall', 'outptr', 'fwd', 'walk', 'mix', 'obj', 'void'))
    ap.add_argument('--cell-win', action='append', default=[], dest='cells', metavar='VA[:SIZE]',
                    help='obj: the global cell at VA points at SIZE (default 0x400) random bytes a row')
    ap.add_argument('--at', action='append', default=[], metavar='VA:SIZE',
                    help='obj: SIZE random bytes written at VA itself (a table in the image\'s data)')
    ap.add_argument('--ptrarg', action='append', default=[], metavar='IDX[:SIZE]',
                    help='obj: argument IDX names SIZE (default 0x100) random bytes')
    ap.add_argument('--ptrarg-null', action='append', default=[], metavar='IDX',
                    help='obj: --ptrarg IDX laid as a null pointer a third of the rows, its buffer still drawn '
                         '(a guard against a null argument; item 2271)')
    ap.add_argument('--stub', action='append', default=[], metavar='slot:OFF:NARGS|ADDR:NARGS',
                    help='obj: a vtable slot of this, or a direct callee, answered per row (the answer printed after the arguments)')
    ap.add_argument('--call', action='append', default=[], help='fwd: ADDR:NARGS:ECX:ARGS, in call order')
    ap.add_argument('--a0', default='this', help='walk: the first callback argument, `this` or `this+0xNN`')
    ap.add_argument('--count-only', action='store_true', help='walk: the walker is called a quarter-million times; one record a row (count, first, last, sums), --port an iterator of its (a0, a1) pairs, --rows N rows (default 3) (item 2672, parked 2667)')
    ap.add_argument('--a1', default='this', help='walk: the second callback argument')
    ap.add_argument('--step', action='append', default=[], help='mix: call:ADDR:NARGS:ECX:ARGS | walk:A0:A1 | walk4:OFF, in order; `gated/` prefix')
    ap.add_argument('--glob', type=lambda s: int(s, 0), help='mix: the global cell (the walk4 argument is *(*cell+0x10)+OFF)')
    ap.add_argument('--vbase', type=lambda s: int(s, 0), help='mix: `this` is a written record, [this-VBASE] points at one whose +4 is the displacement `d` (the g column)')
    ap.add_argument('--gate-bare', action='store_true', help='mix: --gate-this with no value')
    ap.add_argument('--gate-this', nargs='?', const=True, default=False, metavar='OFF:MASK', help='mix: gated steps run when the short at [this] != 0 (`this` is a written object), or with OFF:MASK when the byte at this+OFF has MASK set')
    ap.add_argument('--len-at', type=lambda s: int(s, 0), help='mix: a walk length is the sign-extended short at this+LEN_AT, read as `this+K+len`; gated steps run when it is non-zero')
    ap.add_argument('--gate', type=lambda s: int(s, 0), help='mix: gated steps run when [walker+GATE] == 0')
    ap.add_argument('--gate-ne', action='store_true', help='mix: with --gate, gated steps run when [walker+GATE] != 0')
    ap.add_argument('--store', help='mix: OFF:VAL:SIZE:WOFF — the function stores VAL (SIZE bytes) at this+OFF when the gated steps run and [walker+WOFF] != 0; asserted, and nothing else written')
    ap.add_argument('--elems', help='mix: table:LO:HI:STEP (the pointer words of *(*cell+0x10), a table read through --glob) or list:HEAD:NEXT (a linked list from [this+HEAD] by [node+NEXT]); an `each/` step runs once an element, tokens `elem`, `elem+K` (parked 2585, 2589)')
    ap.add_argument('--vbase2', type=lambda s: int(s, 0), help='mix: a second displacement record at [this-VBASE2], its +4 the `d2` column (tokens `this-K+d2`)')
    ap.add_argument('--bit-at', metavar='OFF:MASK', help='mix: a second gate, the masked byte [this+OFF] & MASK, the last extra column; steps prefixed `bit/` run when it is nonzero')
    ap.add_argument('--stale-col', action='store_true', help='mix: the `stale` leftover ecx is a drawn column of its own after the gate (parked 2673), not the g column')
    ap.add_argument('--ret-arg', action='store_true', help='mix: assert the result (eax) is the stack argument, an out-pointer returned')
    ap.add_argument('--own', type=int, default=0, help='fwd, obj: stack arguments the function itself takes')
    ap.add_argument('--argset', action='append', default=[], help='obj: IDX:V,V,.. — argument IDX drawn from the list (90%%)')
    ap.add_argument('--argfrom', action='append', default=[], help='obj: IDX:OFF:SIZE — argument IDX copies the signed SIZE-byte word at OFF (45%%)')
    ap.add_argument('--win', type=lambda s: int(s, 0), default=0x400, help='obj: bytes of random object')
    ap.add_argument('--bias', action='append', default=[], help='obj: OFF:V,V,.. — a 32-bit word at OFF drawn from the list')
    ap.add_argument('--ret', default=None, help='fwd: none (the default), an integer, or last (the last callee\'s answer); '
                    'obj: eax (the default), xmm0 (a float returned in xmm0, item 2537), al (a bool\'s low byte, item 2336) or none, said only for a void function (item 2090: the obj shape '
                    'took fwd\'s none and eight value-returning sweeps compared no eax)')
    ap.add_argument('function', help='`Class::method@00xxxxxx`, as the export names it')
    ap.add_argument('--item', type=int, required=True)
    ap.add_argument('--port', help="the port's Rust expression (`v`, `t` or `_this` in scope)")
    ap.add_argument('--off', type=lambda s: int(s, 0), help='the field offset (field; range: default 0x4)')
    ap.add_argument('--via', type=lambda s: int(s, 0), help='field: the pointer at this+VIA holds the record; vcall: the object whose vtable is called')
    ap.add_argument('--cell', type=lambda s: int(s, 0), help='cell: the global cell that holds the record pointer')
    ap.add_argument('--slot', type=lambda s: int(s, 0), help='vcall: the vtable slot the function calls')
    ap.add_argument('--nargs', type=int, default=0, help='vcall: stack arguments the slot takes')
    ap.add_argument('--tail', action='store_true', help='vcall: the slot is entered by a jmp, not a call')
    ap.add_argument('--base', type=lambda s: int(s, 0), help='vcall: the slot is compared with this base address (parked 2021)')
    ap.add_argument('--base-via', type=lambda s: int(s, 0), help='vcall --base: the base arm calls the object at this+VIA')
    ap.add_argument('--base-slot', type=lambda s: int(s, 0), help='vcall --base: ... its slot OFF')
    ap.add_argument('--handed', default='', help='vcall --base: the constants the function pushes, V,V,.. '
                    '(--base-call: `pN` the function\'s own argument N)')
    ap.add_argument('--base-call', type=lambda s: int(s, 0), help='vcall --base: the base arm calls the same '
                    'object\'s slot OFF handed --handed; the other jumps into --slot with --own arguments '
                    '(item 2112, parked 2095\'s sibling); --via VIA compares the object at this+VIA')
    ap.add_argument('--base-not', action='store_true', help='vcall --base-call: the base arm answers `== 0`')
    ap.add_argument('--place', action='append', default=[], metavar='LOC:EXPR:SIZE',
                    help='obj: the word at LOC (`m+OFF`, `cN+OFF`, `pN+OFF` or an address expression) points at '
                         'SIZE random bytes less EXPR, the offset the function adds — a new `p` window (parked 2113); '
                         'LOC `sN` lays stub N\'s answer as the pointer (a record a callee answers, item 2241)')
    ap.add_argument('--place-null', action='append', default=[], metavar='K',
                    help='obj: --place K (by its order on the line) laid as a null pointer a third of the rows, '
                         'its buffer still drawn (a guard against null; item 2253)')
    ap.add_argument('--wbias', action='append', default=[], metavar='WIN+OFF:V,V|LO..HI',
                    help='obj: a 32-bit word of window m, cN or pN held to the list or range every row; OFF `*` every word')
    ap.add_argument('--fill', action='append', default=[], metavar='WIN:EXPR',
                    help='obj: every 32-bit word i of window cN or pN is EXPR (e.g. `i//3`, the div_3_table)')
    ap.add_argument('--argrange', action='append', default=[], metavar='IDX:LO:HI',
                    help='obj: argument IDX drawn from LO..HI every row (parked 2045)')
    ap.add_argument('--slotbase', action='append', default=[], metavar='OFF:ADDR',
                    help='obj: the stubbed slot OFF laid as the base ADDR half the rows, a 0/1 flag after the '
                         'answers (parked 2095)')
    ap.add_argument('--name', dest='module', help='obj: the module\'s snake name, for an overload whose snake an '
                    'earlier sweep holds (parked 2045: two `WorldData::restrict`s, two `get_land`s)')
    ap.add_argument('--ecx-int', metavar='LO..HI|V,V,..', help='obj: `ecx` an integer drawn from the range or '
                    'the list (90%%), not the object; printed last on the row (parked 2141)')
    ap.add_argument('--edx', metavar='LO..HI|V,V,..', help='obj: `edx` an integer drawn from the range or the '
                    'list (90%%), beside `ecx`; printed after `ecx` on the row (item 2346, parked 2338)')
    ap.add_argument('--edx-win', metavar='WIN', help='obj: `edx` the address of window WIN (m, cN or pN) rather than an '
                    'integer -- a register argument that is a pointer (`world_to_screen`\'s `in_EDX`; item 2801, parked 2791)')
    ap.add_argument('--deref', action='append', default=[], metavar='IDX:K:N',
                    help='obj: stub IDX prints the N bytes its argument K points at, read when it is called '
                         '(a caller\'s local the record names only by depth; item 2379, parked 2348); K `@VA` reads '
                         'the absolute cell VA instead, as it stands when the stub is called (item 2513, parked 2509)')
    ap.add_argument('--stub-write', action='append', default=[], metavar='IDX:WIN+OFF:SIZE:VAL',
                    help='obj: stub IDX writes VAL (SIZE 1, 2 or 4 bytes) at window m, cN or pN\'s word each call: '
                         'a callee that unlinks the head its caller loops over (item 2346)')
    ap.add_argument('--ecx-answer', action='append', default=[], metavar='IDX:OFF[:FLAGOFF]',
                    help='obj: stub IDX answers from its own `ecx`: the word at ecx+OFF, or with FLAGOFF ecx+OFF '
                         'itself when the word at ecx+FLAGOFF is set, else null (a slot of a record laid in a '
                         'window, `kill_group_move`\'s orders; item 2400, parked 2369)')
    ap.add_argument('--ring', action='append', default=[], metavar='HWIN+HOFF:LO..HI:NEXT:PREV:BASE:STRIDE',
                    help='obj: a circular doubly linked list in the c windows LO..HI at BASE+STRIDE*k, its head the '
                         'word at HWIN+HOFF: each node\'s PREV laid as its predecessor on the NEXT walk, a fifth '
                         'of the rows an early stop (item 2400, parked 2369)')
    ap.add_argument('--tame', action='store_true', help='obj: --ecx-int and --edx held to their draw every row, '
                    'no wild word (a register that indexes a table; item 2346, parked 2338)')
    ap.add_argument('--answers', action='append', default=[], metavar='IDX:V,V,..',
                    help='obj: stub IDX\'s answer (by its order on the line) drawn from the list (90%%) (parked 2141)')
    ap.add_argument('--vtat', metavar='WIN+OFF', help='obj: the stubbed vtable is laid at this word of window '
                    'm, cN or pN rather than at `this` (a slot of an object reached through a table; item 2170)')
    ap.add_argument('--vtat-half', action='append', default=[], metavar='WIN+OFF',
                    help='obj: the stubbed vtable laid at this word half the rows, the other half left as drawn '
                         '(a record compared with a class\'s vtable, a `--wbias` of its address; item 2223)')
    ap.add_argument('--answer-win', action='append', default=[], metavar='IDX:SIZE',
                    help='obj: stub IDX answers the address of SIZE random bytes, a new `p` window after the '
                         'places (a callee that answers a record; parked 2151); repeated for one IDX, '
                         'one window a call in call order, the last repeated (parked 2623)')
    ap.add_argument('--cdecl', action='store_true', help='obj: the function is `__cdecl` (a bare `ret`), so '
                    'the caller pops its arguments and the esp check expects them left (item 2181)')
    ap.add_argument('--argeq', action='append', default=[], metavar='IDX:WIN+OFF',
                    help='obj: argument IDX copies the 32-bit word at WIN+OFF (45%%) once every window is laid — '
                         'an argument compared with a placed record\'s field (item 2200)')
    ap.add_argument('--record', action='append', default=[], metavar='IDX[@S]',
                    help='obj: stub IDX\'s calls printed after ` # `, each `ecx,arg..` with a pointer into a '
                         'window by its name, and held to a `log` the port pushes (a tail target\'s hand-off; '
                         'item 2281); `@S` splits them by the row of --sparse table S their `ecx` lies in, a '
                         'token a row and one for the rest (a walk of the leaders table; item 2309)')
    ap.add_argument('--record-order', action='store_true',
                    help='obj, fwd (each --call, `sK:` its place): the --record stubs\' calls one token a call, in the order they were made, each '
                         'headed `sIDX:` — two argument-less callees on one `ecx` ordered (item 2341, parked 2318)')
    ap.add_argument('--record-edx', action='store_true',
                    help='obj: the --record stubs\' calls carry `edx` after `ecx` (`ecx,edx,arg..`) — a hand-off '
                         'in `edx` (`compute_trade`\'s near leader to `trade_value`; item 2400, parked 2369)')
    ap.add_argument('--seq', action='append', default=[], metavar='IDX:N[:T,T..]',
                    help='obj: stub IDX answers N values a row in call order, the last repeated past N, '
                         'printed after the stubs\' answers (a callee called more than once; item 2217); '
                         'the last drawn from the terminators T, so a loop on the answer ends (item 2368)')
    ap.add_argument('--answer-range', action='append', default=[], metavar='IDX:LO:HI',
                    help='obj: stub IDX\'s answer drawn from LO..HI every row, as --argrange an argument '
                         '(an answer the function indexes a table by; item 2217)')
    ap.add_argument('--cell-this', action='append', default=[], metavar='VA',
                    help='obj: the global cell at VA holds `this` itself, so a method that reads the global '
                         'as well as its object sees one object (item 2217)')
    ap.add_argument('--vtat-also', action='append', default=[], metavar='WIN+OFF',
                    help='obj: the stubbed vtable laid at this word of window m, cN or pN as well, `this` '
                         'keeping it (a record whose slots the function asks beside its own; item 2217)')
    ap.add_argument('--alias', action='append', default=[], metavar='WIN+OFF:TARGET[+K]',
                    help='obj: the word at this window offset laid as the address of window TARGET (plus K) half the '
                         'rows, the other half left as drawn (a pointer that must name another window, or its own: '
                         '`check_last_connect`; item 2797, parked 2791)')
    ap.add_argument('--sparse', action='append', default=[], metavar='LOC:STRIDE:N:OFF,SIZE[/OFF,SIZE]',
                    help='obj: LOC points at a zeroed table of N rows STRIDE apart, each row\'s slices `p` '
                         'windows after the pointed buffers and before the places; `@VA` the image\'s own table '
                         '(the leaders table, parked 2201; item 2247). Two forms, not one: `@VA` is the table '
                         'itself at VA; a bare address is a CELL holding a pointer to the table — `@` on a cell '
                         'reads the pointer as row 0 and faults as a wild index (Loop 2744)')
    ap.add_argument('--gtab', action='append', default=[], metavar='CNT:LO..HI:PTR:SIZE',
                    help='obj: a counted array behind image words (item 2709, parked 2699): the count at CNT drawn from LO..HI, '
                         'the array pointer at PTR, SIZE random bytes an element; `c` windows the head and array, `p` the elements')
    ap.add_argument('--link', action='append', default=[], metavar='pA..B+OFF:SIZE:DIR[:T,T..]',
                    help='obj: figure k (window p(A+k))\'s link at OFF drawn from the figures below (`<`) or '
                         'above (`>`) its own index or a terminator (-1), so a walk through them ends '
                         '(`o_up`, an upgrade chain; item 2276)')
    ap.add_argument('--rows', type=int, default=220, help='obj: rows drawn, for an arm few of the 220 reach '
                    '(Loop 2003, 2287\'s line; item 2314)')
    ap.add_argument('--outarg', action='append', default=[], metavar='IDX:K:V,V|LO..HI',
                    help='obj: stub IDX writes a value drawn from the list or range through its argument K '
                         '(unless null), printed after the answers, `o` in a place (a callee that answers through '
                         'the address of the caller\'s local: `get_inside`; item 2325)')
    ap.add_argument('--stub-cdecl', action='append', default=[], metavar='IDX',
                    help='obj: --stub IDX (a direct callee) is `__cdecl`: its arguments are read and the caller '
                         'pops them (`memcpy`, an import the emulator cannot enter; item 2325)')
    ap.add_argument('--write-mod', action='store_true', help='register in the existing or selected sweep target in sorted order')
    ap.add_argument('--regen', action='store_true', help='redraw a generated sweep in place: both files rewritten, '
                    '`mod.rs` left alone; refused when either file is missing (Loop 2003, 2222; item 2298)')
    ap.add_argument('--fwd-gate', metavar='OFF:MASK:VA[:T,T..]',
                    help='obj: a forwarder gated on a byte of gamec, then a call to VA with the caller\'s stale ecx, '
                         '--own arguments and the tail T.. (GroupOut::issue_*; item 2443, parked 2433)')
    ap.add_argument('--cmd-gate', metavar='OFF:MASK:CMD:SIZE:FIELD,..[:C[:PRE]]',
                    help='obj: issue_halt\'s shape -- gamec byte gate, check_accept_issue, add_group, the command '
                         'built in a local, add_command (GroupOut::issue_*; item 2454, parked 2444)')
    ap.add_argument('--cmd-obj', metavar='CMD:SIZE:FIELD,..[:C]',
                    help='obj: issue_come_out / issue_unqueue\'s shape -- a command whose fields are bytes and words '
                         'read off the object argument 0, check_accept_issue, add_command (item 2469, parked 2466)')
    ap.add_argument('--cmd-byte', metavar='CMD[:SIZE]',
                    help='obj: issue_recall\'s shape -- a group id, check_accept_issue, add_group, a one-byte '
                         'command, add_command (item 2481, parked 2472)')
    ap.add_argument('--cmd-seh', metavar='CMD:SIZE:FIELD,..:C:IGN:WIDE:NARROW:DW:DN',
                    help='obj: issue_treaty\'s shape -- a LeaderOut and a leader checked against the image\'s '
                         'leaders table, check_accept_issue, add_command, else an SEH-framed feedback-log arm '
                         '(two Strings, Error::report, two closes) (item 2486, parked 2482)')
    ap.add_argument('--chain', metavar='CELL:OFF:STRIDE:P:I:N:SIZE',
                    help='obj: PathFinder::find_wpath\'s three-level walk -- a global cell, a row word picked by '
                         'argument P, a table word picked by argument I, a SIZE-byte record (item 2495, parked 2484)')
    ap.add_argument('--jump', metavar='IDX:N',
                    help='obj: a function that is one indexed jump over N arms -- argument IDX drawn in 0..N-1 '
                         '(item 2615, parked 2607)')
    ap.add_argument('--table', metavar='VA:STRIDE:ROWS:COLS:SIZE:XOFF:VOFF',
                    help='obj: WorldData::get_bottom\'s walk -- the image\'s table at VA, ROWS rows STRIDE apart, each '
                         'a pointer to COLS record pointers whose i16 at XOFF is the next column (ends the walk) and '
                         'at VOFF the next row (item 2621, parked 2617)')
    ap.add_argument('--row-byte', metavar='OFF[:LOW,..]',
                    help='obj, with --table: each record\'s own row in the byte at OFF (item 2719, parked 2713)')
    ap.add_argument('--row-link', action='append', default=[], metavar='OFF:SIZE:DIR[:T,T..]',
                    help='obj, with --table: a --link over each row\'s records (item 2719, parked 2713)')
    ap.add_argument('--ptrtab', action='append', default=[], metavar='LOC:N:SIZE[:vt]',
                    help='obj: an array of N record pointers at LOC, each a SIZE-byte record (the places come before every --place); `vt` lays the stubbed '
                         'vtable at each record too (item 2782, parked 2772)')
    ap.add_argument('--dry-run', action='store_true')
    args = ap.parse_args(split_gate_this(argv))
    tab_place = []
    for spec_ in args.ptrtab:
        # The tables' places come before every `--place`, so `p<N>` in the other flags counts them first.
        try:
            pb = len(args.ptrarg) + sum(int(sp.split(':')[2], 0) * len(sp.split(':')[3].split('/'))
                                        for sp in args.sparse) + len(tab_place)
            x = ptrtab_expand(spec_, pb)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        tab_place += x['place']
        args.vtat_also = args.vtat_also + x['vtat_also']
    args.place = tab_place + args.place
    if args.table:
        try:
            pb = len(args.ptrarg) + sum(int(sp.split(':')[2], 0) * len(sp.split(':')[3].split('/'))
                                        for sp in args.sparse) + len(args.place)
            x = table_expand(args.table, pb, args.row_byte, args.row_link)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.place = args.place + x['place']
        args.link = args.link + x['link']
        args.wbias = args.wbias + x['wbias']
    if args.jump:
        try:
            args.argrange = args.argrange + jump_expand(args.jump, args.own)['argrange']
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
    if args.chain:
        try:
            pb = len(args.ptrarg) + sum(int(sp.split(':')[2], 0) * len(sp.split(':')[3].split('/'))
                                        for sp in args.sparse) + len(args.place)
            x = chain_expand(args.chain, args.own, len(args.cells), pb)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.cells = args.cells + x['cells']
        args.place = args.place + x['place']
        args.argrange = args.argrange + x['argrange']
    if args.cmd_seh:
        try:
            x = cmd_seh_expand(args.cmd_seh, args.own)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.stub, args.answers, args.record = x['stub'], x['answers'], x['record']
        args.deref, args.ptrarg, args.port = x['deref'], x['ptrarg'], x['port']
        args.wbias, args.at, args.sparse = x['wbias'], x['at'], x['sparse']
        args.argrange, args.answer_range = x['argrange'], x['answer_range']
        args.ret = args.ret or 'none'
        if args.win == 0x400:
            args.win = 0x20
    if args.cmd_byte:
        try:
            x = cmd_byte_expand(args.cmd_byte, args.own)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.stub, args.answers, args.record = x['stub'], x['answers'], x['record']
        args.deref, args.port = x['deref'], x['port']
        args.ret = args.ret or 'none'
        if args.win == 0x400:
            args.win = 0x20
    if args.cmd_obj:
        try:
            x = cmd_obj_expand(args.cmd_obj, args.own)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.stub, args.answers, args.record = x['stub'], x['answers'], x['record']
        args.deref, args.ptrarg, args.port = x['deref'], x['ptrarg'], x['port']
        args.ret = args.ret or 'none'
    if args.cmd_gate:
        try:
            x = cmd_gate_expand(args.cmd_gate, args.own)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.stub, args.cells, args.wbias, args.record = x['stub'], x['cells'], x['wbias'], x['record']
        args.answers, args.deref, args.port = x['answers'], x['deref'], x['port']
        args.ptrarg = args.ptrarg or x['ptrarg']
        args.ret = args.ret or 'none'
        if args.win == 0x400:
            args.win = 0x20
    if args.fwd_gate:
        try:
            x = fwd_gate_expand(args.fwd_gate, args.own)
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        args.stub, args.cells, args.wbias, args.record = x['stub'], x['cells'], x['wbias'], x['record']
        args.port = x['port']
        args.ret = args.ret or 'none'
        if args.win == 0x400:
            args.win = 0x20
    if args.regen and args.write_mod:
        print('sweep_gen.py: --regen leaves mod.rs alone; drop --write-mod', file=sys.stderr)
        return 2
    try:
        if args.shape == 'walk':
            sp, script, tp, test, mod = generate_walk(args.function, item=args.item, a0=args.a0, a1=args.a1,
                                                      count_only=args.count_only,
                                                      rows=args.rows if args.rows != 220 else None, port=args.port)
        elif args.shape == 'mix':
            sp, script, tp, test, mod = generate_mix(args.function, item=args.item, steps=args.step,
                                                    glob=args.glob, gate=args.gate,
                                                    gate_this=args.gate_this or args.gate_bare, vbase=args.vbase, len_at=args.len_at,
                                                    gate_ne=args.gate_ne, ret_arg=args.ret_arg, store=args.store, elems=args.elems,
                                                    stale_col=args.stale_col, vbase2=args.vbase2, bit_at=args.bit_at)
        elif args.shape == 'obj':
            sp, script, tp, test, mod = generate_obj(args.function, item=args.item, win=args.win, bias=args.bias,
                                                    own=args.own, ret=args.ret or 'eax', port=args.port,
                                                    argset=args.argset, argfrom=args.argfrom, cells=args.cells,
                                                    at=args.at, ptrargs=args.ptrarg, stubs=args.stub,
                                                    places=args.place, wbias=args.wbias, fills=args.fill,
                                                    argrange=args.argrange, slotbase=args.slotbase,
                                                    module=args.module, ecx_int=args.ecx_int,
                                                    answers=args.answers, vtat=args.vtat,
                                                    answer_wins=args.answer_win, cdecl=args.cdecl,
                                                    argeq=args.argeq, seq=args.seq,
                                                    answer_range=args.answer_range,
                                                    cell_this=args.cell_this, vtat_also=args.vtat_also,
                                                    vtat_half=args.vtat_half, alias=args.alias, place_null=args.place_null,
                                                    sparse=args.sparse, ptrarg_null=args.ptrarg_null,
                                                    record=args.record, record_order=args.record_order, record_edx=args.record_edx, links=args.link,
                                                    outarg=args.outarg, stub_cdecl=args.stub_cdecl, rows=args.rows,
                                                    edx=args.edx, edx_win=args.edx_win, tame=args.tame, stub_write=args.stub_write,
                                                    deref=args.deref, ecx_answer=args.ecx_answer, ring=args.ring, gtab=args.gtab)
        elif args.shape == 'void':
            sp, script, tp, test, mod = generate_void(args.function, item=args.item, own=args.own)
        elif args.shape == 'vcall' and args.base is not None and args.base_call is not None:
            sp, script, tp, test, mod = generate_vcallbase(args.function, item=args.item, slot=args.slot,
                                                           base=args.base, base_slot=args.base_call,
                                                           handed=[h for h in args.handed.split(',') if h],
                                                           own=args.own, via=args.via, negate=args.base_not,
                                                           win=args.win if args.win != 0x400 else 0x200,
                                                           port=args.port)
        elif args.shape == 'vcall' and args.base is not None:
            sp, script, tp, test, mod = generate_vbase(args.function, item=args.item, slot=args.slot, base=args.base,
                                                       handed=[h for h in args.handed.split(',') if h],
                                                       via=args.base_via, via_slot=args.base_slot,
                                                       win=args.win if args.win != 0x400 else 0x200, port=args.port)
        elif args.shape == 'fwd':
            sp, script, tp, test, mod = generate_fwd(args.function, item=args.item, calls=args.call,
                                                     own=args.own, ret=args.ret or 'none',
                                                     order=args.record_order)
        else:
            sp, script, tp, test, mod = generate(args.shape, args.function, item=args.item,
                                                 port=args.port, off=args.off, via=args.via,
                                                 cell=args.cell, slot=args.slot, nargs=args.nargs, tail=args.tail, own=args.own)
    except ValueError as e:
        print(f'sweep_gen.py: {e}', file=sys.stderr)
        return 2
    # The script keeps the line that drew it, so `--regen SCRIPT` redraws it
    # after the lane's job directory is gone (item 2325, parked 2311).
    script = script.rstrip('\n') + '\n\n' + KEPT + json.dumps([a for a in argv if a not in NOT_KEPT]) + '\n'
    if args.dry_run:
        try:
            mp = sweep_registry(mod, test)
            registered = mod if mp.name == 'mod.rs' else registration(mod).rstrip()
        except ValueError as e:
            print(f'sweep_gen.py: {e}', file=sys.stderr)
            return 2
        print(f'# {sp}\n{script}\n# {tp}\n{test}\n# {mp}\n{registered}')
        return 0
    if args.regen and args.port is None and tp.exists():
        # The test's hand-written port is kept between the generated halves
        # (item 2368: two of 2341's ten restored it by hand from a copy).
        kept = kept_port(tp.read_text(), test, f'todo!("the port of {parse(args.function)[0]}")')
        if kept is not None:
            test = kept
        else:
            print(f'sweep_gen.py: {shown(tp)}: no port block where the new test holds one (its windows '
                  'changed?); the port is `todo!()` again — restore it from git', file=sys.stderr)
    for p in (sp, tp):
        if p.exists() != args.regen:
            print(f'sweep_gen.py: {p} ' + ('is missing; --regen redraws a generated sweep' if args.regen
                                           else 'exists; a sweep is generated once (--regen redraws it)'),
                  file=sys.stderr)
            return 2
    try:
        mp = sweep_registry(mod, test)
        registered = mod if mp.name == 'mod.rs' else registration(mod).rstrip()
        updated = None
        if args.write_mod:
            text = mp.read_text() if mp.exists() else ''
            updated = insert_mod(text, mod) if mp.name == 'mod.rs' else insert_standalone(text, mod)
    except ValueError as e:
        print(f'sweep_gen.py: {e}', file=sys.stderr)
        return 2
    sp.write_text(script)
    sp.chmod(0o755)
    tp.write_text(test)
    if args.write_mod:
        mp.write_text(updated)
    print(f'{shown(sp)}\n{shown(tp)}\n{registered}'
          + ('' if args.write_mod or args.regen else f'  (add to {shown(mp)})'))
    return 0


if __name__ == '__main__':
    sys.exit(main())
