# L85: the internal continuation gap begins in cleanup and recycler state

The L82 chain and L81 second-input replay produce the same native unit/path/mode
boundary but execute 33,494 and 34,845 instructions respectively. Their first
instruction divergence occurs during cleanup of an active tree, before restoring
the unit's saved search. Four diagnostic entry-word substitutions make their
entire instruction-address/size streams agree. They do not establish equal heap
state, a native allocator model or a correct intervening-world simulation.

No new capture was used, no main score moved, and PR #6 stays frozen.

## Diff first, then read the responsible code

Both traces first pass the existing strict native second-output comparison.
The chained model executes the witnessed limit-95 first call and the original
caller collide instruction before re-entry. The independent model is prepared
from the fully validated second packet, bound to the compact fixture's retained
second payload hash. Both stop at the same callee boundary.

The common instruction prefix ends at zero-based index 445. At index 442,
`BRTree<int,unsigned_long>::close` reads its head-node word at 0x345f3670:
zero in the chain, nonzero in the second-input model. The conditional branch
then skips or enters recursive cleanup. This is the active pathfinder validlist,
not the unit's saved validlist about to replace it.

Owned-image evidence: `INDEX.tsv` maps `close` to 0x454000 and `delete_children`
to 0x455340. The PDB `PathFinderData` names validlist at +0x10 and blocklist at
+0x0c; both corresponding tree types place head_node at +0x0c. The owned
`PathFinder::astar_path` export at 0x683770 shows cleanup of active containers
before adopting the unit's saved containers. These are reading locators, not
transcribed implementation. The dynamic branches and ablations below back the
narrow execution claim; they do not identify the writer that populated the
active containers between native captures.

After omitting that first cleanup, the first divergent instruction moves to
index 480: a nonempty active blocklist at 0x345f36a0. After omitting both tree
cleanups, the next split is index 5796, in the recycler for
`TreeNode<PathNode_*,int>` at 0x47a6b0, reading length at 0xc8d9b8.
After that intervention, the next split is index 6220, in the recycler for
`BRTreeNode<TreeNode<PathNode_*,int>_*,unsigned_long>` at 0x47a750,
reading length at 0xc8d828. The owned index names these functions; traces retain
the actual reads, registers and branch windows outside Git.

## Bounded interventions and the timing trap

The four entry words are the two active head-node pointers and the two recycler
lengths. Their values in the captured second input are nonzero, nonzero, 1, 2;
the chained entry values are 0, 0, 0, 1. Every other input byte is preserved.
Each trial resets to the second capture, repeats exactly, and must pass the
native return, modes, complete unit and all path-capacity bytes comparison.
Only the pre-existing explicit path-pointer relocation is allowed.

| Second-input intervention | Instructions | First differing instruction versus chain |
| --- | ---: | ---: |
| None | 34,845 | 445 |
| Null active validlist root | 33,675 | 480 |
| Also null active blocklist root | 33,339 | 5,796 |
| Also set first recycler length to 0 | 33,416 | 6,220 |
| Also set second recycler length to 0 | 33,572 | 3,719 |
| Instead set second recycler length to its chained entry value, 1 | 33,494 | None: full sequence agrees |

The zero-length fourth trial is retained as a useful negative control. A first
automation assertion wrongly assumed all four chained entry words were zero;
it failed. The second recycler is zero at a later pop, but one at entry.
Substituting the later value at entry changes an earlier branch. The final
experiment explicitly retains captured entry values, chained entry values and
the diagnostic zero override separately. Counts alone are not the assertion:
the matched-entry trial compares every instruction address and size.

These omissions bypass cleanup and hide pool entries. They can leave unreachable
objects and alter retirement/reuse history. They are **not coherent replacement
world states**, implementation fixes, or new native intervention witnesses.
They isolate which retained inputs are sufficient to remove this measured
control-flow difference. Complete memory and allocation fingerprints remain
separate from instruction and output agreement; the final report names any
fingerprint components that still differ. In the final matched-entry trial,
registers, declared memory, initializedness, model history and write hashes all
still differ from the chain. Both unmodified baselines restore.

## Reproduction and validation

`tools/explore/continuation_cleanup.py` implements the bounded experiment with
existing byte guards and explicit model services. Shared-region word patches
combine without losing earlier edits. Undeclared, ambiguous, unaligned,
read-only, scratch and duplicate word patches refuse. Trace comparison detects
an ended prefix as a difference, not equality. Executable assertions pin all
measured counts, divergence indices, repetition and native output comparisons.
The focused suite includes counterexamples that break these claims and passes
71 tests. The full release gate passes 1,203 tests, 782 fixture requests with
none missing, clippy, formatting, the install survey and paperwork checks.
The final complete experiment took 100.89 seconds; second-input preparation
accounted for 85.70 seconds.

```sh
PYTHONPATH=tools/explore uv run --offline --with unicorn==2.1.4 python \
  tools/explore/continuation_cleanup.py INSTALL COMPACT_FIXTURE SECOND_PACKET
```

Use the L83 fixture and L81 `packet-1`; generated JSON belongs outside Git.
The broad second input is validated/prepared once per invocation, then reused
for the bounded trials. This command is not the capture-free-reader L83 command:
it reads the existing broad payload, but never launches the game.

Evidence is under
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-continuation-cleanup-gap`,
including exploratory traces, failed-assumption context, final tool output,
source copies and validation. The next boundary to establish is which intervening
native activity owns this shared cleanup/recycler state. Matching a single
unit's saved graph does not by itself reconstruct those shared inputs. The
retained L81 trace records one other A* call in frame 224 between the selected
call and its frame-225 continuation. This is an existing candidate to investigate,
not proof of which activity wrote either tree or recycler word.
