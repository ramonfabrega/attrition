# The journal, one file per landing

Since 2026-09-17 a landing's story is a file here, `<date>-item-<N>.md`,
written by the worker whose item it is and edited by nobody else.
`docs/JOURNAL.md` is the chronicle up to that day and stays append-only
for the steering passes' own entries; the two texts are one journal to
`tools/queueledger.py`, which reads both, and to `grep -r`.

The reason is measured, not stylistic: `git merge-tree` over the 58 merges
of 2026-09-06 to 09-17 found **27 real content conflicts** in
`docs/JOURNAL.md` and 7 in `docs/QUEUE.md`, because an append-only file
puts every worker's addition at the same anchor. A conflict is a turn or
two of a commander's context to resolve by hand, and a wrong resolution
drops or duplicates an entry silently. A file per landing cannot conflict.

The rules that ride with it:

- **The first line is the heading the single file used**: `## <date> —
  item <N>: <title> (<model>)`. The ledger's `item N` and heading regexes
  read it, so a deletion from the queue is still caught.
- **Everything the working agreement asks of a journal entry still
  applies** — the score before and after, the value diff on the frame the
  word moved, what was established and what was not.
- **A worker touches neither `docs/JOURNAL.md` nor `docs/QUEUE.md`.** It
  reports its numbers in its done message; the commander books findings
  and writes the scoreboard line. The floors test in `rondata::diff` stays
  the worker's, so a commander who forgets the line fails the gate.
- **A steering pass writes to `docs/JOURNAL.md`**, not here: it is the
  main thread, the only writer that day, and the chronicle is its record.
