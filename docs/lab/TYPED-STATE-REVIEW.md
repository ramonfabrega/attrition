# Typed state oracle: working review

The charter is in progress. **Pilot the typed-root approach; do not yet replace
the gamelog.** The first offline experiment joins RTTI to PDB types, reads
selected global roots, follows one plain-struct pointer and retains exact
terrain singles. Logger parity, complete excess coverage and the live-item
head-to-head have not run. This is not an adoption verdict on a completed
four-step experiment, nor a proposal to move a main score.

Work starts from main `5c8e6d4` on `codex/typed-state-oracle`. The continuation
line is parked at L87, commit `3ff12db` on `codex/continuation-lab`; its later
L88 owner-gap observation remains available at `c205210`. Neither branch is
merged into this work. The A* replay stays available for counterfactuals.

## Findings and current limits

| Question | Measured result | Limit / falsifier still owed |
| --- | --- | --- |
| Can RTTI names join the type stream? | 1,855 discovered PE32 vtables join unambiguously to complete PDB layouts. | Discovery is bounded to supported non-construction COLs; not a census of every compiler RTTI variant. |
| Can we type heap bytes? | 120,068 vptr hits normalize to 109,780 private-memory object candidates; their extent union covers 7,234,024 bytes. | A candidate is not a live allocation. Allocation count and live-allocation coverage are explicitly unknown. Freed/pool storage and accidental hits remain possible. |
| Are simulation classes reachable? | RTTI candidates include 400 Unit, 512 Group, 200 Ammo and one Terrain. PDB globals reach World, Leaders and game_random without RTTI. | Candidate counts are not active simulation counts. Root membership and live container extents need validation. |
| Can the type stream drive fields? | Six selected roots emit declared fields, fixed arrays and direct bases, with explicit missing/unsupported rows. | Virtual bases remain unresolved; static members need global addresses. Pointer type alone does not supply a dynamic array length or union discriminator. |
| Can it recover exact terrain singles? | The PDB-derived master_land_heights pointer and container fields expose 58,081 float32 values, matching the world's 241×241 corner dimensions. Exact bits retained externally. | This is map 14 at a mid-frame delegation boundary, not the Great Lakes ambiguity fixture or a logger-equivalence result. |
| Can it reach plain structures? | A declared World.wdata pointer decodes the first WData record: 16 scalar fields and one pointer. | Only one element was requested; full dynamic extent and child pointers are not claimed. |

The census completed in 5.88 seconds on the retained 796.301 MiB packet before
root decoding was added. That timing includes packet validation and cached type
loading; fresh PDB export is separate (21.12 seconds in the measured run).
This is not the head-to-head investigation timing the charter requests.

## Why the denominator and schema need care

VirtualQuery ranges and allocation-base regions are not allocator objects.
RTTI hits cannot establish which pool slots are alive. Therefore the tool emits
`live_allocation_count: null` and `live_allocation_coverage: null`; it reports
candidate byte union separately, never a percentage of live allocations.
The useful first denominator is rooted, semantically live records, once their
registries or container rules are established.

The PDB supplies declared storage and types, but not all runtime meaning:
pointers omit extents, unions omit active tags, globals may have unresolved
addresses and logger fields may be transformed or derived. A generated field
list can prevent silent omissions only if unsupported fields remain failures
or explicit unresolved entries. The prototype preserves these entries, raw
float bits and non-storage record counts. It does not mark all memory decoded.
The selected roots are an explicit pilot scope, not an automatically complete
simulation-root census.

## Reproducible implementation

`typed_state_export.py` invokes llvm-pdbutil on the owned PDB. Exports live
outside the repository, include source hashes and a manifest, and bind GUID/age
to the executable's RSDS identity. The exact decorated RTTI name joins the PDB's
UniqueName, avoiding a demangler's name-normalization ambiguity.

`typed_state.py` checks PE32 COL/type-descriptor/hierarchy links and the first
virtual target, joins layouts, scans retained words and normalizes secondary
vptrs by their complete-object offset. It revalidates the complete source
payload before using its ranges. Unsupported construction displacements are
not interpreted. `typed_fields.py` handles scalar storage, exact float bits,
pointers, arrays, modifiers, enums, bitfields and direct bases. Unsupported
field kinds or missing bytes remain visible. `typed_state_pilots.py` checks an
explicit Array length/size/list contract plus independent world dimensions
before reading the height buffer. This contract is not inferred from `float*`.

The PE32 RTTI shape is cross-checked against LLVM's
[Microsoft C++ ABI emitter](https://github.com/llvm/llvm-project/blob/main/clang/lib/CodeGen/MicrosoftCXXABI.cpp).
Simple type codes and pointer modes follow the installed LLVM CodeView
TypeIndex header. Actual class layouts and offsets come from the owned PDB;
no symbol dump, layout export, original bytes or decoded values enter Git.

```sh
uv run --offline --with pyyaml==6.0.3 python tools/explore/typed_state_export.py \
  INSTALL/sbl/rise.pdb EXTERNAL_EXPORT --llvm-pdbutil /path/to/llvm-pdbutil
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 --with numpy==2.5.1 \
  python tools/explore/typed_state.py INSTALL CAPTURE EXTERNAL_EXPORT/types.json \
  --globals EXTERNAL_EXPORT/globals.txt --root world --root leaders \
  --root game_random --root units --root groups --root terrain
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 python \
  tools/explore/typed_state_pilots.py INSTALL CAPTURE \
  EXTERNAL_EXPORT/types.json EXTERNAL_EXPORT/globals.txt
```

The current capture is
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-memory-payload`.
Its payload SHA-256 remains
`d0b4ce6ca20fcf2c1e17f6d385d48150cf84660c706ae944c3b372897824756f`.
The decoded height bytes hash to
`d21edfd45d0a42cf81e9d52aae319087c0a514087ca0980ef55c40c10382ae62`.

Ten authored tests exercise malformed RTTI chains, secondary vptrs, ambiguous
forward types, missing bytes, extent/budget failures, unresolved global symbols,
fixed arrays, signed bitfields, exact NaN bits and overlap accounting. These
are decoder controls, not agreement with the original logger.

## Remaining charter work

1. Resolve or bound virtual inheritance and rooted container membership;
   report each sim family reached, absent or unresolved. Preserve the unknown
   live-allocation denominator unless independent allocator evidence supplies it.
2. Acquire at the logger's end-frame boundary, through the shared nonblocking
   capture lock. Current trace entry and logger boundary are different phases:
   the owned Game::do_frame calls GameLog::end_frame after advancing the frame.
   Record both frame identities and before/after root checks. A paused simulation
   thread does not freeze other threads or make the 1.6-second copy atomic.
3. Enumerate every logger key on that same frame. Match or classify every
   discrepancy; unsupported keys may not count as agreement. Compare the excess
   against main's 232-key pin, unopened families and exact height information.
4. Answer 520 or 523 (or steering's current successor) from the same scenario,
   with wall time/cost measured against the normal capture-and-widening route.

No new capture has been taken. The next native experiment must first earn its
boundary and framing tests; existing mid-frame bytes cannot answer step 2.

The final focused run passes 31 tests with the owned executable supplied, with
no skips. The final census plus six-root decode takes 4.86 seconds, preserving
all census counts and the exact height hash. It reports 36 unresolved virtual
base entries across the six roots (3 World, 20 Leaders, 3 Units, 3 Groups,
7 Terrain); RNG has none. `reachable_state_complete` remains false even for a
root whose inline fields decode, since pointer reachability is a separate claim.
