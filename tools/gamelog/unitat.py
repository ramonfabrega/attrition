#!/usr/bin/env python3
"""unitat.py — one unit out of a dump, by name, its path's ends and its orders
first issued first.

    python3 tools/gamelog/unitat.py <dump> <frame | lo-hi> <who>/<o> [--all] [--enums DIR]

Four lanes wrote the same scratch parsers (parked 1981: 1948's `blk.py` and
`ord.py`, 1982's `units.py` and `th.py`; 1739; 1601 (d)): a unit's
`UNITDATA` block names its type by `TypeIndex` number, which
`decomp/enums/TypeIndex.txt` turns into a name by hand; the order kind and
the figure's type share the field name `type`; and the order stack is
printed top first, so "the first order" read off the dump is the last one
issued (parked 1739). This prints one frame — the header, the path's two
ends (`--all` every entry), the orders first issued first with the top
marked, the figures — or a window, `lo-hi`: the first frame whole and then
only the lines that changed, a per-unit history across the window (1601
(d)). `--enums DIR` names the enum export (`~/ghidra-projects/decomp/enums`
by default, outside the repository); without it the number stands alone.
Exits 2 when the unit is absent on every frame asked.
"""
import os
import sys

DEFAULT_ENUMS = os.path.expanduser('~/ghidra-projects/decomp/enums')


def type_names(enums):
    """`{number: NAME}` from `TypeIndex.txt`; duplicates joined with `|`."""
    names = {}
    path = os.path.join(enums, 'TypeIndex.txt')
    if not os.path.isfile(path):
        return names
    for line in open(path):
        parts = line.strip().split('\t')
        if len(parts) == 3 and parts[0].lstrip('-').isdigit():
            n = int(parts[0])
            names[n] = f"{names[n]}|{parts[2]}" if n in names else parts[2]
    return names


def _node():
    return {'fields': [], 'children': []}


def parse_block(lines):
    """A `BEGIN` tree from the block's lines (the first is `BEGIN UNITDATA`)."""
    root = _node()
    stack = [(-1, root)]
    for raw in lines:
        s = raw.rstrip('\n')
        stripped = s.lstrip(' ')
        if not stripped:
            continue
        indent = len(s) - len(stripped)
        while len(stack) > 1 and stack[-1][0] >= indent:
            stack.pop()
        if stripped.startswith('BEGIN '):
            node = _node()
            stack[-1][1]['children'].append((stripped[6:].strip(), node))
            stack.append((indent, node))
        else:
            key, _, val = stripped.partition(' ')
            stack[-1][1]['fields'].append((key, val))
    return root['children'][0][1] if root['children'] else root


def units(path, lo, hi):
    """Yield `(frame, block lines)` for every UNITDATA block on frames in `[lo, hi]`."""
    frame = None
    block = None
    depth = None
    with open(path, encoding='utf-8', errors='replace') as fh:
        for line in fh:
            s = line.rstrip('\n')
            stripped = s.lstrip(' ')
            indent = len(s) - len(stripped)
            if stripped.startswith('BEGIN FRAME '):
                if block:
                    yield frame, block
                    block = None
                frame = int(stripped.split()[2])
                continue
            if frame is None or not (lo <= frame <= hi):
                continue
            if block is not None:
                if stripped and indent <= depth:
                    yield frame, block
                    block = None
                else:
                    block.append(s)
                    continue
            if stripped == 'BEGIN UNITDATA':
                block = [s]
                depth = indent
        if block:
            yield frame, block


def field(node, key, default='?'):
    for k, v in node['fields']:
        if k == key:
            return v
    return default


def deep_field(node, key, default='?'):
    """The first `key` anywhere under `node`, depth first."""
    v = field(node, key, None)
    if v is not None:
        return v
    for _, child in node['children']:
        v = deep_field(child, key, None)
        if v is not None:
            return v
    return default


def identity(unit):
    return int(deep_field(unit, 'who', -1)), int(deep_field(unit, 'o', -1))


