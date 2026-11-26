#!/usr/bin/env python3
"""book.py — a booking's edit to `docs/QUEUE.md`: one item leaves, its
successors stand where they are told, a scoreboard line's words change, and
no other item's paragraph moves by a byte.

    python3 tools/book.py 4728 --with next.md --after 4693 \\
        --line '4288 whole 3690 whole now' '4288 whole 3739 whole now'
    python3 tools/book.py 4690                  # the item leaves, nothing takes its place
    python3 tools/book.py 4728 --with next.md --dry-run

`next.md` holds the successors as the queue will carry them: one paragraph
an item, each opening `NNNN. **`. With neither `--after` nor `--before` they
stand where the item that left stood. `--line OLD NEW` replaces words that
stand exactly once in the file; it is how the `Parity held out:` lines, the
lanes line and the opener are moved, and it may be given any number of
times.

An item is its head line to the next blank line, whatever its sentences
say. The driver of 2026-11-26 booked each landing with a script of its own,
and one matched an item's end by its stop sentence: the sentence was wrapped
across two lines in that item and stood whole in the next, so the next item
left with it (`33bd00cb7` put it back; `queueledger.py` named it at the
gate's pins step, thirteen minutes later). So this checks what that script
could not: after the edit the queue holds the items it held, less the one
that left, plus the successors, and every other item reads byte for byte as
it did. Anything else is refused, exit 2, with nothing written.

It does not commit, mint a number, or know the seats' order: the guard
(`the_handoff_s_unseated_rows_stand_in_the_seats_order`) and
`queueledger.py` still read the file, and `tools/guard.sh the_handoff_s`
is the reflex after it.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HEAD = re.compile(r'(?m)^(\d{3,5})\. \*\*')


class Refused(Exception):
    pass


def spans(text):
    """`[(number, start, end)]` of the items, in the file's order. An item
    runs from its head to the blank line after it, that line included."""
    out = []
    for m in HEAD.finditer(text):
        blank = text.find('\n\n', m.start())
        out.append((int(m.group(1)), m.start(), len(text) if blank < 0 else blank + 2))
    return out


def numbers(text):
    return [n for n, _, _ in spans(text)]


def items(text):
    """`{number: paragraph}`."""
    return {n: text[a:b] for n, a, b in spans(text)}


def book(text, old, successors='', after=None, before=None, lines=()):
    """The queue's text with item `old` gone, `successors` standing in its
    place (or after or before the item named) and each of `lines`' words
    replaced. Raises `Refused`."""
    was = items(text)
    if len(was) != len(numbers(text)):
        raise Refused('the queue holds an item number twice')
    if old not in was:
        raise Refused(f'item {old} is not live in the queue')
    new = [n for n, _, _ in spans(successors)]
    if successors.strip() and (not new or not HEAD.match(successors.lstrip('\n'))):
        raise Refused('the successors do not open on an item (`NNNN. **`)')
    for n in new:
        if n in was:
            raise Refused(f'item {n} is already live in the queue')
    block = successors.strip('\n')
    block = block + '\n\n' if block else ''
    _, a, b = next(s for s in spans(text) if s[0] == old)
    if after is None and before is None:
        out = text[:a] + block + text[b:]
    else:
        out = text[:a] + text[b:]
        anchor = after if after is not None else before
        at = next((s for s in spans(out) if s[0] == anchor), None)
        if at is None:
            raise Refused(f'item {anchor} is not live in the queue to stand beside')
        i = at[2] if after is not None else at[1]
        out = out[:i] + block + out[i:]
    for old_words, new_words in lines:
        count = out.count(old_words)
        if count != 1:
            raise Refused(f'{old_words!r} stands {count} times in the queue; a line is named by words that stand once')
        out = out.replace(old_words, new_words)
    now = items(out)
    want = [n for n in was if n != old] + new
    if sorted(now) != sorted(want) or len(numbers(out)) != len(want):
        raise Refused(f'the queue would hold {sorted(now)} where the booking asks {sorted(want)}')
    moved = [n for n in was if n != old and now[n] != was[n]
             and not any(o in was[n] for o, _ in lines)]
    if moved:
        raise Refused(f'the booking would change item {moved[0]}, which it does not name')
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('old', type=int, help='the item that leaves')
    ap.add_argument('--with', dest='successors', type=Path, help='a file of the items that take its place')
    where = ap.add_mutually_exclusive_group()
    where.add_argument('--after', type=int, help='stand the successors after this item')
    where.add_argument('--before', type=int, help='stand the successors before this item')
    ap.add_argument('--line', nargs=2, action='append', default=[], metavar=('OLD', 'NEW'),
                    help='replace words that stand exactly once in the file')
    ap.add_argument('--queue', type=Path, default=ROOT / 'docs' / 'QUEUE.md')
    ap.add_argument('--dry-run', action='store_true')
    args = ap.parse_args(argv)
    text = args.queue.read_text()
    try:
        out = book(text, args.old, args.successors.read_text() if args.successors else '',
                   args.after, args.before, args.line)
    except (Refused, OSError) as e:
        print(f'book.py refuses: {e}', file=sys.stderr)
        return 2
    if not args.dry_run:
        args.queue.write_text(out)
    for n, a, _ in spans(out):
        print(out[a:out.find('\n', a)][:96])
    print(f'{len(numbers(text))} items before, {len(numbers(out))} after' + (' (dry run)' if args.dry_run else ''))
    return 0


if __name__ == '__main__':
    sys.exit(main())
