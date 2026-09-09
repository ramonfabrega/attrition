# Native restore entry and its delegation boundary

## What is established

The congestion witness's first native restore is **frame 224, owner 0, object
16**. The probe captures the saved graph at restore entry, then the exact
registers, stack and budget immediately before delegation to the larger
pathfinder. Executing the original wrapper prefix outside Wine reproduces all
nine recorded registers, all 48 bytes of the delegation stack and every declared
semantic byte. It reads **36 semantic bytes**, uses 16 argument bytes and 32
bytes of initialized scratch, maps 32 KiB and executes **31 instructions**.

The captured repath counter is 1, yielding limit 300 and saving=1. Coordinates
(3336, 31848) are the unit's **current position**, not its path destination.
The structural graph at entry still has 223 physical nodes, 67 PathNodes and
three opaque CollBlocks. The next traced native A* call, in frame 224 at step
48 and anti=0, returns 1. This is live evidence of a successful search following
the restore handoff; that search has **not** yet been replayed outside Wine.

The prefix's 256 reset runs plus baseline take about 0.044 seconds locally,
including checks. Fresh execution matches the complete register/memory/write
history. All ten individual dependency omissions fail; four deliberately
corrupted outputs fail. These measurements concern a wrapper, not A* throughput
or end-to-end test-suite speed. No simulation headline floor moves.

## Instrument and evidence

`RON_RESTORE_PROBE` requires the census and graph modes. It moves the one graph
capture from the first suspension return to the first restore entry. The
census remains an independent observer of the same A* calls. Two small
trampolines preserve flags and integer registers: one at
`PathFinder::find_upath_restore@00688f40`, one at its direct call instruction
0x688fa5. The second delegates to the original `PathFinder::find_upath@00682f30`
and continues at 0x688faa. The original callee executes and returns through the delegation trampoline;
the restore wrapper's own return is left intact.
Prologue/call bytes, image base and coverage mode are checked before patching.

The local PE listing establishes the wrapper's explicit read set: GameAccess's
GameDaemon pointer at 0xc061bc, the selected repaths slot, objects pointer at
0xc0618c, selected owner array pointer, object pointer and encoded x/y words.
The mode pair at 0xe85ec0/0xe85ec4 is captured before and at delegation.
The wrapper's 101 original bytes end immediately before the direct call; the
replay stops there. It does not treat the downstream function as successful.

The driver bounds owner <8 and object ID <512, validates registry length and
capacity before fetching the unit, and copies through checked reads. It records
one invocation per process. The 196-byte authored prefix packet contains its
identity/frame/unit, nine entry registers, four entry stack words, nine
read-set words, nine delegation registers, twelve delegation stack words and
two delegation mode words. The graph is the existing typed snapshot format,
validated against this entry's unit/frame and receipt rather than a suspension
return. Original code and captured bytes remain outside git.

## The first dependency beyond the wrapper

A separate bounded experiment enters the real larger pathfinder with the native
handoff arguments and captured graph available. It stops on its fifth
instruction, **0x682f3a**, reading **FS:[0]**, the Windows exception-chain head.
No TEB or exception state has been invented to make that pass. This is a
thread-context boundary before a world-state boundary; the saved graph alone
is not a runnable resumed search.

The first attempt supplied only the 16-byte prologue, leaving zero-filled page
padding after it. Unicorn tried to translate onward into the next unmapped
page before executing any instruction. That was an emulator setup failure,
not an observed game dependency. Supplying the original first 256 code bytes
(which include control-flow boundaries), with byte guards still active, reaches
the actual FS read after five instructions. The report requires that exact
instruction and read failure; an unrelated failure cannot count as this result.

The older `tools/explore/command_oracle.py` already maps a coarse page at zero
for its FS exception-chain setup. This is not a newly discovered Wine
limitation. The strict bounded runner deliberately excludes address-zero
regions; importing the old whole-page accommodation would weaken its null-read
checks. The next implementation should give thread state an explicit segment
and byte contract.

Next: make the Windows thread context an explicit captured/emulated input, then
continue the same read-set experiment into the larger pathfinder. Its listing
also contains TLS and SIMD use later; their requirements must be measured,
not inferred from this five-instruction prefix or bypassed with fake success.

## Validation and artifacts

`/tmp/attrition-restore-entry` holds both captures, the trace and image binding.
`/tmp/restore-prefix-final.json` records the replay and first downstream
boundary. `restore_prefix.py INSTALL DIRECTORY` checks packet/graph receipts,
image identities, exact prefix replay, reset/fresh behavior, negative controls
and the downstream refusal. The ordinary congestion validator still matches
the original reference on all 1,401 RNG/frame pairs, roster, orders, targets,
A* returns and 64 census metadata shapes. The game was closed normally; all five
backed-up profile files were restored and compared byte for byte.

Four authored Python tests reject framing, altered handoff fields, every
missing binding/call receipt and explicit graph/restore failures. The
exact C callbacks pass host fixtures for entry/delegation, 24 failed/short-read
cases, graph failure, owner bounds, failed/short writes, single-shot behavior,
installation refusal and register-save stub shape. Live prefix replay supplies
the stronger check of the actual calling convention and register preservation.

Validation: full release gate passes (269 rondata, one ignored, 822 sim,
13 fixed, three doctests), rondata 238.90 seconds, process-tree peak 8,570 MiB
under 20 GiB. Log `/tmp/restore-prefix-release.log`. Clippy with warnings denied,
formatting, install survey and documentation guards pass. Four restore packet
Python tests, the 5/9/5 graph/census/scenario suites and the C callback fixture
pass. Default preprocessing is unchanged, the default and live DLL compile
without warnings, and restore mode without graph mode rejects.
