# Complete observed GUY records from typed figure arrays

The retained packet now reproduces **516,369 of 602,211 printed occurrences**,
with zero mismatches in the compared scope. This follows the
[world-grid checkpoint](2026-09-22-world-grid-bridge.md), without a new capture.
There are 179,583 distinct agreeing (address, PDB path) references; 336,786
compared occurrences repeat a reference. Whole-frame parity remains false.

## Identity and complete observed records

Following each active, identity-checked unit's PDB-typed `guys` pointer array
reaches 134 figures. The bridge validates declared length/capacity, pointer
width, retained bytes and the figure's `(who, o)` against its owning unit.
The logger identity is `(who, o, guy_num)`, **not the pointer-array slot**.
A duplicated identity, null pointer or missing referent refuses traversal.
No allocator-liveness claim follows from these checks.

All 228 GUY records in the frame link to 114 identities, printed twice across
the frame-level and nested unit/animal GUY scopes. Every one of their 48 printed
occurrences agrees: **10,944 additional comparisons**. Twenty decoded figures
belonging to owner 9 have no logger record and remain explicitly unlogged;
they are not called agreeing or live. The bridge reports completeness of the
observed printed records, not every field in the in-memory Guy object.

`GameLog::dump_guys` at `0092fe90` and the typed unit containers establish the
owning path. `GuyData::log_data` at `005de6c0` establishes the emitted fields,
coordinate-wrapper reads and integer-cast aliases. The bridge retains all
other logger occurrences and refuses ambiguous storage or duplicate field
names rather than selecting whichever happens to match.

## A float-looking call is an unsigned-word call

Five labels reinterpret `turret_inc`, `bank`, `last_bank`, `pitch` and
`last_pitch` as DWORDs. The decompiler displays their float-typed arguments,
but the logger call uses vtable offset `0x18`. Reading the executable's
`GameLog` vtable at `00b39158` gives `0043cef0` for that slot: the unsigned
`GameLog::say` overload. Float output is the different slot `0x14`, pointing
to `0043cf10`.

The bridge compares the PDB decoder's four exact bytes as an unsigned integer;
it never numerically converts or rounds them. All 1,140 observed word
occurrences are zero in this frame. Authored controls exercise negative zero
and a nonzero NaN payload, but **nonzero original-run float-word behavior has
not been observed here**. The unsigned-word interpretation is backed by the
vtable target and overload reading, not inferred from zero-value equality.

## Against the unread-key pin

`typed_pin_coverage.py` parses the actual UNREAD literal in this branch's
`crates/rondata/src/diff/coverage.rs`, rejecting unsupported syntax and duplicate
keys rather than trusting its historical comments. On base `5c8e6d4` that is
226 path/key pairs across 18 paths. Of those:

- **164** have every observed occurrence matched on this frame (70 at PR #7's
  initial scalar checkpoint, 85 after leader projections, 164 with GUY).
- **59** are present but not fully matched.
- **3** are absent and receive no agreement credit.

This does not edit the main parser, reduce its pin or establish coverage on
other frames. The comparison is to the branch's source hash recorded in the
external report, not a claim about an advancing main tip.

## Reproduction and limits

The single-command runner now includes the figure traversal and GUY bridge.
A fresh packet-to-report run takes **21.41 seconds**, gives the same 516,369
matches, and has no bridge errors. The external manifest and complete gzipped
comparison retain the input identities, authored-tool hashes and stage timings.
The report keeps 70,460 unmapped, 15,181 unclassified-text and 201 ambiguous
occurrences visible. It never advertises whole-frame parity.

No pointed figure records were acquisition anchors. Their agreement with both
printed copies is evidence at those observation points, not atomicity or an
all-thread freeze. Both the world checkpoint `cc0bc7e` and expanded figure/pin checkpoint
`b8e4d79` pass the full release gate: 1,278 release tests, 885 fixture requests
with none missing, clippy, formatting and paperwork guards. All 86 focused
acquisition/oracle tests pass. Synthetic byte-view corruption over the actual
retained packet yields exactly three tile-mask mismatches and exactly two
negative-zero GUY-word mismatches; the original packet is unchanged. Fable still chooses adoption. The next technical frontier is the
remaining leader projections and container families, not another capture by
default; item 520's mid-decision causality remains outside this boundary dump.


The review checkpoint is [draft PR #9](https://github.com/ramonfabrega/attrition/pull/9),
stacked on #7. Durable evidence lives under the typed-state-market experiment's
`followup/wide-scalar-checkpoint/`, including the complete compressed comparison,
coverage and pin reports, source hashes, gate logs and synthetic failure controls.
An additional 124 encrypted leader outputs agree in an **unpromoted** probe
(`typed-next-leader-pilot.py` and JSON); they are excluded from every total above.
That is an offline next-pass opener, not a capture request or an adoption gate.
