# Clock advancement is not gameplay activity

## Failing premise

The soak counted distinct replay digests to ensure generated games were active.
That digest already included the frame counter, even before the recent RNG
addition. A world with no gameplay changes could satisfy the activity threshold
merely by advancing time. Including RNG made a second bookkeeping-only source
available; it did not create the original gap.

A new regression was first run with activity still wired to the old replay
digest. It held the generated world fixed, incremented only the frame and RNG
for 400 iterations, and observed 401 distinct supposedly active states rather
than one. The test failed as intended. Evidence:
`/tmp/soak-activity-negative.log`.

## Separate signals from one traversal

Each sample now retains the selected gameplay-state digest before adding the
clock and RNG, then extends the same hash with those two fields for replay
equality. The world is traversed once. The activity check counts the first
signal; frame-by-frame replay comparison uses the second. The existing
per-scenario and aggregate activity thresholds are unchanged.

The regression now establishes that clock-only and clock-plus-RNG advancement
leave one activity state, that the actual activity guard rejects that frozen
series, and that replay still notices clock changes. The preceding RNG-only
regression remains in place. The existing moved/wounded-unit test now checks
both signals, including restoration after moving the unit back. All six soak
module tests pass; `/tmp/soak-activity-positive.log` retains the result.

No gameplay rules, tick behavior, generated scenarios, or RNG consumption
change. The private test digest values change; they are not persisted oracle
hashes or phase scores. No simulation score moves.

## Limits

This prevents clock/RNG-only progress from masquerading as activity. It does
not prove useful gameplay: other selected fields, including orders and idle
counters, may change without the mechanic of interest running. The activity
signal still hashes selected state, not every simulation field, and finite
hashes can collide. Mechanic coverage and fidelity need their own assertions;
the soak remains a generated determinism/termination check. The runtime
experiment remains paused.
