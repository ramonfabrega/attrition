# A complete modeled pathfinding call from retained memory

The independent follow-up on `codex/replay-allocator-lab` now reaches the
original `PathFinder::find_upath` return boundary. This advances L64's
allocator experiment without a live capture, TypeSafe request, simulation
change or main-loop score change. Draft PR #4 remains on its existing branch.
The experiment starts from `0320233`.
The subsequent paired native output check is [L67](2026-09-21-native-path-agreement.md);
it closes the unit/path comparison on a new pre/post packet.

## What returned, and what agrees

The retained September-21 memory-payload packet completes in **197,650 guest
instructions**, returning **EAX 8** at the original caller boundary `0x688faa`.
The internal A* return observed at `0x68335e` is **1**, matching the paired
native A* trace return. There is no native outer-return witness in this check.

The completed attempt performs 113 modeled allocations, three memory fills,
six copies and six frees, recording 44,444 writes. It adds 170,332 captured
bytes through 43 data-dependency expansions, plus 416 bytes for explicitly
declared objects. This is 170,748 additional captured bytes beyond the narrow
baseline; it excludes executable code and modeled memory, and is not a minimal
read-set measurement. Each failed expansion attempt is discarded and restarted.

The final path has length eight and capacity forty. The observer retains the
whole 344-byte unit record and **all forty 16-byte path slots**, including
inactive slots. The 640 path bytes have SHA-256
`8cb0ad70e5777ca9ac07bddf849a9807ab84275419fc4b7147903d635902fea8`.
These bytes remain outside git. They have **not been compared with native
post-call path bytes**; a matching A* scalar is not a path-fidelity result.

Eight resets of the same final runner reproduce the complete GPR, declared
non-code memory, initialization, model-state and write fingerprints. A fresh
process with perturbed XMM/x87 registers and control state reproduces the whole
reported final result. FXSAVE is still not imported: this is evidence about
this one execution, not a general extended-state solution.

Moving the modeled arena from `0x184e0000` to `0x19710000` preserves the return,
instruction count and all path-slot bytes. The complete unit record differs
only in two bytes within its relocated path pointer. Raw whole-memory and write
hashes differ, as expected; no normalized comparison of all mutated state is
claimed. Each alternate control also passes one same-instance reset.

## Explicit service contracts

The reusable `tools/explore/payload_replay.py` promotes the recurring external
probe. It validates the retained packet, pins the owned executable hash,
checks captured import targets against executable image ranges, and defaults
to no modeled runtime services. Missing data and unsupported services refuse.
All generated observations must be redirected outside git.

Three new authored service models build on L64's bounded malloc-success model:

- `modeled_memory_fill.py` implements a capped `memset` with guest stores.
  It uses the low byte of the fill value, returns the destination, and checks
  stack arguments, direction flag, return target and entry into the service.
- `modeled_memory_copy.py` implements a capped non-overlapping `memcpy`.
  Guest reads and writes retain initialization and exact byte-bound checks.
  Overlap, wrapping ranges, invalid entries and unset source bytes refuse.
- `modeled_lifetime.py` retires exact modeled allocations or explicitly
  declared captured objects. Null is a no-op; unknown, interior and double
  frees refuse. Every later overlapping read/write through an alias refuses.
  Addresses are never reused and physical heap metadata is never modified.

These are chosen ABI/service models, not a reconstructed Windows CRT. They do
not establish native allocation failures, address reuse, physical chunk sizes,
caller-saved clobbers or runtime timing. Each has authored negative controls,
including partial writes, reset after failure, quotas and malformed calls.

The pinned executable's import slots are `0xac54f0` (malloc), `0xac5424`
(memset), `0xac5428` (memcpy), and `0xac5500` (free). The owned PE table and
captured slot contents identify these services; no imported DLL code is run.

## Object lifetime and the callee's stack

The first free was a captured path backing array, not a modeled allocation.
An explicit logical-object declaration lets the model retire that object
without pretending to decode the native heap. Three declarations suffice:

