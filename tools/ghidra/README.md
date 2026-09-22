# Reading the original with Ghidra

The shipped `riseofnations.exe` plus its own `game/sbl/rise.pdb` is the
project's specification (`docs/DECISIONS.md` entry 6). These scripts are how it
is read. They take the user's install and produce reading notes outside the
repo; none of what they produce may be committed (entry 7), and none of it is
ever transcribed into Rust.

Everything here runs headless through `analyzeHeadless`. Nothing needs the GUI.

## Setup, once

```
brew install ghidra openjdk@21          # ghidra is a formula, wants JDK 21
tools/ghidra/analyze.sh <install>       # hours; import + auto-analysis with the PDB
tools/ghidra/export.sh                  # minutes; decompile everything to files
```

`analyze.sh` builds `~/ghidra-projects/ron.gpr` (override with
`RON_GHIDRA_PROJECT`). **Never re-run it against an existing project** — every
other script reopens the analysed program with `-noanalysis` and takes a couple
of minutes, not hours.

## The export is the primary tool

`export.sh` writes, under `$RON_GHIDRA_PROJECT/decomp/`:

| file | what |
| --- | --- |
| `funcs/<Class>/<method>@<addr>.c` | one file per function — 48k of them, every class its own directory |
| `types.txt` | every structure the PDB loaded, offset / type / field name |
| `vtables.txt` | every `Class::vftable`, slot offset → method name |
| `INDEX.tsv` | address, qualified name, file — the map from a symbol to its file |

After that, almost every question is `grep`, not Ghidra:

```
ls decomp/funcs/LeaderData | grep -E 'avail|preq'         # what methods exist
cat decomp/funcs/LeaderData/has_preq@*.c                  # read one
grep -rl 'gain_tech(' decomp/funcs | head                 # who calls it
grep -n 'struct LeaderData ' -A400 decomp/types.txt | grep ' 0x6c18'   # name a field by offset
grep -A60 'vtable Unit::vftable' decomp/vtables.txt | grep '+0xcc'    # name a virtual slot
grep -rn 'field_0x' decomp/funcs/Leader/gain_tech@*.c     # what the typing missed
```

Re-run `export.sh` after any manual change to the project (a retyped
parameter, a renamed field); it is cheap enough to treat as a build step.

## When a pass is still needed

`run.sh <Script.java> [args]` runs one script against the analysed program and
prints its output:

| script | answers |
| --- | --- |
| `DumpCallers.java <out.c> <Class::method>...` | who references this function, and their decompilation — matches the *qualified* name |
| `DumpFuncs.java <out.c> <substr>...` | decompile every function whose *short* name contains a pattern |
| `DumpRange.java <out.c> <startHex> <endHex>` | decompile an address range — the linker keeps a translation unit together, so a range around a known function reads its neighbours |
| `DumpStruct.java <TypeName>` | a struct's fields, by name and offset |
| `DumpVtable.java <Class::vftable> [n]` | a vtable, slot → method |
| `DumpData.java <symbol> [n]` | a data symbol's first n entries as i32 and u8 |
| `DumpRefs.java <out.c> <dataSymbol>` | what *writes* a data symbol — for `.bss` tables filled at startup |
| `DumpAddrRefs.java <hexAddr> [span]` | every function referencing an absolute address |
| `DumpEnum.java <out.c> <Enum> <v1,v2,..>` | the names an enum gives to values — `0x212` → `COLOSSEUM` |
| `DumpEnumAll.java <out.txt> <Enum> [...]` | a whole enum, sorted, to a file — `decomp/enums/TypeIndex.txt` is all 869 names of the type space, and is how a range test in a decompile (`0x227 < i && i < 0x243`) is read as "an epoch" |
| `ExportAll.java <outdir>` | what `export.sh` runs |

With the export in place these are mostly for things the export cannot hold
still: a data dump after a retype, a fresh vtable walk, an address sweep.

## Traps, each of which cost a wrong conclusion once

- **`SymbolTable.getGlobalSymbols(name)` never matches a C++ method.** It
  searches the global namespace only, so it returns empty for `Leader::gain_tech`
  — which reads exactly like "no callers". `DumpCallers` iterates the function
  manager and matches `getName(true)`; `DumpRefs`, `DumpData` and `DumpEnum`
  still use the global lookup and are only right for free functions and data.
