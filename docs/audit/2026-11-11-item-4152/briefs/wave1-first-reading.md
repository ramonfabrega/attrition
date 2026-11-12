# Wave 1 brief — first reading of one leaf family of the leaders record

You are a first reader for item 4152 of the attrition project (a Rust
reimplementation of Rise of Nations; `CLAUDE.md`, which you inherit, has the
rules). Your spawn line names your **family** and its **slug**. The family's
row — its leaves, their offsets, and the writers an earlier pass found from
the listing — is in
`docs/audit/2026-11-11-item-4152/README.md`, table "The families". Read that
row and nothing else of that file. **§20's writer list is a lead to check, not
a finding to copy.** It came from store sites in the listing and was not read
function by function.

## What to produce

`docs/audit/2026-11-11-item-4152/<slug>-reading.md`, in the worktree you
are in (`/Users/rf-studio/code/fun/attrition/.claude/worktrees/att-4152`).
**Append per leaf as you finish it. Never save it all at the end**: the
account's wall can kill you mid-run, and what is on disk is what survives.
Start the file in your first few minutes with a header (family, leaves, your
model) and add to it as you go. Write only that one file. Do not commit, and
edit nothing else.

For **each leaf**, a section with:

1. **Every writer, by both nets.**
   - The export: `grep -rn -- '->leafname\b' ~/ghidra-projects/decomp/funcs`
     and `grep -rn 'field_0x<off>\b'`. `<off>` is the `LeaderData` offset,
     and also that offset plus the `LeaderData` base inside whatever class
     the function's `this` is; check `types.txt` for `Leader` (size
     `0x6eec`) against `LeaderData` (`0x6ee4`).
   - The listing:
     `/opt/homebrew/opt/llvm/bin/llvm-objdump -d --no-show-raw-insn --start-address=0x... --stop-address=0x... /Users/rf-studio/code/fun/attrition/game/riseofnations.exe`.
     The leaders array's `LeaderData` for slot `i` sits at
     `0xe3a390 + i·0x6eec` (`docs/CHECKSUM.md` §8 line ~434). Stores show
     as `0xe3a390+off(%reg)`, or as `0x<off>(%reg)` off a `Leader*`. Below
     `+0x200`, a register-relative store cannot be told from another
     record's by displacement alone, so name the type of the base register
     from the function.
   - Read each writer's body around the store (`grep -n` and line range),
     enough to state its predicate.
2. **The event** that steps it, **by how much**, and **the predicate**:
   which kinds or slots are exempt, which branch, which index. The predicates
   are where earlier readings have been wrong. Arithmetic is in integers at
   the original's own scales; a frame is a sim frame. Name a constant's
   units. Where a value is a float, give its exact bits or rational and say
   so.
3. **What clears or resets it**: `Leader::init`, `Leaders::clear`,
   `reset_score`, or a per-frame fold.
4. **Who reads it**, briefly. A reader that changes a decision is worth a
   line.
5. **Where `crates/sim` already has that event**: grep the crate (for
   example `grep -rn 'fn take_damage\|fn do_damage' crates/sim/src`) and name
   the Rust function and file a builder would add the counter to, or say the
   crate has no such event and what it stops at (a SEAM comment, an unported
   system).
6. **Reached by a plain game?** State whether a skirmish against the
   computer, with no scenario or Conquer the World, reaches each writer, or
   whether it sits behind the trigger layer, CtW or multiplayer.
7. **Confidence and what is not read.** Say which callers you did not
   follow and which stores you could not attribute.

Close the file with a short **summary table**: leaf | writers (Name (addr))
| step | reset | crate site | plain game? | confidence.

## Rules

- Every shell call starts with
  `export PATH=$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH`.
- Never use a bare `cat`, `read` or `tee` with no operand. Read source with
  `grep -n` and `sed -n 'A,Bp' file`, never whole large files.
- **Cite a function as `Name (00xxxxxx)`, never `Name@00xxxxxx`**: the `@`
  form is checked by a guard that walks `docs/**`.
- `types.txt` names a `field_0xNN` (`grep -n 'struct /rise.pdb/LeaderData '
  -A400 ~/ghidra-projects/decomp/types.txt | grep ' 0x...'`), and
  `vtables.txt` names a vtable slot. `INDEX.tsv` maps an address to its file.
  `tools/ghidra/README.md` "Traps" lists the decompiler's traps; read it
  first (`sed -n '85,206p'`).
- Read; never transcribe a decompiled body into the file. State the rule in
  your own words and integers.
- No Ghidra headless, no run of the original, nothing from the install
  (`/Users/rf-studio/code/fun/attrition/game/`) copied into the file beyond
  addresses and constants.
- Grep the writers of every field you call frozen and the callers of every
  function you call once-only.
- Hand back one paragraph: leaves written, writers found, anything that
  contradicts the README row's writer list, and your token and time sense.
