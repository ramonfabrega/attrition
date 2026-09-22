# Prepared: two packets for the natural next restore call

L79 prepares the missing native boundary identified by
[L76](2026-09-22-continuation-boundary.md), after the offline continuation and
recycler work in [L77](2026-09-22-absent-search-state.md) and
[L78](2026-09-22-recycler-acquisition-epochs.md). No game was launched and no new
capture is claimed. The collector is opt-in and awaits a fresh lane slot.

## Question and selection

Does the bounded model reproduce the actual next restore invocation for the same
registry-resolved unit when given that invocation's own inputs and world snapshot?
This would be an isolated native-call comparison. It would **not** prove that our
model reproduces the intervening frame or other units' searches.

`RON_RESTORE_SECOND` retains the established first call: complete pre-call graph,
prefix, context, inventory and payload, followed by the limit-95 intervention and
whole post-unit/path witness. The limit is restored before post observation,
including its failure paths. Only successful post observation with outer return
−1 and all five saved containers present arms the second selection.

The selector ignores subsequent restore entries for other owner/id pairs. On the
next matching entry it resolves the registry again and requires the same unit
address and a nondecreasing frame. Existing context and post-state checks verify
the unit's owner/id fields. A changed address, malformed identity, unexpected
active entry or collector failure closes selection. The original frame bound
0–1400 remains; there is no unbounded retry. After the second packet, all further
entries are ignored. Identity here is the observed owner/id/address tuple, not a
new object-generation proof against same-address ABA reuse.

The second call gets a fresh complete pre-call packet and whole post-unit/path
witness. It receives **no intervention**. The delegated limit/saving words must be
300/1 after payload acquisition and at return. If they differ, observation refuses;
it does not overwrite a value the second call or another actor changed.

`RON_RESTORE_SECOND` requires `RON_RESTORE_LIMIT95` and `RON_RESTORE_POSTSTATE`.
It rejects `RON_RESTORE_POSTGRAPH`: that collector is scoped to five present
saved containers, whereas successful completion may leave all five absent.
Neither its schema nor the comparison rules are weakened for this experiment.

## Two packets, one original trace

First packet filenames remain unchanged. Second files carry `second-` prefixes:
search graph, restore prefix/context, memory inventory/payload and restore
post-state. The log path is untouched. Storage is reused only after the first
packet has returned and been written, so a second failure preserves its files.
The first post-state retains version 2 and its limit-intervention trailer; the
unchanged second call uses the existing version-1 post-state format.

INFO 190 begins a packet with index, unit, owner and id; INFO 191 ends it with
index, unit and outer return. Both include the trace's frame. INFO 192 refuses the
sequence. These delimiters bind packet identity; existing graph/context/payload
and post-state receipts remain in force. The first packet must suspend and both
must finish before the paired reader accepts a result.

`second_restore_packet.py` validates both prefix/post witnesses and delimiters
before projecting either packet into a fresh output directory. It removes only
the other packet's collector receipts from the derived trace. All native call,
return, census and startup events are retained, allowing the existing strict
readers to run without duplicate packet receipts. It records the full original
trace hash, selected record interval, file hashes and validated payload hash.
The original capture is unchanged. Output failure leaves an explicit
`.incomplete` marker; a destination is never overwritten. A changed post-state
during projection is rejected. Projection itself copies the selected payload,
so allow additional disk space beyond the two source packets.

Full validation still checks image/tracer identity, context, graph, inventory,
payload framing, mappings and anchors. Projection is a transparent derived view,
not an excuse to select a favorable return or omit an intervening call.

## Offline checks and proposed run

The authored integration harness executes the actual entry, delegation, limit,
return, selection and filename code with authored memory and file services.
Context/payload collection is mocked at that boundary and remains covered by its
existing independent failure tests; this is not a native ABI or timing test.
Controls exhaust every callback read on both packets, prefix/output failures,
limit restoration before post-read failures, unrelated-unit filtering, changed
registry address, frame rollback, unexpected entries, duplicate delegation/return,
missing suspended roots, incomplete payload, mode drift and ignored third calls.
The two C-produced post packets decode through the existing Python formats.
Reader controls reject missing/reordered/duplicate delimiters, orphan/failure
receipts, mismatched packet data and post-state changes during projection.

The affected suite runs 49 tests with the owned executable supplied, without
skipping the two emulator-backed checks. Default tracer syntax passes; the two
incompatible option combinations fail with their intended diagnostics. The
opt-in build compiles and links as COFF i386 in a private staging directory, and
an instrumented executable copy is prepared there. Neither the shared install
nor the profile was modified. Source image SHA-256 remains
`30478a44b577cb11ebcbbbf53d3e93ba02fd2aacf3bdefa6552c9b6449625079`.

The proposed capture uses the same map 14, seed 12345, authored congestion
scenario and frame-1400 quit, with a 180-second timeout. Prior runs lasted about
30 seconds; a second broad payload adds acquisition and disk cost. Each payload
retains the existing 1 GiB/5-second caps, range checks and anchor checks. Those
deadlines are checked between OS calls, rather than cancelling an in-flight
call. These checks do not make the broad snapshot atomic. The scenario does not substitute
an immediate second invocation for natural scheduling.

After a granted slot: run the standard unattended harness, verify restoration of
all five settings files and release the lane before analysis. Bind the actual
images, validate both packet envelopes, then project and replay the second with
its own captured inputs. Borrowed extents must be derived from that new packet;
old capture addresses must not be reused. Compare outer return, every unit byte
and every path-capacity byte. A missing packet, undefined dependency or mismatch
remains a finding. FXSAVE import and modeled intervening world transitions remain
separate limitations. No main score changes from preparing this collector.

Reproduction artifacts, prepared command and staged build hashes live under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-second-restore-collector`.
PR #6 stays frozen; this work remains on `codex/continuation-lab`. The full
release gate passed: 1,203 tests, 782 fixture requests with none missing, plus
clippy, formatting, install survey and paperwork checks. Peak process-tree memory
was 8,049 MiB. The final intervening-call preservation control also passed all
seven packet-reader tests. No lane slot is booked yet.
