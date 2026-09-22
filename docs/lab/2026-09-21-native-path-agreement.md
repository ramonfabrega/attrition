# Native and modeled pathfinding agree at the captured return

One newly authorized capture closes L65/L66's missing output comparison.
Replaying that exact pre-call packet returns the same outer value and produces
**all 640 native path-buffer bytes**, including inactive slots. The complete
**344-byte unit record** agrees except for its explicitly relocated four-byte
path pointer. This is a measured single-call unit/path boundary, not a general
Windows runtime or whole-process fidelity result. No main-loop score changed.

## Acquisition and identity

The user relayed Fable's authorization for one capture. The prepared map-14,
seed-12345 scenario ran to closing frame 1401 in **21.67 seconds**, exited zero,
and restored all five settings files. The unattended runner released its lock;
the user was notified that the lane was free before offline replay began.
There were no retries or additional live runs.

The retained directory is
`/Users/rf-studio/ron-data/lab-captures/2026-09-21-native-poststate/map-14`.
Its image binding, prefix/context/graph, inventory, payload, post-state framing
and receipts validate. Frame 224's payload contains **838,914,048 bytes** across
171 ranges, acquired in **1,449 ms**; all 77,156 anchor bytes agree. Acquisition
is still non-atomic. Payload SHA-256:
`771fee1449962e9c96c45715c1963174493ecfd3880f2beed0d3495c210806eb`.

The 1,268-byte post-state packet contains the complete unit and forty path
slots. Its native outer return is **8**, path length **8**, capacity **40**.
The whole path-buffer SHA-256 is
`8cb0ad70e5777ca9ac07bddf849a9807ab84275419fc4b7147903d635902fea8`.
This also matches the earlier packet's modeled path, but the decisive comparison
below uses this new packet's own pre-call state.

The prior and new gamelogs have identical lengths and differ on only three
preamble lines: two loading-sync numeric values and `Sys Memory`. Every other
line agrees. This is evidence about the logged scenario, not all unlogged state
or a claim that observation has no timing effect.

## Paired replay and complete output comparison

The new capture has different addresses. Three logical borrowed objects were
re-established from its captured headers, checked for bounded captured bytes,
and supplied explicitly: the unit's ten-slot path array (160 bytes), and two
sixty-four-pointer recycler arrays (256 bytes each). No old absolute pointer
argument was reused. These remain logical extents, not decoded physical chunks.

The same opt-in malloc/memset/memcpy/free policies and six writable callee
argument words complete in **197,650 guest instructions**. The attempt performs
113 allocations, three fills, six copies and six frees, with 44,444 writes.
Forty-two dependency additions supply 165,784 captured bytes; the borrowed
objects add another 416 beyond the narrow baseline. The discovery and eight
same-instance reset checks together took 103.27 seconds on this run; this is
not a benchmark or the cost of one already-prepared call.

| Observation | Native | Modeled | Comparison |
|---|---:|---:|---|
| Internal A* return | 1 | 1 | Equal |
| Outer `find_upath` return | 8 | 8 | Equal |
| Active path length | 8 | 8 | Equal |
| Path capacity | 40 | 40 | Equal |
| Entire path buffer | 640 bytes | 640 bytes | Every byte equal; zero changed slots |
| Unit record | 344 bytes | 344 bytes | Only offsets `0xb8`–`0xbb` differ |

The differing unit field is its path allocation address: native `0x335d8780`,
modeled `0x1a9c1030`. All four differing bytes belong to that one field; **no
other unit offset is excluded**. Eight resets reproduce the model's complete
GPR, declared-memory, initialization, service-state and write fingerprints.
L65's earlier extended-state and relocated-arena controls remain scoped to that
earlier packet; no new extended-state control was needed to claim this measured
native output agreement. FXSAVE is still not imported.

## A check that can fail

The comparator now has `--require-agreement`. It exits nonzero if outer return,
path length/capacity, any unit byte outside the explicit path pointer, or any
active/inactive path byte differs. The JSON report is still emitted on a
comparison failure. It requires `--replay` and the matching payload hash.
Authored mutations independently fail the return, header, unit and slot checks;
a known path-pointer relocation alone passes. The native paired capture passes
this assertion. A copy of the real replay report with its last inactive byte
flipped fails the CLI with exit 1, names slot 39, and still reports active-path
agreement. The unmodified report passes.

```sh
uv run --offline tools/explore/restore_poststate.py "$INSTALL" "$CAPTURE" \
  --replay /tmp/native-paired-replay.json --require-agreement \
  > /tmp/native-paired-assertion.json
```

The fresh replay used `--borrow 0x33002b30:160`,
`--borrow 0x31a0f6f8:256`, and `--borrow 0x3333cd20:256`, with
`--services malloc+memset+memcpy+free --mutable-arguments --repeat-final 8
--observe-unit-path`. These arguments belong only to this capture.

Reports, checked object headers and the capture manifest remain external.
Paired replay/comparison/assertion results, the gamelog difference and validation
are retained under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-native-path-agreement`.
The acquisition was built from `cabc85d`; the only subsequent implementation
change adds the explicit agreement assertion. Nineteen focused comparator and
replay-helper tests pass. The full gate at `/tmp/native-path-agreement-gate`
passes 332 rondata, 855 sim, 13 fixed and three doc tests, with all 782 requested
fixtures present. Clippy, formatting, survey and paperwork checks pass. Peak
tree RSS is 8,450 MiB with two workers. Final note edits pass the paperwork
guard.

## What this earns, and what it does not

This demonstrates an offline native-function oracle for one captured pathfinding
call: bounded runtime models can reproduce a nontrivial native output rather
than merely get past imports. It supports a narrow pilot using this method for
pathfinding questions or controlled input experiments. Fable decides whether
that is useful to the main workflow; adoption and headline movement are not
claimed here.

Stop expanding allocator behavior without a failing case that needs it. The
next useful question is whether another deliberately chosen call or intervention
keeps this output agreement. Physical heap behavior, allocation failure/reuse,
all other mutated globals, exception paths, and general thread/extended-state
restoration remain outside the evidence. No further capture is currently booked.
