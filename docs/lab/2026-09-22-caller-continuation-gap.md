# L82: isolate the caller transition between two native restore calls

L81 proved that the second captured input reproduces its native output. This
experiment asks a different question: can the first model's live state produce
that same output without importing the second snapshot's world?

## Measured boundary and first control

Both inputs come from the retained L81 pair under
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-second-return-modes`.
The whole first native post-unit and second native pre-unit differ at one byte:
+0x88 changes 2 to 3. The PDB `UnitData` record identifies +0x88 as the **short**
`collide`; +0x8a is the adjacent `collide_o`. L81's aligned-word comparison was
an observation window, not a claim that +0x88 was a 32-bit field.

All delegated registers and six callee arguments agree. One saved caller-stack
word differs, at index 6 of the captured delegation stack, from 1 to 2. The
experiments retain the first caller tail and stop at the established callee
return boundary, before the wrapper epilogue. They do not claim that resuming
the caller with its old saved registers would reproduce subsequent execution.

The initial ablation prepares the first packet's declared memory, verifies the
modeled limit-95 result against its native witness, and chains a limit-300 call
without resetting heap/global state or cumulative lifetime budgets. The frozen
chain agrees with the second native return, modes and every path slot; its only
unit discrepancy outside explicit path-pointer relocation is +0x88. A diagnostic
trial that copies only the observed second-input word removes that discrepancy.
That trial is an input intervention, not a simulated world transition; it is
retained separately from the original-instruction experiment below.

## Original caller instruction

Both recorded wrapper return addresses are `0x5f7e29`, inside
`Unit::do_move@005f7b30`. The owned executable's listing places a 16-bit increment
of the unit field at `0x5f7e05`, immediately before the order update, argument
pushes and call at `0x5f7e24`. The type record establishes the field name; the
listing establishes the exact instruction and width. No original code bytes
or decompiler bodies are copied into the repository.

`caller_collide.py` loads that instruction from the hash-pinned owned image and
executes it with only code and a complete 344-byte unit region declared. It
requires one instruction, exactly one two-byte store at +0x88, and no changes
to any other unit byte. Controls cover raw 16-bit values across byte/sign/wrap
boundaries, fresh/reset equivalence and invalid input extents/addresses.

`native_chain_gap.py` validates the source pair and both projections, including
byte hashes against their source artifacts. It requires the recorded caller,
equal delegated registers/arguments, and a native unit delta confined to collide.
It derives three logical borrowed extents from this first packet's typed
headers, rather than trusting addresses from another run. These are model
extents, not claims about the native allocator's ownership or physical sizes.

The final experiment compares frozen re-entry with re-entry after that original
instruction computes the collide update from the first modeled post-unit. It
copies only the computed two bytes into the preserved chain. The second snapshot
is used to validate the expected input transition and as an output comparison
target; its value is not the source of the computed update. Both chains are
repeated and baseline restoration is checked. The original instruction computes 2 to 3 from the first modeled post-unit.
Its chain matches native return 8, modes 300/0, path length 8/capacity 40, all
640 path bytes and all 344 unit bytes except the explicit path-pointer
relocation. The frozen control retains exactly the extra +0x88 discrepancy.
Both chains repeat exactly, and the prepared baseline restores.

L84 subsequently [tested whether collide affects this callee](2026-09-22-collide-passthrough.md):
it does not read or write the field in nine repeated interventions. The
correction closes a carried-through unit-record difference, not a pathfinding
computation difference; that distinction limits the causal interpretation here.

Each chained second call executes 33,494 emulated instructions; L81's
independent second-snapshot replay executes 34,845. Thus the checked output
agreement does not establish identical internal execution.
L85 [localizes that trace gap](2026-09-22-continuation-cleanup-gap.md) to active
tree cleanup and recycler availability: four diagnostic entry-word substitutions
match the instruction stream, without proving equal heap or world state. The first input's
42 dependency additions supply 166,492 captured bytes; its three borrowed
logical extents total 672 bytes. The preparation counter was 98.07 seconds. L83 checked its timing scope:
that counter stops before the chained experiment callback, so it is not the
whole experiment duration; total wall time was not retained.

## Provenance and limits

The output-only comparator is now explicit as `compare_boundary`. The ordinary
`compare` still rejects a foreign payload hash or intervention provenance before
calling it. Cross-input chain reports retain both the first-input payload hash
and the second-native target hash and label their narrower comparison scope.
Tests verify that the new helper does not weaken ordinary same-input binding.
The frozen-gap assertion rejects any additional unit byte, mode, return, header
or inactive path-slot disagreement.

This is one selected pathfinder boundary. It does not implement the movement
caller, scheduler, intervening A* call, shared-world updates, wrapper epilogue,
native allocation reuse or extended CPU-state import. A sufficient input change
for one observed output is not proof that the rest of the world stayed fixed.
No capture was requested or run, no main score moved, and PR #6 remains frozen.

The affected suite passes 56 tests with the owned image supplied. The full
release gate passes 1,203 tests, 782 fixture requests with none missing, clippy,
formatting, the install survey and paperwork checks. Artifacts, reproduction
commands, initial ablation and final chain results live under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-caller-continuation-gap`.
