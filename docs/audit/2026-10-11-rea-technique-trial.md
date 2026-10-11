# REA technique trial — reuse evidence before adding an engine

Direct, single-worker assessment on `codex/core-loop-investigation`, starting
at `9bea3f1d`. User authorized reading/adapting REA, followed by a runtime
comparison only if the integration looked promising. Commander remains paused.
This is a tooling assessment, not a completed queue mechanic or independent
review. No simulation, comparison logic, pins, fixtures or shared install changed.

## Source review and adoption decision

Reviewed upstream commit `d802304c542b20bc86e22cc492718e89a8a5579b`, downloaded
as source into ignored `target/rea-assessment/`; no package scripts executed,
MCP registration, skill installation, new engine, or Ghidra process started.
The source archive and extracted tree occupy approximately 44 MB with the trial.
Disk reported 68 GiB available before this work. Upstream main is the reviewed
version; this does not establish the behavior of a different npm release.

| Technique | Existing equivalent / gap | Decision |
|---|---|---|
| Function dossier | Named/PDB-typed exports, index, objdump and caller scripts already exist; gathering their results is manual | Trial a small question-specific bundle, retaining full evidence locally |
| Artifact-bound evidence | Journals retain paths and commands; individual investigation slices do not always carry byte identity | Add exact input/output hashes to this trial; hashes alone do not prove export provenance |
| Observation/inference/unknown separation | Specifications, hypothesis killers and review debt already require this | Apply explicitly to the ammo question; no additional ledger framework |
| Snapshot cache | Static exports already amortize analysis; there is no need to re-decompile them per question | Keep exports; do not build another persistent cache without a measured recurring cost |
| Ghidra bridge | Our saved project includes PDB typing and project corrections | Defer live REA comparison: reviewed provider does not reopen existing projects |

Implementation sources, pinned for reproducibility:

- [Function bundle](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/bridge/ghidra/ReaGhidraBridge.java):
  `analyzeFunctionObservation` collects pseudocode, assembly, calls, references,
  strings, basic blocks and value-flow information. It explicitly excludes
  unresolved indirect flows from reference edges. Complete evidence can be large.
- [Snapshot cache](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/src/application/binary/AnalysisSnapshotCache.ts)
  and [identity schema](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/src/domain/analysisSnapshot.ts):
  target/profile/query binding, exclusion of live/mutating/cursor-dependent
  operations, bounded entry count, and explicit cached-provenance limitations.
- [Launcher](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/src/ghidra/GhidraLauncher.ts)
  constructs `-import`, `-readOnly`, `-deleteProject`; the
  [installation contract](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/docs/installation.md)
  explicitly says it never opens an existing user project. Queries wait for
  default auto-analysis. This establishes lack of project reuse, not inability
  to load a PDB by any means, nor a measured runtime cost on our executable.

A runtime trial remains useful if existing-project support becomes available,
or if a bounded question requires provider features our exports lack. Installing
Hopper or Binary Ninja would test an additional variable and is not needed to
adopt this technique. No performance or fidelity claim about those engines is made.

## Local experiment

One-off `target/rea-assessment/bundle_trial.py` reads the existing export index,
four exact named functions and the installed executable; runs the existing
LLVM objdump on their index-bounded address ranges; retains full text plus a
manifest of hashes, commands, direct call sites and explicit limitations.
It copies only the four small reading slices, not the corpus or project.
No REA source was copied into the prototype. The prototype stays scratch rather
than becoming a supported tool before repeated use earns that maintenance cost.

Functions: the already-solved caravan predicate as a calibration, plus the
three-function ammo path named by the current coverage word. The calibration
checks the predicate only, not a blind re-solve of the whole caravan mechanic.

| measurement | result |
|---|---|
| Four-function collection, three consecutive runs | 0.1923 / 0.0734 / 0.0742 s |
| Complete first bundle, including manifest | 143,427 bytes |
| Question summary with evidence pointers | 2,366 bytes |
| Repeated output | All four function records and input identities equal |
| Executable-byte checks | Four direct call sites and return addresses agree |
| Known-answer control | Caravan flag read at offset 0x2b8, mask 8 |

These are local collection times, not REA timings, saved agent time, token counts,
or a controlled cold-cache benchmark. The summary is about 1.65% of the full
bundle's bytes but contains less information; it is an entry point, never a
replacement for evidence needed to decide a claim. Peak RSS was not measured:
macOS rejected `/usr/bin/time -l`'s sysctl access. No memory benefit is claimed.

Important limitation: the next indexed entry bounds disassembly but is not a
formal function-body boundary. Literal direct calls are identified; indirect
calls, tail flows and complete caller discovery are not supplied. Source hashes
identify the observed export and binary separately; they do not establish that
the former was generated from the latter. There is no persistent cache to trust.

## Application to the next question

The retained trace's two game-RNG return offsets in ammo initialization match
original-byte calls at `0x0067c8c4` and `0x0067c8f6`, returning at +0xcd9 and
+0xd0b. The enclosing calls return at `Objects::add_ammo+0x119` and
`GraphicEvents::execute_game_events+0x40d`. All four edges were also checked
using narrow, raw-byte objdump queries, independently of the bundle parser.

Observation: the runtime trace reaches initialization through the release path.
Unknown: which actor/target fired, and whether our sim omits the release or
initializes it differently. Static caller evidence does not identify that actor.
Existing sim code already implements animation-crossing and target/pivot gates;
reading those gates is not a new fidelity discovery. Do not patch a gate from
this bundle alone.

Next experiment for item 1648: enumerate the sim's releases at frame 3439 and
compare whole-cast animation/order state in the retained run721 window. A
matching release moves the question downstream into initialization; an absent
release requires tracing the first differing decision/state. Identify that
before selecting a shooter or requesting another capture. The summary retains
these alternatives and falsifiers rather than embedding one favored cause.

Score/value movement: **none**. Coverage word remains **3439**, **23 vs 25**
draws; run721 remains **951 keys / 146 standing** from the prior validated
landing. No replay was rerun and no new value equality is claimed.

## Evidence, reproduction and validation

All generated proprietary evidence stays ignored in `target/rea-assessment/`:
`bundle_trial.py`, `verify_trial.py`, `trial1` through `trial3`, `question.json`,
`results.json`, `call-*.txt`, `verification.log`, and the pinned upstream source.
Original trace: `target/item-1647-investigation/original-draws3439.txt`.

Reproduce collection with `python3 target/rea-assessment/bundle_trial.py NAME`,
using a new, unused NAME (existing paths are refused). Validate retained trials:
`python3 target/rea-assessment/verify_trial.py`. It rechecks hashes and retained
bytes, equality across runs, the four original-byte edges and the control.
Verification passed. This is not a differential sim test or independent audit.

Documentation-only tracked change: `tools/guard.sh` and `git diff --check`
both exited 0; all five pinned source links resolve in the downloaded revision. Full release gate is unnecessary because no executable
project code or comparison coverage changes. Source-review conclusions remain
provisional pending later independent review; the executable checks establish
only the specific bytes/edges stated above. Commander queue remains intact.
