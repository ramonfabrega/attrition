"""A brief is the frame, the queue's own words, and what is reserved
(parked 1137, the nineteenth pass).

Until this tool a brief was the previous brief: the commander read the last
commander's from the job's scratch directory, copied the lane's last one and
edited it. What a steering pass ruled reached a worker as a clause added to
the copy, nothing ever left one, and the item before's sentences rode along
— ten journals of twenty found one. The frame is a file in git that the
pass writes, the item is quoted from `docs/QUEUE.md` and never retold, and
the checklist is held to the frame: a row no section carries fails here.
"""
import re
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import brief  # noqa: E402

QUEUE = '''# The queue

## Where things stand

**Opener: the commander resumes.**

## The queue

In dependency order.

1133. **Great Sahara's word: frame 8, ours 6 draws against 7** (1066),
    at index 0, widened on run382. No mechanism is named.

1127. **East Indies' second word: frame 6151, ours 9 draws against 8**
    (1120), at index 0. No mechanism is named.

1131. **Chapter thirty-eight's word: frame 878, ours 7 draws against 5**
    (1113). No mechanism is named.

## How to maintain this file

- Rewrite it.
'''


def checklist():
    text = (ROOT / 'docs/audit/README.md').read_text()
    a = text.index('## The brief checklist\n')
    b = text.index('## How a second reading is run')
    return text[a:b]


def row_numbers(text):
    """The item numbers a checklist row is filed under: every number of a
    parenthesis that opens on one — but not an example's (`976's`), a
    parked successor's (`parked 1134`) or a date's."""
    out = set()
    for group in re.findall(r'\((\d{3,4}\b[^()]*)\)', text):
        out.update(re.findall(r"(?<!parked )\b(\d{3,4})\b(?!'s|-\d)", group))
    return out


class TheFrameHoldsTheChecklist(unittest.TestCase):
    def test_every_row_of_the_checklist_is_in_a_section_of_the_frame(self):
        live = re.sub(r'~~.*?~~', '', checklist(), flags=re.S)
        rows = row_numbers(live)
        self.assertGreater(len(rows), 60)
        frame = row_numbers(brief.FRAME.read_text())
        self.assertEqual(sorted(rows - frame - brief.NOT_A_BRIEF_S), [],
                         'rows of the checklist that no section of tools/brief/frame.md carries')

    def test_the_frame_carries_no_struck_row(self):
        struck = set()
        for gone in re.findall(r'~~(.*?)~~', checklist(), flags=re.S):
            struck |= row_numbers(gone)
        self.assertIn('726', struck)
        live = row_numbers(re.sub(r'~~.*?~~', '', checklist(), flags=re.S))
        frame = row_numbers(brief.FRAME.read_text())
        self.assertEqual(sorted((struck - live) & frame), [])

    def test_the_frame_is_held_to_its_ceiling(self):
        # Parked 1226: the frame took a dozen rows a pass and shed none —
        # 10,125 characters to 12,959 in the twentieth pass, where a brief's
        # length had been measured as a cost. The ceiling is the size the
        # last pass left the frame at, and a pass that adds a row strikes
        # one; it may only fall.
        self.assertLessEqual(len(brief.FRAME.read_text()), brief.FRAME_CEILING)

    def test_every_kind_s_sections_are_in_the_frame(self):
        have = set(brief.sections(brief.FRAME.read_text()))
        for kind, names in brief.KINDS.items():
            self.assertEqual([n for n in names if n not in have], [], kind)


