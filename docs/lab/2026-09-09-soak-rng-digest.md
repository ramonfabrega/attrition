# The soak detects RNG-only divergence

The soak's selected-state digest omitted `Sim::rng.seed`. Two states could
therefore agree on all digested visible fields while already predicting
different random outcomes. Ledger L23 identified that gap.

The digest now consumes the complete unsigned gameplay seed, widened without
sign loss into the existing integer digest input. No draw is consumed by
hashing it. This changes only the test digest: simulation arithmetic, gameplay
RNG advancement, generated input orders, and fidelity floors are unchanged.
The scenario/order generator has its own RNG; it is not simulation state and
is not folded into this digest.

A new regression was first run against the old digest. It failed at seed bit
0: changing only that bit left the digest unchanged. After the implementation,
it checks all 32 single-bit seed changes, checks that an extra gameplay draw
changes the digest before any tick or order, and checks that restoring the seed
restores the original digest. Only the gameplay seed is mutated in that test.
The negative log is `/tmp/soak-rng-negative.log`; the passing full soak-module
run is `/tmp/soak-rng-positive.log` (five tests).

The digest documentation now says selected gameplay state rather than implying
exhaustive state coverage. Other omitted state and finite-hash collisions remain
possible; the soak is a determinism/termination check, not a fidelity oracle or
a proof of complete state equality. This closes the identified RNG omission,
not all possible digest gaps. No simulation score moved.
