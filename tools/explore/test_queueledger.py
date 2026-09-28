"""The queue ledger reads an item number of any width the queue has minted.

Parked 1006, the eighteenth pass: every pattern in `tools/queueledger.py`
was `\\d{1,3}`, so from item 1000 on `1002. ` booked nothing and `(1003)`
parked nothing. The counts stood still for a tranche (492 booked, 532
live) and an item could leave the queue in silence — the one failure the
ledger exists to catch. These fail on that tree.
"""
from pathlib import Path
import importlib.util
import unittest

TOOLS = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location('queueledger', TOOLS / 'queueledger.py')
queueledger = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(queueledger)


class FourDigitItems(unittest.TestCase):
    def test_a_four_digit_item_is_booked(self):
        text = 'prose\n\n1061. **Great Lakes\' second word**\n    (1052).\n\n997. **older**\n'
        self.assertEqual(queueledger.booked(text), {1061, 997})

    def test_a_four_digit_park_is_live(self):
        self.assertEqual(queueledger.parens('(1006) **the ledger** and (1050/1051)'),
                         {1006, 1050, 1051})

    def test_a_wrapped_sentence_is_still_not_an_item(self):
        # The blank line is the discriminator, at any width.
        text = 'the word is\n4924. East Indies follows.\n'
        self.assertEqual(queueledger.booked(text), set())

    def test_a_five_digit_number_is_a_frame(self):
        # Frames run to 24,000; an item number of five digits is a century
        # of tranches away, and `(18140)` is a capture's length.
        self.assertEqual(queueledger.parens('(18140) and (24000)'), set())
        self.assertEqual(queueledger.booked('\n24000. **no**\n'), set())

    def test_the_references_read_four_digits(self):
        refs = queueledger.references('takes 1061; item 1050 and items 1019; 1048 closes')
        self.assertEqual(refs, {1061, 1050, 1019, 1048})

    def test_the_journal_names_four_digits(self):
        named = queueledger.named_in('## 2026-09-28 — item 1052: the army\nitems 1040, 1034 and 997\n')
        self.assertEqual(named, {1052, 1040, 1034, 997})

    def test_no_pattern_is_three_digits_wide(self):
        source = (TOOLS / 'queueledger.py').read_text()
        self.assertFalse('{1,3}' in source.split('"""', 2)[2], 'a pattern spells its own width')


if __name__ == '__main__':
    unittest.main()
