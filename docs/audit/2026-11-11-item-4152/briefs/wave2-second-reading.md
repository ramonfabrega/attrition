# Wave 2 brief: a blind reading of one leaf family of the leaders record

You are a reader for item 4152 of the attrition project, a Rust
reimplementation of Rise of Nations. `CLAUDE.md`, which you inherit, has the
rules. Your spawn line names your **slug** and the **leaves**: field names of
`LeaderData` (`types.txt`, `struct /rise.pdb/LeaderData`, size `0x6ee4`),
with their offsets.

**You are the blind reading.** Another reader has written up this family.
Your value lies in not having seen that write-up. **Do not open:**

- anything in `docs/audit/2026-11-11-item-4152/` other than the file you
  write, and in particular its `README.md` and any `*-reading.md` or
  `*-verdicts.md`;
- `docs/CHECKSUM.md` §20 (lines 1666–1836);
- `docs/PARKED.md`;
- the journal entries `docs/journal/2026-11-11-item-41*.md`;
- `docs/LEADERS.md`, if it exists.

Re-derive the family from the decompile export and the PE alone. Every other
document under `docs/` and `crates/` is fair to read.

## What to produce

`docs/audit/2026-11-11-item-4152/<slug>-second.md`, in the worktree you are
in (`/Users/rf-studio/code/fun/attrition/.claude/worktrees/att-4152`).
**Append per leaf as you finish it. Never save it all at the end**: the
account's wall can kill you mid-run, and what is on disk is what survives.
Put a header (slug, leaves, your model) in the file within your first few
minutes. Write only that one file. Do not commit, and edit nothing else.

For **each leaf**, a section with:

1. **Every writer, by both nets.**
   - The export: `grep -rn -- '->leafname\b' ~/ghidra-projects/decomp/funcs`
     and `grep -rn 'field_0x<off>\b'`. `<off>` is the `LeaderData` offset,
     and also that offset shifted by the `LeaderData` base inside whatever
     class the function's `this` is; compare `Leader` (size `0x6eec`) with
     `LeaderData` in `types.txt`.
   - The listing:
     `/opt/homebrew/opt/llvm/bin/llvm-objdump -d --no-show-raw-insn --start-address=0x... --stop-address=0x... /Users/rf-studio/code/fun/attrition/game/riseofnations.exe`.
     The global leaders array's `LeaderData` for slot `i` is at
     `0xe3a390 + i·0x6eec`, so a store shows as `0xe3a390+off(%reg)` or as
     `0x<off>(%reg)` off a `Leader*`. Below `+0x200`, a register-relative
     displacement cannot be told from another record's, so name the type of
     the base register from the function.
   - Read each writer's body around the store (`grep -n` and a line range),
     enough to state its predicate.
2. **The event** that steps the leaf, **by how much**, and **the predicate**:
   which kinds or slots are exempt, which branch, which index. Arithmetic is
   in integers at the original's own scales, and a frame is a sim frame.
   Name each constant's units. For a float, give its exact bits or rational.
3. **What clears or resets it.**
4. **Who reads it**, briefly.
5. **Where `crates/sim` already has that event**: grep the crate and name
   the Rust function and file a builder would add the counter to. If the
   crate has no such event, say so and say what it stops at (a SEAM comment,
   an unported system).
6. **Reached by a plain game?** State whether a skirmish against the
   computer, with no scenario and no Conquer the World, reaches each writer,
   or whether it sits behind the trigger layer, CtW or multiplayer.
7. **Confidence, and what you did not read.**

Close the file with a **summary table**: leaf | writers (Name (addr)) | step
| reset | crate site | plain game? | confidence.

## Rules

- Every shell call starts with
  `export PATH=$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH`.
- Never use a bare `cat`, `read` or `tee` with no operand. Read source with
  `grep -n` and `sed -n 'A,Bp' file`, never a whole large file.
- **Cite a function as `Name (00xxxxxx)`, never `Name@00xxxxxx`.** A guard
  that walks `docs/**` checks the `@` form.
- `types.txt` names a `field_0xNN`, `vtables.txt` names a vtable slot, and
  `INDEX.tsv` maps an address to its file. `tools/ghidra/README.md`
  "Traps" (`sed -n '85,206p'`) lists the decompiler's traps; read it first.
- Read the decompiled bodies, but never transcribe one. State the rule in
  your own words and integers.
- No Ghidra headless and no run of the original. Copy nothing from the
  install into the file beyond addresses and constants.
- Grep the writers of every field you call frozen, and the callers of every
  function you call once-only.
- Hand back one paragraph: the leaves written, the writers found, and your
  sense of the tokens and time it took.
