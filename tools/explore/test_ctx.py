"""Cases for tools/ctx.py: a lane reads its own context (DECISIONS 134 §9)."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
import ctx  # noqa: E402


class Ctx(unittest.TestCase):
    ROWS = [{'name': 'lane-1', 'sessionId': 'aaa', 'ctx': 212_400},
            {'name': 'lane-2', 'sessionId': 'bbb', 'ctx': 96_000},
            {'name': 'gone', 'sessionId': 'ccc', 'ctx': None}]

    def test_the_row_is_found_by_session_and_in_either_shape(self):
        # Red first: the reader took a dict's `rows` and lore b179 answers
        # `agents`, so every session read "no row".
        self.assertEqual(ctx.ctx_of(ctx.rows_of({'count': 3, 'agents': self.ROWS}), 'aaa'), 212_400)
        self.assertEqual(ctx.ctx_of(ctx.rows_of({'rows': self.ROWS}), 'bbb'), 96_000)
        self.assertEqual(ctx.ctx_of(ctx.rows_of(self.ROWS), 'bbb'), 96_000)
        for session in ('ccc', 'zzz', None, ''):
            self.assertIsNone(ctx.ctx_of(self.ROWS, session))

    def test_the_line_plans_a_landing_past_the_mark(self):
        self.assertEqual(ctx.line(96_000), 'context: 96 k — under 200 k: carry on')
        deep = ctx.line(212_400)
        self.assertIn('context: 212 k', deep)
        self.assertIn('land the first act', deep)
        self.assertIn('never clear mid-build', deep)


if __name__ == '__main__':
    unittest.main()
