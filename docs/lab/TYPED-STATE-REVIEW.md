# Typed state oracle: working review

**Pilot typed snapshots as an additional diagnostic oracle. Do not replace the
gamelog yet.** Checkpoint 2 validates broad scalar and whole observed GUY-record agreement,
recovers exact terrain bits and exposes state the selected logger records omit.
It also falsifies the idea that PDB field names alone settle container lifetime,
runtime type or logger ownership. The four-step charter is still in progress.
No main score, parser, comparator, queue or simulation code changes here.

The live review is consolidated in **PR #7**, rebased on `origin/main`
`d892281` for this steering pass. PR #9's entire checkpoint-2 tranche is folded
in. PRs #5 and #6 are closed as superseded review surfaces without rebasing;
their replay branches, native witnesses and ledger rows remain available.
The continuation line remains parked at L87 (`3ff12db`), with L88 at `c205210`.
No worktree or branch belonging to the recomp spike is modified.

### Main-loop-adjacent changes to read first

| Change | Disposition and evidence |
| --- | --- |
| Two optional end-frame hooks | In this PR, isolated hook wiring now at `193cc10`, with return-root checks at `b060278`. RVA `0x5329d0` is logger entry; `0x192586` is the normal post-logger continuation. Both are under `RON_STATE_FRAME`, preserve extended state, and exclude restore probes. Main's twelve intervening commits do not touch `tracer.c`. Rebuilt default main/lab COFF objects are byte-identical after timestamp normalization; snapshot variant builds. |
| PR #6's post-graph prerequisite guard | Parked at `1eea6d0`, **not in this PR**. It requires `RON_RESTORE_POSTSTATE` when `RON_RESTORE_POSTGRAPH` is set. The current main/typed-state tracer has no post-graph collector path, so adopting the guard alone is not a prerequisite for this pilot. Read it with the parked replay capability if revisiting that line. |

No collector was launched for the rebase check; the packet remains read-only.
The [steering index](STEERING-INDEX.md) is the single navigation page.

## Measured checkpoint

| Charter question | Evidence | What remains open |
| --- | --- | --- |
| Type the heap | The earlier retained packet joins 1,855 discovered PE32 vtables to complete PDB layouts. In the new packet, active-leader unit registries yield 800 non-null slots, 127 with the logger's active bit, and no unresolved identities or identity disagreements after RTTI dispatch. | These are registry slots, not allocator objects. Live-allocation count/coverage remain unknown. Complete graph traversal and group/ammo membership are unfinished. |
| Agree with the logger | 516,369 of 602,211 printed occurrences agree, representing 179,583 distinct storage references. All 107 UnitData identities link; every field of all 228 observed GUY records agrees. Three WORLD copies independently match 111,600 grid values and 3,600 × 15 cell scalars. Zero scoped mismatches. | 70,460 unmapped, 15,181 unclassified text and 201 ambiguous occurrences remain. Full logger parity is **not** established. |
| Measure the excess | All 58,081 terrain float32 bit patterns are retained. 2,058 corners have ambiguous six-decimal projections; nearest-decimal reconstruction differs at 1,422. Twenty active-flag Animal registry entries under owner 9 have no UnitData record in this frame. | Current bits do not prove initial bits. Registry flags do not prove allocation liveness. The complete excess-field matrix remains unfinished. |
| Head-to-head | [Item 327 state-side result](2026-09-22-market-state-head-to-head.md): 206/206 targeted fields agree, including the empty Merchant slot, regional rare count 4 and wealth bucket 6. First numbers in 3m41s focused work (7m55s including consolidation); no capture. | Endpoint state corroborates the logger, but the packet is after the offer decision. Writer, formula and emptying cause remain unanswered by this snapshot alone; no controlled speedup claim. |

### The capture is the reference game over the measured interval

The snapshot is taken at **logger frame 11,186**, after **trace tick 11,185**.
The selected logging window is `[11183, 11189)`; quit is at 11190. The runner
acquired the shared nonblocking profile lock and used the existing Wine lane
reservation. It exited successfully and restored all five backed-up settings
files; the lane is released.

- 11,191 overlapping frame seeds against run53: **zero differing**.
- Six overlapping dump frames against run123 (11183–11188): **zero differing**
  with WORLD and AMMO excluded because their requested detail differs. This
  comparison does not validate the excluded records.
- Packet: **843,001,856 payload bytes**, 177 ranges; inventory 0 ms at the
  timer's resolution, copy footer 249 ms and completion receipt 250 ms.
- Eight anchors (game pointer/frame plus six inline roots), 312,064 bytes:
  unchanged before/after copying, in overlapping copied chunks and after the
  logger returns. These checks do not cover every child object.