| Captured base | Logical bytes | Evidence |
|---|---:|---|
| `0x34a21050` | 160 | Unit path header: capacity ten; PDB `PathData` size sixteen; observed copy of 160 before growth. |
| `0x32cd4e90` | 256 | Captured graph pool at header `0xc8d9b0`: capacity sixty-four pointers; exact existing 256-byte graph region; observed copy before growth. |
| `0x346464c8` | 256 | Header `0xc8d8c0`: capacity sixty-four pointers; broad payload and observed 256-byte copy before growth. |

These sum to 672 logical bytes, of which 256 were already in the narrow graph.
An initially registered fourth pool was unused and was removed from the final
recipe. `payload_refs.py` now supplies the recurring bounded, aligned pointer
search; a match reports raw neighboring words and does not infer an object.

The owned type export places `Unit.path` at `+0xb8`; `Stack<PathData>` has
list, capacity, length and increment at successive four-byte offsets. The
owned growth call sites at `0x46e8c0`, `0x457540`, and `0x4574c0` corroborate
the copy/free transitions. These are local evidence references, not imported
source. Logical object bounds remain explicit model inputs.

After A* returned, execution exposed a different modeling error: the original
callee writes its own stack parameter slots. The pinned function ends with
`RET 0x18` at `0x68371e`, and the reached stores at `0x6834cc` and `0x6834dc`
target two of those six words. The opt-in `--mutable-arguments` divides the
52-byte captured region into a read-only return word, six writable argument
words, and a read-only six-word caller tail. Tests exercise writes and resets
as well as refusal outside that exact argument interval. The narrow baseline
runner's default policy is unchanged.

## Reproduction and retained evidence

With the owned install and validated L62 packet:

```sh
uv run --offline tools/explore/payload_replay.py "$INSTALL" "$PACKET" \
  --services malloc+memset+memcpy+free \
  --borrow 0x34a21050:160 --borrow 0x32cd4e90:256 \
  --borrow 0x346464c8:256 --mutable-arguments \
  --repeat-final 8 --observe-unit-path > /tmp/modeled-resumption.json
```

The declarations are specific to this packet, not portable pointers. The
extended-state and arena controls add `--perturb-extended-state` or
`--arena 0x19710000`. Limits remain one million instructions per attempt,
128 dependency attempts, a one-MiB allocation arena, 1,024 calls per service,
and bounded copy/fill sizes and borrowed-object totals.

Sources, observations, controls and a hash manifest are retained at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-modeled-resumption`.
They bind payload SHA-256
`d0b4ce6ca20fcf2c1e17f6d385d48150cf84660c706ae944c3b372897824756f`
and original image SHA-256
`30478a44b577cb11ebcbbbf53d3e93ba02fd2aacf3bdefa6552c9b6449625079`.
The final write digest is
`d6e4388e3b15e718e2dc087e540578845d902b91da1ea592c5a33f92c1e6f688`.

## Validation and next falsifier

All **111 focused tests pass**, including the retained-context checks and
negative service/lifetime/stack/observer tests. The full release gate passed
at `/tmp/modeled-resumption-gate`: 332 rondata, 855 sim, 13 fixed and three
doc tests; all 782 requested fixtures present; clippy, formatting, data survey
and paperwork guards clean. Peak tree RSS was 8,132 MiB with two test workers.
The final validation-note edit also passes the paperwork guard.

The five retained lab gamelogs were checked for frame 224; none contains it.
The current packet logs nearby frames 221/222, then 421/422. The existing unit
logger and Rust parser can expose active path entries, but these logs do not
supply this call's post-state. Even an end-frame record would need careful
boundary matching, and would not cover unused path capacity.

The next useful experiment is a bounded **native post-return unit and path
observation**, paired with the same pre-call packet, followed by comparison of
every recorded field and slot. Prepare that collector offline before requesting
a live lane through Ramon. Until then this is a completed modeled execution
with one native scalar agreement. Native path fidelity, complete mutated-state
fidelity and general allocator/extended-state behavior remain open. Fable can
review the method independently; no adoption or capture permission is implied.
