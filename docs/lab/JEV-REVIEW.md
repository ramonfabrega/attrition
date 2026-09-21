# Jev and retained-state replay: findings for steering review

The purpose of this review is to surface findings and let Fable challenge their
value. Adoption, a small pilot, stronger evidence, or parking the work are all
valid outcomes. This is not a request to advance the main queue, accept a new
completion score, or merge the whole branch as a unit.

The September 19–21 tranche starts at `30b5bae`, after the previously merged lab
work, and its experimental tip is `4a38663` on `codex/jev-lab`. There are thirteen
research commits before this review packaging. The lab is not holding the
capture lane. Coordinate any future run through Ramon; no capture is requested
by this review.

## What changed our understanding

| Finding | Evidence | What it does not establish |
|---|---|---|
| Jev can help locate evidence that a lexical shortlist misses. | [Evidence experiments](2026-09-19-jev-evidence.md), then [one real investigation](2026-09-19-jev-resumption.md); exact requests and results retained. | No general 10× problem-solving result, blind discovery study, reliable arithmetic oracle, or permission to trust the top result. Compound questions can lose necessary sources. |
| A bounded broad capture can make several subsequent dependency questions offline work. | [One live payload](2026-09-21-memory-payload.md): 796.301 MiB in 1,617 ms, 166 selected ranges, 77,156 matching anchor bytes, unchanged observed scenario/search projections. | Not atomic, portable, or proven sufficient for complete replay; timing is one local run. |
| The first use of that packet moved the obstacle from missing game data to a runtime service. | [Offline frontier](2026-09-21-payload-frontier.md): nine data boundaries resolved with 36,256 added bytes; reproducible refusal at a four-byte CRT malloc request after 382 instructions. | No full A* return, native output equivalence, complete FXSAVE import, or modeled allocator. No main fidelity score moved. |

Jev's cumulative estimated cost was approximately $0.059800 against the $1 lab
ceiling. That is historical reported usage multiplied by the experiment's price,
not an invoice or a current pricing claim. The follow-on capture/replay work used
no further API requests. Jev helped navigate existing evidence; executable probes
produced the new runtime findings.

## Review charter

The useful steering question is whether any of these methods would shorten a
real investigation on today's main branch. The headline, call coverage and
current blockers may have changed since the lab branched. Fable should feel free
to choose a different pilot, reject the premise, or identify a cheaper existing
tool; the experiments below are evidence to assess, not a checklist to ratify.

My recommendation is to consider one opt-in acquisition/offline-inspection pilot
where repeated missing inputs are currently expensive, keep Jev as an optional
source finder, and keep further emulation as a separate research bet. Adoption
should not wait for a complete Windows runtime model, nor silently commit main
to building one. A finding that changes the investigation strategy can be useful
even when none of this implementation is adopted.

## Concerns that can be considered separately

| Concern | Files / commits | Adoption boundary |
|---|---|---|
| Semantic source discovery | `tools/explore/jev*.py`, query example and offline tests; `30b5bae`, `f616bfc`, retrieval report in `7de8aad`. | Explicit selected files are sent to an external API on cache misses. It uses a pinned model, exact-request cache and a local budget ledger, not an account-wide spending cap. No default main-loop integration is proposed. |
| Strict local replay inputs and observer compatibility | `thread_context_probe.py`, `coord_table_probe.py`, `register_image_stub.h`, restore/context probes and tests; `7de8aad` through `732bc11`. | Native code/data come from the owned install and externally retained packets. Existing byte guards and reset controls remain essential. These commits have dependencies and are not independent cherry-pick promises. |
| Range inventory and bounded streaming payload | Inventory/payload writers, readers and authored fixtures; `20cf7ee`, `c8ad5fd`, `5de0555`; live findings in `4a38663`. | Explicit experimental defines only. Requires the restore/context/graph probes and shared capture coordination. Reports must preserve exclusions, failures and non-atomicity. |
| Shared import-library names | **`1bf7bcf` only**, `tools/trace/kernel32.def`: VirtualQueryEx, GetSystemInfo, GetTickCount. | The only changed path outside `docs/lab/` and `tools/explore/`. Normal builds do not import these unused names. Keep this adoption decision separate from activating a capture probe. |
| Runtime-service replay | One-off source and observations under the external experiment directory below. | Not a reusable allocator implementation or adoption candidate yet. Further work needs explicit ABI, allocation/uninitialized-byte tracking, extended-state validation and native output comparisons. |

Original executables, captured memory, settings backups and decompiler output
are not in this diff. Tracked compressed Jev evidence contains authored project
prose/source, requests/responses and numerical observations. The API credential
is not retained. The snapshot collector is generic over memory ranges; it is not
a schema claim about every selected allocation.

## Validation and integration status

The last experimental release gate, `/tmp/memory-payload-live-gate`, passed:
332 rondata, 855 sim, 13 fixed and three doc tests; 782 requested fixtures present,
clippy/format/data survey/paperwork clean. The focused review suite below passes
**48 tests with the install and retained capture supplied**, with no live game or
API access. The frozen Jev manifest/state hashes also revalidate offline.
Collector tests include failed/short reads and writes, changed ranges, deadlines,
cap edges, anchors and incomplete files; control/inventory/payload Windows builds
and the authored C sanitizer checks are documented in the payload report.

At packaging, fetched `origin/main` is
`02ac2a8c126a457523aa77762cfe967f4249ed3a`, 65 commits beyond the common ancestor
`c671bd0c19d9a97f1c8e50112528e6d6aa78b19c`. A non-checkout merge-tree check reports
no conflicts. **That is not an integrated gate.** Tests cited above ran on the
lab tree. Any selected implementation needs review and a gate against the main
tip on which it is actually adopted; this PR does not rewrite the main handoff.

For the local reviewer, set `RON_INSTALL` to the owned game directory and
`RON_RESUME_CAPTURE` to the retained payload capture, then run:

```sh
PYTHONPATH=tools/explore \
RON_CONTEXT_IMAGE="$RON_INSTALL/riseofnations.exe" \
RON_RESUME_INSTALL="$RON_INSTALL" RON_RESUME_CAPTURE="$RON_RESUME_CAPTURE" \
uv run --offline --with unicorn==2.1.4 python -m unittest \
  test_jev_evidence_lab test_thread_context_probe test_restore_prefix \
  test_restore_context test_memory_inventory test_memory_payload \
  test_allocation_selection test_resume_frontier
python3 tools/explore/jev_evidence_analyze.py
uv run --offline tools/explore/memory_payload.py "$RON_INSTALL" "$RON_RESUME_CAPTURE"
```

Capture-dependent tests skip when their environment is absent, so a passing
count without the supplied install/capture is not equivalent to the 48-test run.
These focused checks complement the repository release gate; they do not all
become automatic gate tests merely by existing in the lab.

On this machine the retained packet is
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-memory-payload`.
The one-off replay source and observations are under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-payload-frontier`.
Both have manifests. They are intentionally external dependencies, not files a
GitHub checkout alone can reproduce. The capture lane is free; neither reviewing
these artifacts nor investigating the next allocator model requires it.