The packet SHA-256 is
`ad64789cd60bc5b9b4616ba16e657b9ba679081a9e9d21a097fcbd5b7d25aad6`.
It lives outside Git at
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-typed-state-market/map-14`.
The native collector was built from `092f387`; analysis tools follow separately.

## What the falsifiers caught

**The calling convention is not the source-language spelling.** The owned
listing of `GameLog::end_frame` consumes the global logger; the normal caller
does not reliably load ECX with a logger pointer. The collector uses the
PDB-resolved global, validates the logging window and preserves integer,
x87 and SSE state. It does not fabricate a restore-probe context for this phase.

**Logging is not assumed pure.** `GameLog::full_dump` and `dump_units` call
`NetDaemon::process_all`. A second hook at the normal `Game::do_frame`
continuation checks the same roots after the logger returns. The live run
reports no changed anchors; authored controls detect a changed root and a
failed return-check read. Other threads remain runnable. Neither this check
nor a frame-boundary pause proves an atomic heap snapshot or rules out an ABA
change between observations.

**A typed pointer does not prove an active container.** Walking all ten unit
registries indiscriminately produced nonsense in inactive players' storage.
The current traversal uses the logger's leader activity flag before following
those containers. Owners 2–7 are explicitly excluded on that basis. The four
remaining registries contain 800 slots: 127 active-flag entries, 673 others.
That distinction is preserved; nobody calls the 800 slots live allocations.

**Declared type is not dynamic type.** The registries declare `Unit*`, but
owners 8 and 9 contain `Animal`. RTTI selects the complete layout, and the PDB
inheritance graph must contain the declared pointer type. The active entries
are six Units for owner 0, 61 for owner 1, 40 Animals for owner 8 and 20 for
owner 9. The first three sets are exactly the 107 dumped UnitData identities.
The last set is additional registered state, not a claim about its gameplay role.

**Stored coordinates are not logged coordinates.** The first scalar pass had
321 disagreements, exactly x/y/z for all 107 records. The established
`SubObjectData::Coord` XOR representation (`docs/FORMATS.md`, coordinates)
accounts for every one. Reports retain the stored word, transformed value and
printed value separately. A wrong-word mutation fails the comparison.

**A field name does not prove logger ownership.** WORLD flattens multiple
containers into the same block. Naively joining every `size` to `World.size`
produced both false agreements and disagreements. The current bridge refuses
all 201 repeated-name occurrences as `ambiguous_logger_ownership`; the test
includes one coincidentally equal value so it cannot pass by equality alone.
This is why the inventory retains every printed occurrence and why a partial
bridge never advertises whole-record agreement.

## Coverage pin and terrain precision

The unread-key denominator is parsed from the source literal, never its
historical comments. At the original base `5c8e6d4`, checkpoint 2 fully matches
164 of 226 path/key pairs on this frame (59 partial, three absent). **On the
rebased `d892281` source, the pin is 367 keys across 19 paths: 152 fully matched,
212 partial, three absent.** Main widened its leader reader and changed the
pin; neither measurement is silently carried over as the other's denominator.
No main parser or coverage pin is edited by this lab.

The current height grid's exact-byte hash is
`7788322c83527366091bc97f37bbe338b36cbbe233c2e2904db8c476e938fbe5`.
The precision experiment projects each exact single to six decimals, then
chooses the nearest single using exact rational distances and ties to even.
The 1,422 differing reconstructions measure information lost by this projection;
they are not 1,422 observed simulation errors.

All 2,058 ambiguous corners still print the same values as run3's initial,
frame-1 and frame-2 grids, and have the same bits as the earlier retained
frame-224 packet. Across the whole grid, however, 661 printed corners differ
from run3 and 401 exact words differ from the earlier packet. We therefore
retain the initial-grid question: equal prints cannot prove equal initial bits,
and the older packet is another scenario at a later-than-initial boundary.
No initial height fixture is replaced on this evidence alone.

## Implementation, controls and cost

`typed_state_export.py` keeps PDB exports outside Git and binds their source
hash/GUID/age to the executable. `frame_snapshot_plan.py` derives root addresses,
sizes and boundary offsets from that export. The native collector has a
separate stream identity, caps inventory at 8,192 records/two seconds and copy
at one GiB/five seconds, excludes its own image and the active stack, rechecks
range metadata, and refuses incomplete reads/writes. Deadlines are checked
between OS calls; they do not preempt a blocked call.

`frame_snapshot.py` requires the completion receipt, both installed hooks, the
logger-return check and a subsequent trace frame. A footer alone is insufficient.
`frame_state.py` decodes the retained roots, registry candidates and exact height
bits. `typed_logger_inventory.py` accounts for every line, while
`typed_logger_compare.py` implements the deliberately narrow scalar bridge.
`typed_height_precision.py` measures decimal projection loss. Unsupported PDB
fields, unknown extents, static-member addresses and virtual bases remain
explicit; reachable-state completeness is never inferred from inline decoding.

The initial native checkpoint's full gate passed: 1,278 tests plus clippy,
formatting, install survey and fixture/paperwork checks. The collector and
reader controls include short transfers, mapping/anchor changes, timeout,
create/close failure, missing/duplicate receipts, wrong frame, malformed
extents, dynamic-type mismatch, inactive containers, false name matches and
256 emitted-hook register/extended-state cases. The default tracer's COFF
object is unchanged after normalizing its build timestamp. The final analysis
checkpoint receives its own focused run and release gate below.

The broad packet is a diagnostic acquisition format, not yet an economical
per-frame recording format: 24,000 packets of this size would exceed 20 TB
uncompressed. Selective typed closure or deltas would need their own completeness
checks. The measured 250 ms copy does not establish steady-state throughput.

## Reproduction and next decision

The generated plan and retained evidence are archived under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-typed-state-market`.
The bound PDB export remains in the earlier
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-typed-state-oracle/bound-export`.
Nothing from the install, the exports or the decoded state enters Git.

```sh
PYTHONPATH=tools/explore python3 tools/explore/frame_snapshot.py CAPTURE PLAN/plan.json
PYTHONPATH=tools/explore python3 tools/explore/frame_state.py INSTALL CAPTURE EXPORT PLAN/plan.json > EXTERNAL_STATE
PYTHONPATH=tools/explore python3 tools/explore/typed_logger_compare.py CAPTURE/gamelog.txt EXTERNAL_STATE EXPORT/types.json > EXTERNAL_COMPARISON
PYTHONPATH=tools/explore python3 tools/explore/typed_height_precision.py EXTERNAL_STATE > EXTERNAL_PRECISION
```

**Pilot:** review the capture format and scalar bridge now; use a selected
snapshot when a concrete question needs hidden storage or exact float bits.
**Park replacement:** complete logger coverage, full rooted liveness and a
controlled live-item head-to-head remain prerequisites for that claim.
Coverage expansion stops at this checkpoint. The [bounded item-327 result](2026-09-22-market-state-head-to-head.md)
answers endpoint state, but cannot establish the earlier offer mechanism from
this packet alone. Steering can compare that temporal contract with PR #8's
independent function experiment. A retained `objects` root already exposes a typed `ammo_objs` array;
it was not an acquisition anchor and is not counted as validated ammo membership.
No further capture is needed for that work on this packet.


### Checkpoint-2 validation and reproduction

Before rebasing, code `b8e4d79` passes 86 acquisition/oracle tests and the full
release gate: 1,278 release tests, 885 fixture requests with none missing,
clippy, formatting and paperwork. Fresh packet-to-report reproduction takes
21.41 seconds and writes an approximately 11 MB gzipped complete comparison.
Synthetic memory-view mutations over the retained packet produce exactly
three tile-mask mismatches and two negative-zero GUY-word mismatches. The
original packet is unchanged. Rebased validation is recorded with this pass.

```sh
PYTHONPATH=tools/explore python3 tools/explore/typed_oracle_compare.py INSTALL CAPTURE EXPORT PLAN/plan.json EXTERNAL_OUTPUT
PYTHONPATH=tools/explore python3 tools/explore/typed_pin_coverage.py crates/rondata/src/diff/coverage.rs EXTERNAL_OUTPUT/comparison.json.gz
```

Detailed evidence: [leader projections](2026-09-22-leader-bridge.md),
[world grids and cell boundaries](2026-09-22-world-grid-bridge.md), and
[complete observed GUY records](2026-09-22-guy-state-bridge.md). The additional
124-output encrypted-leader probe remains unpromoted; it is not part of the
reported coverage and is not the priority for this steering pass.

### Rebased release verdict

At `6151581` on main `d892281`, the full release run executes to completion:
**1,285 passed, two failed, two ignored**. Both failures are inherited from
unchanged main files: the queue's golden word is 626 while its constant is 774,
and landed item 445 remains live in the queue. The run123 whole-frame widening
passes. No queue, journal, constant or guard is changed to hide these failures.
The overall release gate is **red**, not an adoption-ready green verdict.
The 86 acquisition/oracle tests pass independently on the rebased code.

Fixture audit: **896 requests, 142 unique fixtures, zero missing**. Install
survey and offline tool tests pass. Clippy with warnings denied and formatting
pass separately after the red release run; the final lab docs pass the other
15 paperwork tests, with the known item-445 failure explicitly excluded.
Both no-float checks pass. These separate checks do not turn the gate green.
