"""The queue ledger reads an item number of any width the queue has minted.

Parked 1006, the eighteenth pass: every pattern in `tools/queueledger.py`
was `\\d{1,3}`, so from item 1000 on `1002. ` booked nothing and `(1003)`
parked nothing. The counts stood still for a tranche (492 booked, 532
live) and an item could leave the queue in silence — the one failure the
ledger exists to catch. These fail on that tree.
"""
from pathlib import Path
import importlib.util
import re
import unittest
import subprocess
import tempfile
from unittest.mock import patch
import contextlib
import io

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

    def test_a_call_s_argument_is_not_a_park(self):
        # Loop 3103: `(on - 1).min(7)` in docs/PARKED.md read as park 7 and the
        # ledger noted it live on every gate of the forty-first tranche.
        self.assertEqual(queueledger.parens('`ally_i.min(7)` and (1006); f(1050) x (1051)'), {1006, 1051})

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


class TheParkedFileIsGone(unittest.TestCase):
    # DECISIONS 125: docs/PARKED.md is archived, and a parked finding lives
    # in the journal file of the item that found it.
    def test_a_cited_park_is_read_only_after_the_word(self):
        text = ('held out (parked 4124a); kills (4118a to 4118c) and (parked 4058a, 4058b); '
                'parked 4141a to 4141c; run1561a and frame 4118a stand outside it; parked 1713 is an item')
        self.assertEqual(queueledger.cited_parks(text), {'4124a', '4058a', '4058b', '4141a', '4141c'})

    def test_a_park_resolves_in_the_index_or_its_item_s_journal(self):
        index = '# index\n\n- (4124a) the 80 leaves\n- (1224) fifteen seams\n'
        journals = {4200: '## Parked\n\n- **4200a. A thing found**: text\n', 4201: 'names 4200b in passing'}
        cited = {'4124a', '4200a', '4200b', '4201a'}
        # 4200b is written in 4201's journal, not in 4200's: the id names its item.
        self.assertEqual(queueledger.unresolved_parks(cited, index, journals), ['4200b', '4201a'])

    def test_the_archive_is_pinned_to_a_commit_that_holds_the_file(self):
        self.assertRegex(queueledger.ARCHIVE_REV, r'^[0-9a-f]{40}$')
        self.assertEqual(queueledger.QUEUES, ['docs/QUEUE.md'])
        held = queueledger.git('show', '%s:%s' % (queueledger.ARCHIVE_REV, queueledger.ARCHIVE_PATH))
        self.assertGreater(len(held), 1_000_000)
        rows = len(re.findall(r'^- \(', queueledger.ARCHIVE_INDEX.read_text(), re.M))
        self.assertLessEqual(rows, queueledger.ARCHIVE_ROWS)
        self.assertGreater(rows, 0)


class HistoricalBlobs(unittest.TestCase):
    def test_batch_preserves_show_bytes_order_and_repeated_requests(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            subprocess.run(['git', 'init', '-q', d], check=True)
            refs = []
            # Long body, blank body and multibyte text exercise exact byte lengths.
            for data in [b'\n1061. **first**\n', b'', ('é\r\n' * 20000).encode()]:
                result = subprocess.run(['git', 'hash-object', '-w', '--stdin'], cwd=root,
                                        input=data, check=True, capture_output=True)
                refs.append(result.stdout.decode().strip())
            refs = refs[::-1] + refs[:1]
            with patch.object(queueledger, 'ROOT', root):
                expected = [queueledger.git('show', ref) for ref in refs]
                self.assertEqual(list(queueledger.historical_blobs(refs)), expected)
                self.assertEqual(list(queueledger.historical_blobs([])), [])
                with self.assertRaises(RuntimeError):
                    list(queueledger.historical_blobs(['0' * 40]))
                with self.assertRaises(RuntimeError):
                    list(queueledger.historical_blobs(['HEAD\nHEAD']))

    def test_a_ledger_that_reads_no_queue_fails(self):
        # Loop 4041 (the seventy-sixth pass): the queue files were read only
        # when they existed, so a moved or renamed docs/QUEUE.md left `ever`
        # empty and the guard printed "0 numbers ever booked" and exited 0.
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root/'docs/journal').mkdir(parents=True)
            (root/'docs/audit').mkdir()
            subprocess.run(['git', 'init', '-q', d], check=True)
            journal = root/'docs/JOURNAL.md'; journal.write_text('')
            ledger = root/'docs/audit/queue-ledger.md'; ledger.write_text('')
            with patch.multiple(queueledger, ROOT=root, JOURNAL=journal, JOURNAL_DIR=root/'docs/journal', LEDGER=ledger,
                                ARCHIVE_REV=None, ARCHIVE_INDEX=root/'docs/audit/parked-archive.md'):
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    code = queueledger.main()
                self.assertEqual(code, 1, output.getvalue())
                self.assertIn('docs/QUEUE.md', output.getvalue())
                self.assertIn('Loop 4041', output.getvalue())

    def test_silent_deletion_still_fails_against_both_readers(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root/'docs/journal').mkdir(parents=True)
            (root/'docs/audit').mkdir()
            subprocess.run(['git', 'init', '-q', d], check=True)
            queue = root/'docs/QUEUE.md'
            queue.write_text('\n1061. **must be accounted for**\n')
            subprocess.run(['git', 'add', '.'], cwd=root, check=True)
            subprocess.run(['git', '-c', 'user.name=Authored', '-c', 'user.email=authored@example.invalid',
                            'commit', '-qm', 'authored'], cwd=root, check=True)
            queue.write_text('')
            journal = root/'docs/JOURNAL.md'; journal.write_text('')
            ledger = root/'docs/audit/queue-ledger.md'; ledger.write_text('')
            with patch.multiple(queueledger, ROOT=root, JOURNAL=journal, JOURNAL_DIR=root/'docs/journal', LEDGER=ledger,
                                ARCHIVE_REV=None, ARCHIVE_INDEX=root/'docs/audit/parked-archive.md'):
                def result():
                    output = io.StringIO()
                    with contextlib.redirect_stdout(output): code = queueledger.main()
                    return code, output.getvalue()
                failed = result()
                self.assertEqual(failed[0], 1)
                self.assertIn('1061', failed[1])
                with patch.object(queueledger, 'historical_blobs', side_effect=lambda refs: (queueledger.git('show', ref) for ref in refs)):
                    self.assertEqual(result(), failed)
                journal.write_text('item 1061: accounted for\n')
                passed = result()
                self.assertEqual(passed[0], 0)
                with patch.object(queueledger, 'historical_blobs', side_effect=lambda refs: (queueledger.git('show', ref) for ref in refs)):
                    self.assertEqual(result(), passed)


if __name__ == '__main__':
    unittest.main()