- **`DumpFuncs` matches the short name.** `'ObjectType::init'` matches nothing;
  `init` matches everything with `init` in it. Accept the extra hits.
- **"Is this field read anywhere?" cannot be answered by references to its
  address.** Globals are reached through a pointer
  (`GameAccessConst::constantsc->supply_radius`), so a field reports zero
  references whether or not it is read. Read the consumer instead. Likewise
  `DumpAddrRefs` only sees statically resolvable addressing: always include a
  known reader as a positive control before believing a negative.
- **Decode the MSVC decoration before calling a symbol a field.**
  `?x@C@@2PAKA` — the `2` — is a *static* member. That one distinction decided
  `docs/ORACLE.md`'s central claim.
- **A field reached at an offset inside an earlier array is that array.**
  `leader + 0x56fe + type*2` is `num_units` at `+0x5762` with a `-0x32` id
  bias folded into the base. Check `offset + first_index*stride` before naming
  a new field.
- **Constants are not loaded in the representation the file writes.** Whether
  a `rules.xml` value arrives ×256, ×100 or plain is decided by the one line of
  `Constants::init` (or a type loader) that reads it — `get_fraction(…, 0x100)`
  versus `get_item`. Read that line; see `docs/DECISIONS.md` entry 14.
- **Base-class fields are not listed on the derived struct.** `UnitData`
  inherits `SubObjectData`; dump both.
- **A class with two bases has two vtables, and the one the decompile calls
  through is not named `vftable`.** `UnitType` is a `Type` and a `SoundType`,
  and every `(*(code **)(*types[t] + 0x60))()` goes through
  ``UnitType::vftable{for `Type'}`` — which the decompiler prints as
  `vftable_for_Type_`, and which `vtables.txt` did not contain until
  `ExportAll` matched `startsWith("vftable")`. `DumpVtable` now matches names
  on letters and digits only, so either spelling finds it. Re-run `export.sh`
  to get them into `vtables.txt`.
- **A derived class's `this->field_0xNNN` may be a base-class field at
  `NNN − prefix`.** `TerrainOut` (0x6ac0) is `TerrainData` (0x6a80) behind
  a **0x40-byte prefix**, so every `this->field_0x4a4` in a `TerrainOut`
  method is `TerrainData + 0x464` — `master_land_heights.list`, not the
  `PtrArray<IndexBuffer>` the raw offset lands in. `tesselation_level` at
  `+0x4b3c` appearing as `field_0x4b7c` is the check. Before calling a
  field "a second array the dump does not print", subtract the derived
  class's size difference and look again.
- **A fastcall's register arguments are dropped, and the stack pushes
  behind them are printed in their place.** `find_angle@0092d130`,
  `sinx@0092d100` and `cosx@0092d0c0` take their pair in `ecx`/`edx`. The
  decompiler does not track those across the call, so it prints whatever
  it last saw. That has been a constant (`find_angle(0, 0)` in
  `Object::compare_target`), `unaff_EDI`/`unaff_ESI`, a stale
  `pCVar31`/`pCVar32` (the lead in `Ammo::init`), and, worst of all, two
  locals that look plausible: `Ammo::do_damage` printed `find_angle(num_guys,
  index)` and then passed the same two on as `do_damage`'s arguments. Each
  one stood in `docs/COMBAT.md` as fact until a divergence led back to it
  (items 495 and 523). For any call to one of these three, read the
  listing: the two instructions before the `call` that load `ecx` and
  `edx` are the arguments. `grep -n 0x92d130` on an `llvm-objdump` of the
  function finds every site in seconds.
- **`run.sh` must be run with `zsh` or as an executable, and `status` is
  read-only in zsh.** The script once did `status=$?` and died with
  "read-only variable" on every run; it is `rc` now.

## Cheaper than Ghidra

- `game/sbl/rise_z.map` is a 13 MB linker map and a free symbol index:
  `grep -ioE '\?[a-z_0-9]+@Leader@@[A-Z]+' rise_z.map | sort -u` lists a
  class's methods in a second.
- Static tables and strings come straight out of the PE with a few lines of
  Python: image base `0x400000`, map VA→file offset through the section table,
  read the bytes. This is how `sSyncDefines[37]` and the `synclogger.ini` keys
  were recovered. User-facing and config text is UTF-16LE; C-string config
  names are ASCII. Verify any wide string by dumping the bytes around it — a
  regex can anchor mid-string.
