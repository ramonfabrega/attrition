# Leader encrypted-state follow-up

Score-neutral follow-up to [PR #7](https://github.com/ramonfabrega/attrition/pull/7),
on `codex/typed-leader-bridge` from `0cae789`. PR #7 remains the acquisition and
first scalar-bridge checkpoint. Recommendation: continue the offline pilot;
this does not establish whole-frame logger parity or propose a sim change.

## Finding

The same Great Lakes snapshot at logger frame 11186 contains the encrypted
leader records reached through the PDB-typed `data_encrypted` pointer. Four
logged leader identities (0, 1, 8, 9) yield twelve arrays: six resources, six
income entries, and **seven** cap entries per leader. All 76 values agree after
the getter transformations. The initial six-element cap assumption failed;
the promoted tool derives the array indices from decoded PDB fields and
requires exact printed cardinality.

Owned getter readings establish the XOR constants: resources at `0047da20`
uses `0x872`, income at `0046ee60` uses `0x90236`, and resource cap at
`0046ee80` uses `0x1281`, followed by signed 32-bit interpretation. The retained
snapshot/logger comparison backs the observed outputs; it does not establish
all possible getter inputs. The formulas are authored descriptions, with no
install-derived code or type exports committed.

`tools/explore/typed_leader_bridge.py` overlays the previous comparison,
retaining raw storage, transformed value, printed value, identity, array index,
address and PDB path for each newly compared occurrence. It verifies type-export
and payload hashes before decoding. Repeated keys are bridged only within the
explicit LEADERDATA scope, in their printed array order, with unique identity
and exact cardinality. Unknown keys remain unknown.

## Measurement and limits

The retained packet produces 8,301 matched occurrences (7,904 raw integers and
397 transformations), zero scoped mismatches, and zero leader-bridge errors.
The denominator remains 602,211: 578,528 unmapped, 15,181 unclassified text, and
201 ambiguous logger-ownership occurrences remain. This is incremental
coverage, not complete state agreement. The main coverage pin is unchanged.

The root pointer was acquisition-anchored; the referent was **not**. Agreement
with these 76 logged outputs cannot establish referent-wide coherence,
allocation liveness, or atomic snapshot semantics. No new capture was taken.
The next useful widening is the remaining leader arrays and their explicit
logger transformations, followed by container ownership; none needs a capture
until retained bytes or temporal evidence are missing.

## Verification

Nine focused tests pass across the leader and existing scalar bridges. Negative
controls include a corrupted encrypted word, null pointer, missing array index,
duplicate index, unreadable element, short printed array, duplicate leader
identity, and absent logger scope. Signed projection and the seventh cap entry
are positive controls. Full release validation is pending on this follow-up.

Evidence is retained outside Git beside the original packet, under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-typed-state-market/followup/`.
The original packet and type export paths are recorded in the
[typed-state review](TYPED-STATE-REVIEW.md). The CLI takes state JSON, previous
comparison JSON, bound types JSON, and snapshot binary, in that order.
