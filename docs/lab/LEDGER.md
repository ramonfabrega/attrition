# Lab claim ledger

The lab explores methods; steering decides adoption and books queue items.
Decision 29's completion counters remain authoritative. A contract or sweep
is evidence for an item's falsifier, not an additional completion score.
Reports below preserve detailed experiments and limitations; this ledger is
the entry point. Original-game artifacts stay outside Git.

**Current focus:** isolate pre-menu startup failures after the fixed ten-pair
cohort; retain failures and verify observed state for every successful run. The lab owns the live lane until an explicit handoff.
Do not overlap manual Wine/profile operations. The cooperative runner lock
does not protect against arbitrary scripts. This is not a request to start a
Fable worker or capture.

| ID | Claim | Evidence | Adoption effect / cost | Status |
| --- | --- | --- | --- | --- |
| L01 | Own one frame's bytes and reuse the parser instead of retaining the complete capture. | [Streaming captures](2026-09-09-streaming-captures.md); equivalence fixtures. | Lower reader retention; reader migration and release gate. | Ready for steering pilot; `6c956f9`. |
| L02 | Select census frames before decoding; one measured pair fell 78.61 → 28.91 s, with higher RSS. | [Native profile](2026-09-09-native-profile.md). | Less unnecessary parsing; not a memory win by itself. | Ready for pilot; `c728f80`. |
| L03 | Stream shutdown tails and select compared frames; paired RSS about 6.52 GB → 260 MB. | [Shutdown reader](2026-09-09-shutdown-streaming.md). | Lower memory on that workload; preserve shutdown semantics and rerun gate. | Ready for pilot; `fbe9d05`. |
| L04 | Replay can omit unused observations while retaining complete report equality. | [Streaming replay](2026-09-09-streaming-replay.md). | Smaller retention; audit consumers must still request needed fields. | Ready for pilot; `eef695c`. |
| L05 | Indexed replay reduced one workload's RSS about 565 → 118 MB but initially took longer. | [Indexed replay](2026-09-09-indexed-replay.md). | Optional memory/time tradeoff; file access and index lifetime matter. | Optional; `0caf4f8`, `c4fe241`. |
| L06 | Clock-bearing unit preselection reduced a measured replay from 3.579 → 2.844 s with identical output. | [Clock observations](2026-09-09-selective-clock-observations.md). | Avoid unused decoding; timing scope is not the whole suite. | Ready for pilot; `be826c7`. |
| L07 | A read-only viewer can inspect verified replay artifacts and typed field differences. | [Viewer specification](../DEBUG_VIEWER.md); artifact/export tests. | Faster human inspection; pilot on a real divergence. No score credit itself. | Ready for pilot; `0e754fd`, `0855e5a`, `a13abf6`. |
| L08 | Calling the command issuer does not prove emission: package capacity can reject the command after selection state changes. | [35 native cases / Rust decoder checks](2026-09-09-command-loop-experiment.md). | Canonical adapter needs acceptance, emission, and processed-frame witnesses. | Adapter requirement; `e473cf0`. |
| L09 | Live command issuance can travel the native processing path without keyboard input. | [Live command evidence](2026-09-09-live-oracle-unlocks.md). | Basis for manufactured falsifiers; existing demo is not a general scenario adapter. | Lab-only; `e91fbea`. |
| L10 | Scene rendering was suppressed in a short observed window without changing its logged projection. | [Frames 18–34](2026-09-09-live-oracle-unlocks.md). | Candidate render-cost experiment; no device-free startup or full-state proof. | Bounded experiment only. |
| L11 | A live-derived mutating call can be captured and replayed with explicit dependencies and reset. | [Clear-call capsule](2026-09-09-live-call-capsule.md). | Per-function falsifiers; install, captured state, and emulator required. | Lab-only; `2ee7ce0`. |
| L12 | Byte-level read/write permissions and full emulator reset make missing dependencies observable. | [Bounded turn calls](2026-09-09-bounded-turn-angles.md); negative tests. | Safer local conformance sweeps; applies only to declared closure. | Lab infrastructure; `17d852a`. |
| L13 | Natural order-cache and path-deletion calls have bounded native/Rust comparisons. | [Order cache](2026-09-09-natural-order-capsule.md), [path deletion](2026-09-09-natural-path-capsule.md). | Additional item-level evidence; not a replacement for trajectory/value diffs. | Lab-only; `bb3cf4c`, `6c2aaef`. |
| L14 | A mismatch can be minimized into a synthetic Rust regression. | [Injected-fault reduction](2026-09-09-path-counterexample-reduction.md). | Reproducible counterexamples; demonstrated with an injected fault, not a newly found production bug. | Method proven on injected fault; `f1782ae`. |
| L15 | A deterministic congestion scenario reaches native suspended searches. | [Two runs: 1,787 suspensions each](2026-09-09-congestion-probe.md). | Manufacture otherwise absent coverage; command staging and original install required. | Lab scenario; `f4e2943`. |
| L16 | A captured natural search graph supports bounded payload disposal replay. | [223 physical / 202 logical nodes](2026-09-09-natural-search-graph.md). | Local cleanup contract; 52 KiB is guest mapping, not host RSS. Full allocator cleanup unresolved. | Lab-only; `c28d68f`. |
| L17 | Restore-prefix replay reaches the native delegation point; full A* immediately needs uncaptured FS/thread context. | [Restore entry](2026-09-09-restore-entry.md). | Further partial-process emulation has uncertain cost; no full resumption claim. | Parked; `e4cc71e`. |
| L18 | Native menu/Start automation acquires both maps through frame 1400 without clicks. | [Driver and receipts](2026-09-09-autostart.md). | Removes manual capture setup; Wine/display and shared profile still required. | Opt-in; `04684a6`. [Fixed cohort](2026-09-09-startup-cohort.md): 6/10 pairs, 15/19 launches; startup remains open. |
| L19 | Native `+skipIntro` trials preserve every logged body and frame seed in six independent launches. | [Six-launch follow-up](2026-09-09-autostart.md#intro-skip-follow-up). | Removes unnecessary intro playback path; six successes do not diagnose earlier failures. | Opt-in; `1abca1c`. |
| L20 | Full observed acquisitions can be compared with coverage, lifecycle, map/seed, and closing-record checks. | [Comparison tool](../../tools/explore/compare_unattended.py); negative fixtures. | Reusable adoption check; observed projections are not complete native state. | Ready for pilot; `1abca1c`. |
| L21 | Normal application teardown faults after match shutdown; controlled process exit produces valid bounded captures. | [Exit boundary](2026-09-09-autostart.md#exit-boundary-and-limits). | Reliable handling of that known boundary; full destructor path remains open. | Explicit lane behavior; not graceful full-engine teardown. |
| L22 | `focus.sh` now returns 1 on a missed window; bare callers under `set -e` abort. | [Focus tests](../../tools/explore/test_lobby_focus.py); `echelon.sh` / `clickdriver.sh` call sites. | Prevents misdirected clicks; transient misses now abort those legacy captures. | Retain fail-closed in lab; steering must review callers before adoption. `bb3cf4c` is mixed. |
| L23 | The soak digest omits the RNG seed. | [Methodology §4](2026-09-09-methodology-exploration.md#4-the-completion-counters-need-an-external-denominator); `soak::digest`. | Add a stronger determinism assertion after a negative test; this does not improve fidelity scoring. | Open; implementation not claimed. |
| L24 | The core trace parser accepts unknown versions/truncated tails and loses INFO loss notices. | [Synthetic acceptance probe](2026-09-09-methodology-exploration.md#5-make-the-observation-pipeline-machine-readable-and-self-validating). | Finalized captures need a strict health contract; streaming/live readers may need a separate mode. | Open in core parser; strict lab readers are not a global fix. |
| L25 | RSS sampling refusal previously appeared as a fictitious zero peak. | [Measured failure](2026-09-09-methodology-exploration.md#7-gate-cost); current `memcap.sh` fail-closed paths. | Avoid unmonitored runs; sampling denial now aborts instead of reporting zero. | Implemented on branch; shared-script adoption needs gate. |
| L26 | One gate read run13 fifty times; decoding improvements do not eliminate all duplicate reads. | [Profiled gate](2026-09-09-methodology-exploration.md#7-gate-cost). | Candidate fixture/index reuse; shared caches introduce lifetime and isolation costs. | Open optimization; no blanket cache proposed. |
| L27 | Whole-function literal signatures are unnecessary for the clear-call compatibility check. | `live_capsule_probe.h`; fingerprint validation and mutation test. | Replace literal bytes with a 64-bit compatibility fingerprint; SHA-256 image binding remains separate. | Validated: authored vectors, 168 installed-code bit mutations, Windows cross-compile/link. Current header contains only the fingerprint; published history is unchanged. |

| L28 | Both WinMain MF calls returned before three reproduced startup failures; two faults map to WoW64 transition RVA 0x1139. | [Startup transition diagnostics](2026-09-09-startup-transition.md); two fixed four-pair cohorts and dual-mode installed-byte decode. | Opt-in call witnesses and bounded failure contexts; wrong-mode execution is a hypothesis, not a runtime fix. | Prior ORACLE/reproducer evidence subsequently located; see L29. Runtime mechanism still open. No score move. |

| L29 | RNG/frame and modal hooks can preserve their ABI without bulk register/flags restores, following the existing coverage-stub rule. | [Migration and current reproduction limits](2026-09-09-hook-restore-migration.md); 512 emitted/compiled ABI cases and negative opcode checks. | Shared hook emitter is a separate adoption concern; optional live capsule wrappers still need migration. The startup fault persists. | Validated compatibility hardening, not a complete startup fix; fixed cohort recorded in report. |

## Adoption boundary

Steering may adopt individual concerns against its current tip and release gate.
The branch is not a whole-merge proposal. Some historical commits mix lab tools
and shared-file changes; their hashes above are navigation aids, not promises
that every commit is already an isolated cherry-pick. Future shared-file changes
are separate commits with rationale. Ask through Ramon for the live lane.

The one-time cutover moves only this branch's newly added reports/artifacts to
`docs/lab/`, preserves its appended narrative in [HISTORY.md](HISTORY.md), and
restores `docs/JOURNAL.md` to the unchanged pre-lab prefix. Subsequent lab work
writes here, not QUEUE, JOURNAL, DECISIONS, or CLAUDE. Core document/code reference
updates are a separate adoption concern. No completion counter or queue item
was changed by the cutover.
