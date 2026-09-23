# Market item 327: state-side head-to-head

**The retained packet answers the endpoint-state question; it does not by
itself answer the offer's cause.** No capture, simulation change or score
movement. This is independent of PR #8's function-side experiment; its branch,
worktree and results were not read or changed for this measurement.

## Result and falsifiers

The packet is at logger frame **11186**, after trace tick **11185**. Player 1's
PDB-declared `make_list` has length/capacity 11 and a 40-byte element layout.
All eleven elements were decoded through that declared pointer, including
empty slots; none was discarded because its type sentinel was negative.

| Question | Decoded result | What would falsify it |
| --- | --- | --- |
| Is `MAKE[1]` an empty Merchant-shaped remainder? | `t=-1`, `val=909090`, `cat=4`, `escrow=1`, `city=0`, `up=0`, `o=-1`, `num=1`, coordinates zero. All ten fields match the logger. | Any field differs, declared extent is invalid, or identity/frame differs. A remainder's prior identity still needs the earlier frame. |
| Does the original have regional rare knowledge? | `known_rares=4`; `reg_known_rares[1]=4`, the other 63 entries zero. All 65 values match. | Any array element differs. This is original state, not proof of which function wrote it. |
| What is the resource state? | Buckets `[24,6,6,353,47,0]`; resources and income each `[1920,2400,992,1520,960,0]`; resource caps `[2992,2992,2992,15984,2992,2992,2992]`; support all zero. Wealth is index 2: bucket 6, income/resources 992. | Wrong getter mask, signedness, array shape or any logger mismatch. Do not confuse wealth index 2 with metal index 4, whose income is 960. |
| Does the packet retain the re-offer value 227272? | Not as a current make-list offer. Slot 4 still contains `t=61,val=909090`; slots 0 and 7 have already become `t=-1,val=3945568,cat=7`. | A complete slot decode finds 227272. It did not. Residual bytes elsewhere would not establish an active offer. |

**206/206 fields agree with both the capture logger and run123 directly**: 110 make-list fields, 65 rare counters, 31 encrypted
array values. This is a targeted experiment, not another expansion of the
published 516,369-occurrence coverage checkpoint. Make coordinates are zero
in this observation; this does not validate a general nonzero projection.
PDB export and packet hashes were checked against the retained state manifest.
Selected root stability does not establish atomicity for the make-list and
encrypted-state referents; the logger agreement is the scoped observation.

Controls change only a read wrapper, never the packet: one byte at the empty
slot's `t` changes -1 to -2, at the regional counter changes 4 to 5, and at
wealth's encrypted word changes its decoded value. Each changes the targeted
comparison. A negative container length is refused. Separate comparison-value
controls produce exactly one mismatch each for those three fields.

## The boundary matters

The already-retained six-frame logger window supplies the history absent from
one snapshot. These are **logger frame numbers**, not trace ticks:

| Frame | Slot 0 `(t,val)` | Slot 1 `(t,val)` | Interpretation limited to observed state |
| --- | --- | --- | --- |
| 11183 | `(-1,16200)` | `(61,909090)` | Merchant present before re-offer. |
| 11184 | `(61,227272)` | `(-1,909090)` | New Merchant offer and emptied old slot coexist. |
| 11185 | `(430,3945568)` | `(-1,909090)` | Siege Factory occupies head. |
| 11186 | `(-1,3945568)` | `(-1,909090)` | Packet endpoint, after trace tick 11185. |

Thus a native call replayed from this packet begins **after** the decision
under investigation. It can test a function on those inputs; agreement alone
cannot reconstruct the preceding decision's inputs. A temporal claim needs a
pre-decision state or an independently justified reconstruction. This is a
boundary limitation, not a reason to discard either instrument.

The packet does not settle the writer, offer formula, or slot-emptying code
path owed by ECONOMY §14.4. It corroborates their consequences. No new static
reading is presented as an independently ratified rule.

## Cost and recommendation

Request received at 02:16:32 UTC on September 23 (September 22 locally).
Consolidation finished at 02:20:46; first decoded numbers were written at
02:24:27: **7m55s request-to-number, 3m41s focused question-to-number**.
The actual targeted decode command took **5.54 seconds**, using the existing
bound type export and decoded root manifest. Validation followed separately.
This is a warm-artifact measurement and excludes building the oracle tools.

For context, the packet's original build/run/restore cost **221.19 seconds**;
its copy cost 249 ms. Item 520 records run123's broader capture at about
**45 minutes and 1.38 GB**, followed by widening. These are different workloads,
not a controlled speedup ratio. With run123 already on disk, querying its log
is also cheap and answers all 206 fields here. Memory's advantage on this
question is typed access and compatibility with function replay, not exclusive
access to these values. No new capture is justified for these endpoint facts.

**Pilot** the typed packet as an additional instrument. Step 4 now has a
bounded, partial result: endpoint state answered; causal replay still needs
its temporal contract. The packet failed the stronger test of answering the
entire Merchant offer mechanism by itself. Keep coverage expansion stopped
for this checkpoint and let steering compare this result with PR #8.

Evidence lives outside Git under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-typed-state-market/followup/market-head-to-head/`:
probe/check/control scripts, address-bearing decoded rows, all 206 comparisons,
six-frame trajectory, and mutation controls. The packet remains read-only.