class ABriefIsComposed(unittest.TestCase):
    def compose(self, item, kind='residue', **kw):
        args = dict(queue=QUEUE, frame=brief.FRAME.read_text(), base='abc1234',
                    model='Opus 5.5', runs=['run423', 'run424'], sections=[], read=[], note='')
        args.update(kw)
        return brief.compose(item, kind, **args)

    def test_the_item_is_the_queue_s_own_words(self):
        text = self.compose(1133)
        self.assertIn("1133. **Great Sahara's word: frame 8, ours 6 draws against 7** (1066),\n"
                      "    at index 0, widened on run382. No mechanism is named.", text)
        self.assertNotIn('East Indies\' second word: frame 6151, ours 9 draws against 8** (1120),\n'
                         '    at index 0. No', text.split('## The other lanes')[0])

    def test_an_item_the_queue_does_not_book_is_refused(self):
        with self.assertRaisesRegex(ValueError, '1140'):
            self.compose(1140)

    def test_the_other_lanes_are_named_by_their_words(self):
        lanes = self.compose(1133).split('## The other lanes')[1].split('\n## ')[0]
        self.assertIn('1127', lanes)
        self.assertIn('frame 6151', lanes)
        self.assertIn('1131', lanes)
        self.assertNotIn('1133', lanes)

    def test_a_brief_fences_nothing_and_grants_nothing(self):
        text = self.compose(1131, 'chapter')
        for word in ('## Fenced', 'I grant', 'in your grant', 'fenced to'):
            self.assertNotIn(word, text)
        self.assertIn('Code is not fenced', text)

    def test_what_is_reserved_is_said(self):
        text = self.compose(1131, 'chapter', sections=['docs/GOLDEN.md §48'])
        reserved = text.split('## Reserved for you')[1].split('\n## ')[0]
        self.assertIn('run423', reserved)
        self.assertIn('run424', reserved)
        self.assertIn('docs/GOLDEN.md §48', reserved)

    def test_a_chapter_s_rows_are_a_chapter_s(self):
        self.assertIn('issuer chapter', self.compose(1131, 'chapter'))
        self.assertNotIn('issuer chapter', self.compose(1133))

    def test_no_placeholder_survives(self):
        for kind, item in (('residue', 1133), ('chapter', 1131)):
            text = self.compose(item, kind)
            self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])
            self.assertIn('abc1234', text)
            self.assertIn('Opus 5.5', text)
            self.assertIn(f'item-{item}.md', text)

    def test_the_commander_s_own_note_is_last_before_the_landing(self):
        text = self.compose(1133, note='Read `1/2`\'s path on block 6 first.')
        self.assertLess(text.index("Read `1/2`'s path"), text.index('## Landing'))
        self.assertGreater(text.index("Read `1/2`'s path"), text.index('## Build'))

    def carried(self, headline, text):
        """The item's headline is in the brief, however the queue wrapped
        it: a line break inside a title is the queue's and no defect."""
        self.assertTrue(' '.join(headline.split()) in ' '.join(text.split()),
                        f'the brief does not carry the headline: {headline}')

    def test_the_live_queue_composes(self):
        queue = (ROOT / 'docs/QUEUE.md').read_text()
        for item, headline in brief.open_items(queue):
            kind = 'chapter' if 'Chapter' in headline else 'residue'
            text = self.compose(item, kind, queue=queue)
            self.carried(headline, text)

    def test_a_title_the_queue_wraps_composes(self):
        # Booking 1174 (parked 1188): item 1185's bold title ran past the
        # line and the commander wrapped it, as every other line of the
        # queue is wrapped; the live test compared the headline with its
        # whitespace joined against the item as written, the booking gate
        # was red at its first step, and the reflex had not run this file.
        queue = QUEUE.replace(
            "1127. **East Indies' second word: frame 6151, ours 9 draws against 8**\n"
            "    (1120)",
            "1127. **East Indies' second word: frame 6151, ours 9 draws\n"
            "    against 8** (1120)")
        self.assertNotEqual(queue, QUEUE)
        self.assertIn((1127, "East Indies' second word: frame 6151, ours 9 draws against 8"),
                      brief.open_items(queue))
        text = self.compose(1127, queue=queue)
        self.carried("East Indies' second word: frame 6151, ours 9 draws against 8", text)
        lanes = self.compose(1133, queue=queue).split('## The other lanes')[1].split('\n## ')[0]
        self.assertIn('frame 6151, ours 9 draws against 8', lanes)


class TheReflexReadsTheQueue(unittest.TestCase):
    def test_the_reflex_runs_the_suites_that_read_the_queue(self):
        # Parked 1188: `tools/guard.sh` ran no offline test, so a queue
        # line these suites refuse was green on the reflex and red at the
        # booking gate's first step, with a worker already cut off it.
        guard = (ROOT / 'tools/guard.sh').read_text()
        code = '\n'.join(l for l in guard.split('\n') if not l.lstrip().startswith('#'))
        for suite in ('test_brief', 'test_queueledger'):
            self.assertIn(suite, code)


if __name__ == '__main__':
    unittest.main()
