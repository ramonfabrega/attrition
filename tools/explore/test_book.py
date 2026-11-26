"""book.py: a booking swaps one item's paragraph and touches no other.

The case is the tranche of 2026-11-26's: a hand-rolled booking matched an
item's end by its stop sentence, the sentence was wrapped across two lines
in that paragraph and whole in the next, and the next item left with it.
"""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import book  # noqa: E402

QUEUE = """# The queue

Parity held out, sixth: first partings 7675 whole 3690 whole now
Other: 3690 stands here too

## The queue

4723. **Row 133's units from 3633** (a rules item). It builds the refusal. It stops when two landings
running move no column.

4726. **Row 83's groups from 3754** (a rules item). It builds the bell. It stops when two landings running move no column.

4671. **Row 53's leaders from 3716** (a rules item). The leaf is on disk.

## How to maintain this file

- A line.
"""

NEW = """4728. **Row 133's units from 3690** (a rules item). Item 4723 built the refusal. It stops when two
landings running move no column.
"""


class Book(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.queue = Path(self.dir.name) / 'QUEUE.md'
        self.queue.write_text(QUEUE)
        self.new = Path(self.dir.name) / 'new.md'
        self.new.write_text(NEW)

    def tearDown(self):
        self.dir.cleanup()

    def run_book(self, *argv):
        return book.main(['--queue', str(self.queue), *argv])

    def test_a_wrapped_stop_sentence_takes_no_second_item(self):
        self.assertEqual(self.run_book('4723', '--with', str(self.new)), 0)
        after = self.queue.read_text()
        self.assertEqual(book.numbers(after), [4728, 4726, 4671])
        self.assertIn("4726. **Row 83's groups from 3754**", after)
        self.assertEqual(book.items(after)[4726], book.items(QUEUE)[4726])
        self.assertNotIn('4723. **', after)

    def test_the_new_item_goes_where_it_is_told(self):
        self.assertEqual(self.run_book('4723', '--with', str(self.new), '--after', '4726'), 0)
        self.assertEqual(book.numbers(self.queue.read_text()), [4726, 4728, 4671])
        self.queue.write_text(QUEUE)
        self.assertEqual(self.run_book('4723', '--with', str(self.new), '--before', '4671'), 0)
        self.assertEqual(book.numbers(self.queue.read_text()), [4726, 4728, 4671])

    def test_an_item_leaves_with_no_successor(self):
        self.assertEqual(self.run_book('4726'), 0)
        self.assertEqual(book.numbers(self.queue.read_text()), [4723, 4671])

    def test_a_line_s_words_are_replaced_once(self):
        self.assertEqual(self.run_book('4723', '--with', str(self.new),
                                       '--line', '7675 whole 3690 whole now', '7675 whole 3739 whole now'), 0)
        after = self.queue.read_text()
        self.assertIn('first partings 7675 whole 3739 whole now', after)
        self.assertIn('Other: 3690 stands here too', after)

    def test_words_that_stand_twice_are_refused_and_nothing_is_written(self):
        self.assertEqual(self.run_book('4723', '--with', str(self.new), '--line', '3690', '3739'), 2)
        self.assertEqual(self.queue.read_text(), QUEUE)

    def test_words_that_do_not_stand_are_refused(self):
        self.assertEqual(self.run_book('4723', '--with', str(self.new), '--line', 'no such words', 'x'), 2)
        self.assertEqual(self.queue.read_text(), QUEUE)

    def test_an_item_that_is_not_live_is_refused(self):
        self.assertEqual(self.run_book('4999', '--with', str(self.new)), 2)
        self.assertEqual(self.queue.read_text(), QUEUE)

    def test_a_number_already_live_is_refused(self):
        self.new.write_text(NEW.replace('4728.', '4671.'))
        self.assertEqual(self.run_book('4723', '--with', str(self.new)), 2)
        self.assertEqual(self.queue.read_text(), QUEUE)

    def test_a_successor_that_is_not_an_item_is_refused(self):
        self.new.write_text('Row 133 goes on.\n')
        self.assertEqual(self.run_book('4723', '--with', str(self.new)), 2)
        self.assertEqual(self.queue.read_text(), QUEUE)

    def test_a_dry_run_writes_nothing(self):
        self.assertEqual(self.run_book('4723', '--with', str(self.new), '--dry-run'), 0)
        self.assertEqual(self.queue.read_text(), QUEUE)


if __name__ == '__main__':
    unittest.main()
