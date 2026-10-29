#!/usr/bin/env python3
"""pdb_layout.py — a dumped key's struct, offset and slot address (Loop 3768).

Item 3763 found the one writer of `leader:treaties` it needed
(`Unit::set_attacking`'s, at `0x00e3a424`) only by grepping the address:
the decompile spells a field reached through a global as an absolute
address, and one reached through a pointer as `field_0xNN` or
`(p + 0xNN)`, never by the dump's name. This maps a dump key to the PDB
struct its logger prints, reads the field's offset from the decompile
export's `types.txt` at run time, and gives the slot's address where a
global table holds the record.

Only our own naming lives here. `types.txt` is the export's (`RON_DECOMP`,
default `~/ghidra-projects/decomp`) and is never copied into the tree; with
no export every lookup answers None.

**RECORD_STRUCTS**, a key's record prefix → the PDB structs tried in order,
each sourced from the logger the harness reads it from:

- `leader:` → `LeaderData`: the `LEADERDATA` block, `LeaderData::log_data`
  (`crates/rondata/src/gamelog.rs` `LeaderDump`; keys built in
  `crates/rondata/src/diff/harness.rs` as `leader:{k}`).
- `city:` → `CityData`: `CityData::log_data@004895c0` (gamelog.rs `CityDump`).
- `build:` → `BuildData`: the `BUILDDATA` block (gamelog.rs `BuildDump`).
- `death:` → `DeathObjData`: `DeathObjData::log_data` (gamelog.rs).
- `group:` → `GroupData`: `GroupData::log_data@0045e1d0` (gamelog.rs
  `GroupDump`; keys `group:{id}.{key}`, `diff/second.rs`).
- `g.` → `GuyData`: `GuyData::log_data@005de6c0` (gamelog.rs `Guy`; keys
  `g.{name}[{k}]`, `diff/second.rs`).
- `order:<kind>.` → the order's own row (`crates/rondata/src/diff/order.rs`
  `OrderDiff::label`'s variant docs): `move` `MoveOrder`, `guard`
  `GuardOrder`, `garrison` `GarrisonOrder`, `gather` `GatherOrder`, `cast`
  `CastOrder`, `ground` `AttackGroundOrder` (`ATTACKGROUNDORDER`), `air`
  `StrafeOrder` then `AirOrder`, `patrol` `GroupPatrolOrder` then
  `GroupOrder`, `group` `GroupOrder` then `GroupMoveOrder`.

**Unsourced, so absent**: a bare key (`group`, `hp`, `pos` — a unit's own,
whose record the key does not spell), `order:kind`/`order:flags`/
`order:target`/`order:length` (the `OrderList` and `UnitOrder` base rows,
kept off until a landing names their struct), `path[…]`, `ammo.`,
`attack.`. A field on a base class the export lists under the base is not
found under the derived struct (`Leader` lists none of `LeaderData`'s).

**GLOBALS**, a record prefix → (table, base address, stride):

- `leader` → `leaders.list` at `0x00e3a390`, stride `0x6eec`: the PDB's
  `Leaders::list` is `Leader[10]` and `Leader` is size `0x6eec` (eight
  bytes past `LeaderData`'s `0x6ee4`); the base and the stride are
  `GameAccessConst::get_scary_console_leader@005833e0`'s
  (`crates/sim/src/flag_reads.rs`, `crates/sim/src/groups_process_view.rs`),
  and `LeaderData` sits at `+0` of `Leader`: `0x00e3a424` is leader 0's
  `treaties` at `+0x94` (`crates/sim/src/fight.rs`, `docs/SCOUT.md`).
"""
import os
import re
from pathlib import Path

RECORD_STRUCTS = {
    'leader': ('LeaderData',),
    'city': ('CityData',),
    'build': ('BuildData',),
    'death': ('DeathObjData',),
    'group': ('GroupData',),
    'g': ('GuyData',),
    'order:move': ('MoveOrder',),
    'order:guard': ('GuardOrder',),
    'order:garrison': ('GarrisonOrder',),
    'order:gather': ('GatherOrder',),
    'order:cast': ('CastOrder',),
    'order:ground': ('AttackGroundOrder',),
    'order:air': ('StrafeOrder', 'AirOrder'),
    'order:patrol': ('GroupPatrolOrder', 'GroupOrder'),
    'order:group': ('GroupOrder', 'GroupMoveOrder'),
}

GLOBALS = {
    'leader': ('leaders.list', 0x00e3a390, 0x6eec),
}

STRUCT = re.compile(r'^struct /[^/]+/(\S+)\s')
FIELD = re.compile(r'^\s+\+0x([0-9a-f]+)\s+.*?\s(\w+)\s*$')
_CACHE = {}


def decomp_dir():
    return Path(os.environ.get('RON_DECOMP') or Path.home() / 'ghidra-projects/decomp')


def clear():
    _CACHE.clear()


def layouts():
    """`{struct: {field: offset}}` from the export's `types.txt`, or {}."""
    path = decomp_dir() / 'types.txt'
    if path not in _CACHE:
        out, cur = {}, None
        try:
            with open(path, errors='replace') as stream:
                for line in stream:
                    m = STRUCT.match(line)
                    if m:
                        cur = out.setdefault(m.group(1), {})
                        continue
                    m = FIELD.match(line)
                    if m and cur is not None:
                        cur.setdefault(m.group(2), int(m.group(1), 16))
        except OSError:
            out = {}
        _CACHE[path] = out
    return _CACHE[path]


def offsets(struct, field):
    """The field's byte offset in the struct, or None (no export, no field)."""
    return layouts().get(struct, {}).get(field)


def prefix_of(what):
    """A key's record prefix: `order:move.dest` → `order:move`,
    `leader:pop` → `leader`, `g.cur_time[0]` → `g`; a bare key → None."""
    if what.startswith('order:'):
        kind = re.split(r'[.\[]', what[len('order:'):], maxsplit=1)
        return f'order:{kind[0]}' if len(kind) > 1 else None
    m = re.match(r'^(\w+)[:.]', what)
    return m.group(1) if m else None


def record_structs(what):
    return RECORD_STRUCTS.get(prefix_of(what), ())


def layout(what, field):
    """(struct, offset) of the first of the key's structs that carries it."""
    for struct in record_structs(what):
        off = offsets(struct, field)
        if off is not None:
            return struct, off
    return None


def slot_address(what, who, offset):
    """The slot's absolute address where a global table holds the record."""
    table = GLOBALS.get(prefix_of(what))
    if table is None or who < 0:
        return None
    _, base, stride = table
    return base + who * stride + offset


def annotation(what, who, field):
    """` — LeaderData @+0x94; grep `field_0x94`, `+ 0x94)`, `0x00e41310``,
    or '' when the export or the key's struct does not answer."""
    found = layout(what, field)
    if found is None:
        return ''
    struct, off = found
    spell = [f'`field_0x{off:x}`', f'`+ 0x{off:x})`']
    addr = slot_address(what, who, off)
    if addr is not None:
        spell.append(f'`0x{addr:08x}`')
    return f' — {struct} @+0x{off:x}; grep ' + ', '.join(spell)
