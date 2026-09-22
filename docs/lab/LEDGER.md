# Lab claim ledger

The lab explores methods; steering decides adoption and books queue items.
Decision 29's completion counters remain authoritative. A contract or sweep
is evidence for an item's falsifier, not an additional completion score.
Reports below preserve detailed experiments and limitations; this ledger is
the entry point. Original-game artifacts stay outside Git.

**Current focus:** independent allocator replay research on
`codex/replay-allocator-lab`, while draft PR #4 remains stable for Fable's review;
start with [the review handoff](JEV-REVIEW.md) for that earlier tranche. The lab
is not holding the capture lane. Ask through Ramon before any new live run; the
cooperative runner lock does not protect against arbitrary profile/Wine scripts.
No new capture or main-loop adoption is requested by this handoff.

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
| L23 | The soak digest omits the RNG seed. | [Methodology §4](2026-09-09-methodology-exploration.md#4-the-completion-counters-need-an-external-denominator); `soak::digest`. | Gameplay seed added after a failing RNG-only regression; all 32 bit changes and an extra draw are detected. | Closed for the identified omission; [evidence and limits](2026-09-09-soak-rng-digest.md). Not exhaustive state coverage or fidelity scoring. |
| L24 | The core trace parser accepts unknown versions/truncated tails and loses INFO loss notices. | [Synthetic acceptance probe](2026-09-09-methodology-exploration.md#5-make-the-observation-pipeline-machine-readable-and-self-validating). | Finalized captures need a strict health contract; streaming/live readers may need a separate mode. | Finalized core file reads now reject framing/version/loss violations; permissive parse remains diagnostic. See L33. |
| L25 | RSS sampling refusal previously appeared as a fictitious zero peak. | [Measured failure](2026-09-09-methodology-exploration.md#7-gate-cost); current `memcap.sh` fail-closed paths. | Avoid unmonitored runs; sampling denial now aborts instead of reporting zero. | Implemented on branch; shared-script adoption needs gate. |
| L26 | One gate read run13 fifty times; decoding improvements do not eliminate all duplicate reads. | [Profiled gate](2026-09-09-methodology-exploration.md#7-gate-cost). | Candidate fixture/index reuse; shared caches introduce lifetime and isolation costs. | Refreshed with explicit install: run13 still read 47 times; see L30. No blanket cache proposed. |
| L27 | Whole-function literal signatures are unnecessary for the clear-call compatibility check. | `live_capsule_probe.h`; fingerprint validation and mutation test. | Replace literal bytes with a 64-bit compatibility fingerprint; SHA-256 image binding remains separate. | Validated: authored vectors, 168 installed-code bit mutations, Windows cross-compile/link. Current header contains only the fingerprint; published history is unchanged. |

| L28 | Both WinMain MF calls returned before three reproduced startup failures; two faults map to WoW64 transition RVA 0x1139. | [Startup transition diagnostics](2026-09-09-startup-transition.md); two fixed four-pair cohorts and dual-mode installed-byte decode. | Opt-in call witnesses and bounded failure contexts; wrong-mode execution is a hypothesis, not a runtime fix. | Prior ORACLE/reproducer evidence subsequently located; see L29. Runtime mechanism still open. No score move. |

| L29 | RNG/frame and modal hooks can preserve their ABI without bulk register/flags restores, following the existing coverage-stub rule. | [Migration and current reproduction limits](2026-09-09-hook-restore-migration.md); 512 emitted/compiled ABI cases and negative opcode checks. | Shared hook emitter is a separate adoption concern; optional live capsule wrappers still need migration. The startup fault persists. | Validated compatibility hardening, not a complete startup fix; fixed cohort recorded in report. |

| L30 | A fixture-name guard retained two copies of four captures solely to compare list identity. | [Refreshed profile and isolated measurement](2026-09-09-fixture-list-memory.md); deliberate wrong-name rejection. | Direct ordered-name comparison: isolated RSS 2,177,056,768 → 2,736,128 bytes. This is not a full-suite RSS claim. | Implemented as a separable test-only concern; no parser or simulation change. |
| L31 | This Codex worktree needs explicit RON_INSTALL; earlier passes could skip install-dependent test bodies. | [Validation gap](2026-09-09-fixture-list-memory.md#validation-gap-found-during-the-refresh); testenv::install_root lookup. | Explicit-install profile passed in 240.40 test seconds at 9236 MiB sampled tree peak; earlier 35-second runs are not equivalent gates. | Explicit-install release runner added; see L33. Direct cargo and optional-corpus coverage remain separate. |

| L32 | Indexed sibling text preserves run71's complete replay report and every retained Initial field. | [Tradeoff and evidence](2026-09-09-sibling-retention-and-gates.md). | Isolated RSS 2.225 → 1.495 GB; elapsed 7.47 → 8.58 s. Observation arrays still grow with data. | One test migrated; no blanket speed or suite-RSS claim. |
| L33 | Explicit-install orchestration and finalized trace validation prevent two silent validation gaps. | [Gate and trace boundaries](2026-09-09-sibling-retention-and-gates.md); authored negative cases, 87-file preflight. | Required install passed to all gate children; malformed supplied traces fail visibly. | Missing optional captures and semantic completeness remain open. |

| L34 | Optional fixture requests can disappear behind passing test counts. | [Request audit and empty-directory check](2026-09-09-fixture-request-audit.md); test-only lookup records and retained gate report. | Shows present/missing requests per test; optional strict policy. Does not infer unrequested dependencies or complete corpus coverage. | Implemented in explicit-install gate; no new universal fixture requirement. |

| L35 | Soak activity counted replay digests containing the frame counter; frozen gameplay yielded 401 distinct states. | [Negative control and separate activity signal](2026-09-09-soak-activity.md). | One traversal now separates selected gameplay activity from replay clock/RNG state; existing thresholds unchanged. | Implemented and regression-backed. Useful mechanic coverage and full state equality are not claimed. |

| L36 | Finalized traces previously accepted duplicate, skipped, reset, or internally inconsistent FRAME records. | [Sequence contract and negative regression](2026-09-09-trace-frame-sequence.md); 87-file offline preflight. | Matching frame fields and consecutive signed frames checked in the existing validation pass. | Single-run integrity only; endpoint completeness and trace/dump identity remain open. |

| L37 | Python acquisition receipts ignored positive loss notices and conflicting FRAME fields despite complete lifecycle/endpoint checks. | [Failing receipts and stored revalidation](2026-09-09-receipt-transport.md); 19 focused tests and four retained captures. | Shared streamed/in-memory validator now rejects both; equal malformed acquisitions also fail comparison. | Transport/lifecycle only; no new native experiment or fidelity claim. |

| L38 | Receipt and acquisition Python regressions were absent from the release gate. | [Offline lane and injected failure](2026-09-09-offline-test-lane.md); 27 reviewed tests. | Explicit module list runs before survey/release and propagates failure; direct use needs no install. | Reviewed offline selection, not an OS sandbox; new modules require review. |

| L39 | Leader-window setup retained whole sibling captures while ticking; only parsed initial state is needed for construction. | [Exact output and clean isolated measurements](2026-09-09-leader-fixture-memory.md); 112,548 leader values unchanged. | Isolated RSS 1.56–1.57 GB → 0.86–1.07 GB; roughly 0.8 s more test time across two tests. | Scoped memory/time tradeoff; no full-suite RSS or speedup claim. |

| L40 | Indexed observation chunks were lazily indexed immediately before a full observation walk reparsed them. | [Profile and clean timing](2026-09-09-indexed-single-pass.md); complete sibling Initial equality. | Existing eager parser used only for bounded observation chunks: isolated leader pair 6.41–6.42 s → 5.35–5.36 s. | About 17% in this workload; no universal speed or memory claim. |

| L41 | Single indexed frame decoding repeated a lazy indexing pass immediately before a full decode. | [726 complete decoded records and isolated timing](2026-09-09-indexed-frame-decode.md). | Existing eager parser reduces the five-capture clock test from 4.32 s to 3.64–3.65 s. | About 16% for this workload; no full-suite speedup or universal RSS claim. |

| L42 | Reader memory reductions leave room to revisit the two-thread gate policy. | [Monitored three/four-thread experiments](2026-09-10-test-concurrency.md); four-thread runs 118 s versus fresh two-thread control 240.43 s, with identical fixture-request records. | Four-thread sampled peaks 12,596–13,662 MiB under 20 GiB; candidate for an explicit monitored lane. | Benchmark omits only width assertion; default and guard unchanged pending deliberate adoption. |

| L43 | Four-thread throughput previously required filtering the conservative width assertion in benchmarks. | [Monitored opt-in gate and negative controls](2026-09-10-monitored-wide-gate.md). | Full four-thread gate passed in 117.51 rondata seconds at 13,016 MiB, no filtered tests; all 550 fixture requests present. | Direct-Cargo default stays two; environment marker is cooperative, not tamper-proof. |

| L44 | Repeated sibling scans recover about 1.22 MB of owned observations from 1.09 GB of source. | [Bounded observation reuse, warmed equality and timing](2026-09-10-observation-cache.md). | 8 MiB accounted / 32-entry cache; leader pair 5.16–5.17 s → 4.02–4.09 s. | Metadata identity, independent caller copies; measured RSS higher, no universal speed claim. |

| L45 | Road regressions keep whole sibling captures through ticking and comparison. | [Scoped construction and isolated RSS measurements](2026-09-10-road-test-memory.md). | Market-road peak 3.21–3.24 GB → 1.90–1.93 GB; road rings 1.81–1.82 GB → 1.02 GB. | Cold tests about 7% and 12–15% slower; assertions unchanged, no whole-suite claim. |

| L46 | Scoped road construction still retains whole primary captures. | [Indexed primary inputs and complete-field comparison](2026-09-10-road-primary-indexing.md). | Market-road RSS 1.91–1.95 GB → 1.03 GB; rings 0.97–1.02 GB → 0.53–0.55 GB, no observed cold-time penalty. | Correction scan remains; whole comparison capture is a separate opportunity. |

| L47 | Market-road comparison loads run72 whole for one frame. | [Bounded frame, WORLD fields and heights equality](2026-09-10-road-frame-indexing.md). | Peak 1.03 GB → 0.57–0.60 GB, runtime about 6.1 s. | Offset scan remains; no universal or whole-suite gain claimed. |

| L48 | PR #3 needs a reproducible introduction and a clear adoption boundary. | [One-command diagnostic demo and package comparison](2026-09-10-review-demo.md), [adoption map](PR3-ADOPTION.md). | Same city-field witness in 130-record reader comparison and three-record focused replay; artifact 37.7 MB → 0.91 MB. | Diagnostic-window reduction only; full replay inputs and external install remain required. |

| L49 | Focused diagnostics still replay every prefix from setup. | [Owned session checkpoint and recorded-input counterexample](2026-09-10-replay-checkpoint.md). | Run69 clone 1.3 ms, clone + three-record suffix + checks 12.7 ms; run7 restored complete report equals existing replay. | In-memory, full-history clone; no portable snapshot or capture-content binding. |

| L50 | The exporter still restarts setup for every nearby window. | [Checkpoint export, global cursors and complete-window checks](2026-09-10-checkpoint-export.md). | Three restored windows match uninterrupted replay; clone/continue/write 19–22 ms each after prefix. | One open source handle plus metadata; demo hashes inputs; no persistent or portable snapshot. |

| L51 | Need controlled hypotheses, not only fast window replay. | [Figure-record intervention, restricted-export null and clone profile](2026-09-10-figure-intervention.md). | Ten future figure records removed; 130 restricted exports agreed (Gaia hidden; corrected by L52). Report clone+drop 0.71 ms versus sim 0.52 ms. | Local negative result; no root-cause or complete heap-size claim. |
| L52 | Intervention effects were hidden by player-only export linkage; whole figure removal also suppresses Gaia reseating. | [Expose Gaia and isolate clock installation](2026-09-10-intervention-observability.md). | Whole-record removal changes unit state on frames 95–105; clocks-only removal has no measured effect through 3000, from either checkpoint. | One capture; corrected prefix/setup and RNG policy retained; no fidelity score moved. |
| L53 | Correction installation counts include no-op writes. | [Opt-in overwrite audit and paired replay survey](2026-09-10-correction-overwrites.md). | Three intervals: zero changed clocks/seeds; Great Lakes seven actual Gaia reseat changes, East Indies none. | Shared sibling inputs; not corpus-wide redundancy or a policy change. |
| L54 | Seven actual reseat writes need individual causal checks. | [Single-write interventions through the complete run69 capture](2026-09-10-single-reseat-interventions.md). | Tick 99 leaves a 556-frame orientation difference; at frame 106 only control matches the original heading. All seven branches converge by 3001. | One capture, one-write removals; player score unchanged, no redundancy claim. |

| L55 | A saved graph does not yet establish runnable search resumption; semantic search may help find the missing evidence. | [Real Jev investigation and explicit thread-state probe](2026-09-19-jev-resumption.md). | 171 passages searched for five questions; modeled original prologue advances from FS refusal at instruction 5 to `div_3_table` read at 14, with null memory still unmapped. | Authored thread/argument state, no live replay; Jev misses one half of a compound question in its top five. |

| L56 | The modeled resumed-search prologue needs a coordinate table before any table entry can be used. | [Original initializer sweep and next registry refusal](2026-09-19-coordinate-table.md). | 75,456 entries across six extents match floor division; explicit table input advances attempted instruction 14 to 22. | Successful allocation modeled; live extent and unit registry not captured, no resumed A* return. |

| L57 | The optional restore collector still used bulk register/flags saves and discarded a validated registry header. | [ABI migration and fresh native version-2 packet](2026-09-19-restore-collector-abi.md). | 256 authored adapter states pass; one authorized capture completes and its restore wrapper matches live delegation in 31 instructions, with retained registry. | One live run; downstream search still refuses FS:[0]. Thread/table acquisition and full resumption remain outstanding. |

| L58 | Live thread/table inputs are still missing from the suspended-search packet. | [Bounded delegation context sidecar](2026-09-19-restore-context.md). | Offline refusal checks pass; one September-21 capture yields 19,200 table entries matching regeneration and a 256-reset wrapper replay. | One live sidecar; FXSAVE retained but not loaded, no emulated resumed search result. |

| L59 | Captured thread/table inputs have not yet been consumed in resumed-search replay. | [Native A* entry agreement and unit-header refusal](2026-09-21-resume-frontier.md). | 50-instruction entry matches all four native arguments; 64 resets and fresh engine agree; instruction 73 refuses unit +9. | Prefix only; no FXSAVE import or full A* return, no new capture. |

| L60 | A byte-at-a-time expansion would keep requiring adjacent unit fields. | [Version-2 complete unit context](2026-09-21-unit-context.md). | Live 344-byte record and graph overlap validate; replay advances from instruction 73 to 91, refusing the world pointer; scenario projections agree. | One v2 capture; no full A* return, pointer targets not implicitly captured. |

| L61 | Measure broader acquisition cost before replacing repeated per-field captures. | [Bounded memory inventory](2026-09-21-memory-inventory.md). | C/Python metadata agreement, 28 failed/short queries, caps, deadlines and file controls pass; optional imports verified; release gate passes. | Live: 873 ranges, 795.676 MiB broad candidate; known-root allocations plus image 36.566 MiB. Small selection tested offline; subsequent payload acquisition is L62. No full replay claim. |

| L62 | A broad candidate fits 1 GiB, but needs bounded acquisition before offline replay can consume it. | [Streaming memory payload](2026-09-21-memory-payload.md). | Actual C producer/Python consumer agree; transfer, mapping, anchor, deadline and cap failures refuse; three DLL build variants, sanitizers and the release gate pass. | Live: 796.301 MiB in 1,617 ms; 77,156 anchor bytes agree, projections unchanged. Non-atomic; no full replay yet. |

| L63 | Determine whether a broad packet removes the repeated per-field capture dependency. | [Payload-to-allocator frontier](2026-09-21-payload-frontier.md). | Nine data boundaries closed with 36,256 added bytes; fresh processes reach the same 382-instruction CRT malloc refusal. | Exploratory source retained outside git; extended state not imported, no native output comparison or allocator substitution. |

| L64 | Can a bounded allocation-success model advance the retained replay without more capture? | [Modeled allocator](2026-09-21-modeled-allocator.md). | 72 allocations, 128,717 instructions, 153,952 added captured bytes; repeated and extended-state-perturbed runs agree at a 96-byte memset refusal. | Modeled allocator only; no A* return, no full-state/native fidelity claim; no new live run. |

| L65 | Can explicit memory services and object lifetimes close the retained call? | [Complete modeled resumption](2026-09-21-modeled-resumption.md). | 197,650 instructions; outer return 8; internal A* return 1 matches native; eight resets and two fresh controls agree on their stated observations. | All 640 path-slot bytes retained; no native post-path comparison or FXSAVE import; no live capture or main score change. |

| L66 | The completed replay needs a paired native post-return witness. | [Optional native post-state observer and comparator](2026-09-21-native-poststate.md). | Authored C/Python agreement, negative comparisons, sanitizers and three DLL variants pass; observer retains the entire unit and path capacity. | Prepared offline only; no live native post-state or fidelity result yet. |

| L67 | Does the completed modeled call reproduce native output? | [Paired native path agreement](2026-09-21-native-path-agreement.md). | One authorized capture: outer return 8, all 640 path bytes and all 344 unit bytes except the explicit path pointer agree; eight replay resets; agreement assertion passes. | Single-call unit/path fidelity; other mutated state and general runtime behavior unproven. Lane released; main score unchanged. |

| L68 | Can the validated packet support useful input experiments? | [Request versus continuation-budget interventions](2026-09-21-path-interventions.md). | 35 repeated trials / 30 distinct settings; all controls restored. Request edits leave the saved route unchanged; limit 95 suspends and 96 completes in the model. | Altered inputs not native-confirmed. Candidate: explicit native limit-95 witness; no capture booked or main score moved. |

## Current direction

The September 19–21 tranche has a native-confirmed unit/path baseline and
controlled offline interventions. Retargeting the request record does not
retarget its saved search; changing the continuation budget does change the
outcome. The next candidate is an explicitly recorded native limit-95 test
against the modeled 95/96 suspension/completion witness. No slot is booked.
Other mutated state and general runtime behavior remain open.
Fable decides whether any method merits a pilot, adoption, further evidence or
parking; review does not imply approval to merge the whole lab branch.

The runtime factor-isolation experiment is paused at the user's request after
a product cybersecurity restriction; no four-way results exist. Its draft
files, completed ABI-only check, and the independent replay-memory target are
recorded in [the handoff](2026-09-09-paused-runtime-experiment.md). L29 remains
compatibility hardening with unresolved startup reliability, not a runtime fix.

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
