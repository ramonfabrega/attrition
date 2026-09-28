# tools/emu — the original's functions, called

`callfn.py` maps `riseofnations.exe` at its preferred base under unicorn,
enters a function with arguments we choose, and reads `eax`. No game, no
bottle, no capture. `docs/EMULATOR.md` is the document: what it costs per
kind of function, what the first sweep found, and the verdict on where it
belongs in the oracle ladder.

```
uv run tools/emu/callfn.py <install>/riseofnations.exe sweep [SEED] [N]
```

The one third-party Python package under `tools/`, `unicorn`, is declared
inline in the script (PEP 723) and resolved by `uv`; nothing is installed
into the checkout. The sweep's rows are asserted by
`crates/sim/src/path.rs`'s `the_emulated_original_agrees_on_every_row`,
which reads `$RON_EMU_TABLE` or runs the script against `$RON_INSTALL`
(or the checkout's `game/`) and skips, saying so, when neither exists.

## Adding a function

Three things from the listing (`llvm-objdump -d --start-address=…`), the
same three `tools/trace`'s proxies need: the **calling convention** (a
register pair, or `__thiscall` with `ecx = this` and the stack), the
**argument count** — read the `ret <imm>` at the end and divide by four —
and the **layout of any struct passed by value**. Then a `Machine.call`
with the arguments in stack order, first argument nearest the return
address. A function that reaches a singleton faults on its first read of
unmapped memory and the fault names the address: that is the state to lay
out next, at the offsets `types.txt` in the decompile export gives. A
function that allocates or calls a DLL needs an import stub first; none
has been written.

## A function with its callees answered (`hooks.py`)

A game function calls a dozen others and reads its objects through
vtables. `hooks.py`'s `Harness` hooks each direct callee by address and each
vtable slot by a stub, records `(name, ecx, args)`, answers `eax` and pops
the callee's own `ret imm`; objects are laid out by hand in a bump heap at
the offsets `types.txt` gives, and a read of unmapped memory still names the
next field to lay. Items 976 and 1009 each built this scaffold as a scratch
script and lost it with their lanes; item 1019 graduated it.

```
uv run tools/emu/train_arm.py <install>/riseofnations.exe
```

is its first user: `Build::train@0062f9b0`'s CARRY_AIR arm for a
Helicopter, a missile and a Fighter under each shape of gather list
(`docs/PRODUCTION.md`, "The Helicopter and the missile under a point").
