# Audits: the second reading

Each mechanic document in `docs/` was written by one reader from the
decompiled original. The files here are the record of a **blind second
reading**: an independent reader re-derives the mechanic from the same
decompile export without seeing the document or the implementation, and a
third reader adjudicates every disagreement back to the decompiled function
and records a verdict — A (the document) right, B (the second reading) right,
both wrong, or genuinely ambiguous — with the function and the expression that
settles it.

They are reading notes in the same sense as the documents: short expressions
are quoted where a verdict turns on one, nothing is transcribed, and nothing
here is a source to implement from. What they are for:

- **A list of corrections the document and implementation owe.** Each file ends
  with "must change". Until a correction lands, the document is the one that is
  wrong, and it should say so in its own "second reading" note.
- **A record of what is now doubly confirmed**, so a later reader knows which
  claims two people reached independently.
- **Closed open questions**, where the second reader settled something the
  first had listed as not established.

The first pass (2026-08-20) covered the seven mechanics then implemented. Its
headline: arithmetic doubly confirmed almost everywhere; predicates, scopes and
wiring wrong in several places — which unit kinds are exempt, which step a
multiplier belongs to, which building a rule applies to. These are exactly the
errors that tests written from the same reading cannot catch, which is why the
second reading is now part of a mechanic's definition of done (`CLAUDE.md`,
working agreement).

How it was run: six `lean` subagents on Fable 5, one per mechanic, given the
entry points and the traps but not the documents, writing to a scratch
directory; then one adjudicator per report with both readings and the export.
About an hour of wall clock for all seven, because `tools/ghidra/export.sh`
had already turned the decompile into files.
