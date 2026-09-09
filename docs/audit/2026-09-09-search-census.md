# Census before a suspended-search graph capture

## What the runs establish

The retained corpus contains no A* suspension return: 87 finalized trace files
were accepted by the streaming census, with 82 paired A* returns in six files.
These are narrow proxy windows, not all searches executed by those games.
`/tmp/search-census-archives-final.json` retains the per-file results.

The new minimal census lane observes only A* call/return proxies, avoiding the
per-node cost proxies. A 36-frame command-probe run produces three successful
searches and no suspension. It matches the retained control on all 19 logged
frame bodies and 37 frame/RNG records. This validates the exercised hook path;
it cannot validate metadata reads which no suspension reached.

A scripted run through frame 8,000 produces 139 successful searches: 47 at the
unit-grid step, 11 at the intermediate step and 81 at the world-grid step.
There are no failures or suspensions. Every frame/RNG record from 0 through
8,000 is present. The first 18 detailed frame bodies and first 37 frame/RNG
records match the control. Later gameplay has no matching control in this
tranche, so this is a coverage census, not an 8,000-frame fidelity claim.
No parity score moved, and no natural suspended graph was captured.

## The instrument and its limits

`RON_SEARCH_CENSUS` uses the existing checked A* proxy with coverage disabled.
It rejects capsule/suppression combinations and refuses unsupported image bases
or coverage mode. The default preprocessed tracer tokens remain identical.
Only an A* return of −1 reads metadata: saved container sizes/roots and seven
recycler headers, through exact-length `ReadProcessMemory` calls. Up to 64
events are admitted; the cap and any read failure are explicit records. It
neither walks nodes nor modifies game memory. Layout evidence and the authored
protocol are in `docs/FORMATS.md`, “Suspended-search census metadata”.

`search_census.py` reads traces in 128 KiB chunks and pairs A* calls/returns.
Experimental mode requires the minimal proxy receipt, census version and full
per-event metadata completion in an observed frame. It rejects truncated
records, bad headers, trace-health failures, missing slots, read failures,
orphan returns and a missing cap receipt. A run with no gameplay frames is
reported separately from one that observed frames but no suspension. Archive
mode explicitly admits legacy version 1 headers; version 2 changed the
coverage-prologue table, not these proxy records.

The host C fixture compiles the exact census callback used by the DLL and
checks a complete event, each of 15 reads failing or returning short, and the
64-event cap. Nine Python fixtures check framing, matching, missing receipts,
health failures, archive-version selection and capped output. None constitutes
a natural metadata witness. The collector is ready to measure candidate graph
sizes, not ready to claim a complete arbitrary-search capsule.

## A reproducible longer run

`live_session.py stage --end-frame 8000 --fast-forward` now schedules the
original's `ffwd 9` at frame 37 and quit at 8,000. Detailed state logging remains
frames 18–35; the ordinary 36-frame default is unchanged. Seven staging tests
cover restoration, rollback and invalid long-run options before settings change.
The command receipts and final frame were checked in the successful live run.
The documented `ffwd` mechanism is in `docs/ORACLE.md`, “ffwd is the lever the
combat run wanted”; it suppresses the wall-clock wait and most rendering.

An earlier normal-speed long attempt was abandoned after interactive keyboard
input opened chat and paused the game. It was closed by its exact test-process
PID and its settings restored. Its data is exploratory and is not used for the
results above. The successful long run was restarted from the same backed-up
profile with scripted commands. The lobby window also needed repositioning to
the existing driver's measured location; successful input was checked by the
resulting logged frames, not by the driver's click receipts alone.

## Artifacts and next experiment

The short capture is `/tmp/attrition-search-census`, compared with
`/tmp/attrition-capsule-control`; `/tmp/search-census-control.json` records the
comparison. The successful longer capture and report are
`/tmp/attrition-search-census-fast` and its `census-report.json`. Source and DLL
identities are bound by each directory's `capsule-image.json`. The abandoned
attempt remains separately at `/tmp/attrition-search-census-long`.
All original-derived data stays outside git.

The next experiment should deliberately exercise congestion or difficult
obstacles and use the census to choose a graph-capture bound. This seed's
passive progression through 8,000 frames reaches only successful searches;
extending its duration alone is not evidence-based coverage targeting. Neither
synthetic cleanup success nor a zero-event live census establishes resumption.

Validation: full release gate passes (269 rondata, one ignored; 822 sim;
13 fixed; three doctests), rondata 231.67 seconds, process-tree peak 9,064 MiB
under 20 GiB. Clippy with warnings denied, formatting, documentation guards and
the install survey pass. Default and census tracer builds deny warnings; the
default preprocessed tokens match the prior commit. Gate log:
`/tmp/search-census-release.log`.


Follow-up: `docs/audit/2026-09-09-congestion-probe.md` completes the targeted
scenario experiment above. Two launches each reach 1,787 natural suspensions,
with matching observed projections and 64 live metadata events. The negative
passive census remains as measured; the metadata-reading branch is no longer
host-fixture-only. Complete graph capture and resumption remain open.