def describe(unit, names, every):
    """The lines for one unit on one frame, without the frame prefix."""
    guys = [g for n, g in unit['children'] if n == 'GUY']
    types = sorted({field(g, 'type') for g in guys})
    tname = ' '.join(f"{t} {names.get(int(t), '')}".strip() if t.lstrip('-').isdigit() else t for t in types)
    head = (f"uid {deep_field(unit, 'uid')} type {tname or '?'} hits {deep_field(unit, 'myhits')} "
            f"at ({deep_field(unit, 'x_internal')},{deep_field(unit, 'y_internal')}) "
            f"inside_up {field(unit, 'inside_up')} inside_down {deep_field(unit, 'inside_down')} "
            f"up {deep_field(unit, 'up')} down {deep_field(unit, 'down')} "
            f"near {deep_field(unit, 'near_who')}/{deep_field(unit, 'near_o')} group {field(unit, 'group')} "
            f"idle {field(unit, 'idle')} stance {field(unit, 'stance')} "
            f"orders_at ({field(unit, 'orders_x')},{field(unit, 'orders_y')})")
    lines = [head]
    for n, stack in unit['children']:
        if not n.startswith('STACK'):
            continue
        entries = [c for cn, c in stack['children'] if cn == 'PATHDATA']
        ends = [f"[{i}] ({field(e, 'to_x')},{field(e, 'to_y')}) f{field(e, 'flags')} t{field(e, 'tolerance')}"
                for i, e in enumerate(entries)]
        if not every and len(ends) > 2:
            ends = [ends[0], '…', ends[-1]]
        lines.append(f"path {len(entries)}: " + ' '.join(ends))
    orders = [(n, c) for n, c in unit['children'] if n.endswith('ORDER')]
    # The dump prints the stack top first (parked 1739): reversed here, so
    # [0] is the first order issued and the last one is the top.
    shown = []
    for i, (n, o) in enumerate(reversed(orders)):
        kv = ' '.join(f"{k}={v}" for k, v in o['fields'])
        inner = ' '.join(f"{cn.lower()}.{k}={v}" for cn, c in o['children'] for k, v in c['fields'])
        shown.append(f"[{i}] {n} {kv} {inner}".rstrip() + (' <top>' if i == len(orders) - 1 else ''))
    lines.append(f"orders {len(orders)} (first issued first): " + ('; '.join(shown) if shown else 'none'))
    for i, g in enumerate(guys):
        lines.append(f"guy[{i}] type {field(g, 'type')} at ({field(g, 'x')},{field(g, 'y')}) "
                     f"anim {field(g, 'anim', '?')} who/o {field(g, 'who')}/{field(g, 'o')}")
    return lines


def parse_args(argv):
    opts = {'all': False, 'enums': DEFAULT_ENUMS}
    rest = []
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == '--all':
            opts['all'] = True
        elif a == '--enums':
            opts['enums'] = argv[i + 1]
            i += 1
        else:
            rest.append(a)
        i += 1
    if len(rest) != 3:
        print(__doc__.strip().splitlines()[3].strip(), file=sys.stderr)
        raise SystemExit(64)
    path, frames, unit = rest
    lo, _, hi = frames.partition('-')
    lo = int(lo)
    hi = int(hi) if hi else lo
    who, _, o = unit.partition('/')
    return path, lo, hi, (int(who), int(o)), opts


def main(argv=None):
    path, lo, hi, want, opts = parse_args(sys.argv[1:] if argv is None else argv)
    names = type_names(opts['enums'])
    seen = 0
    last = None
    for frame, block in units(path, lo, hi):
        unit = parse_block(block)
        if identity(unit) != want:
            continue
        seen += 1
        lines = describe(unit, names, opts['all'])
        tag = f"f{frame} {want[0]}/{want[1]}"
        if last is None:
            print(f"{tag} {lines[0]}")
            for l in lines[1:]:
                print(f"  {l}")
        else:
            changed = [l for l in lines if l not in last]
            if changed:
                print(f"{tag} changed:")
                for l in changed:
                    print(f"  {l}")
        last = lines
    if not seen:
        print(f"{want[0]}/{want[1]} absent on frames {lo}-{hi}")
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
