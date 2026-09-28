"""What a capture stanza's checks may and must say (parked 941, 961, 966).

Three rules that were prose, each paid for by a run:

- **No check anchors `$` on a gamelog line.** The dump's lines end in `\\r`,
  so `grep -q "MAP_STYLE 22$"` fails on the right value; run316's first two
  checks did, and the queue said `checks FAILED` of a good capture (item 935).
- **A stanza with an `@` line carries `issuesmatch.py`.** `cmdsran.py` cannot
  see an `@` line, and run314's six refusals were found by reading `INFO 17`
  by hand (item 934).
- **`issuesmatch.py --none-refused`** is that check for a stanza with no
  golden twin: item 959 wrote it inline, three times.

The lint reads `tools/gamelog/captures.txt` as `runqueue.sh` does — comment
lines dropped, a blank line ends a stanza — and the unit cases below are the
lines that fooled a run, so the lint is seen to fire before it is trusted
to be quiet.
"""
import importlib.util
import re
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
CAPTURES = ROOT / 'tools/gamelog/captures.txt'

spec = importlib.util.spec_from_file_location(
    'issuesmatch', ROOT / 'tools/gamelog/issuesmatch.py')
issuesmatch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(issuesmatch)

# A stanza that may stand without the matcher, and why. Exact: a stanza
# added here is a decision, and one that gains its check leaves.
WITHOUT_THE_MATCHER = {
    '314': 'the run that found the refusals: every issuer refused under cover=1 (item 923)',
}

# A quoted pattern that ends in an unescaped `$`, or has one before a `|`
# alternative: `"MAP_STYLE 22$"`, `"a$|b$"`.
ANCHORED = re.compile(r'''(['"])(?:(?!\1).)*?[^\\]\$(?:\||\1)''')
# The stages that hand a gamelog's lines on as they are, `\r` and all.
PASSES_LINES = ('grep', 'head', 'tail', 'cat', 'LC_ALL=C')


def stages(check):
    """A check's pipeline, split on the pipes that are outside quotes."""
    out, cur, quote = [], '', None
    for ch in check:
        if quote:
            quote = None if ch == quote else quote
        elif ch in '\'"':
            quote = ch
        elif ch == '|':
            out.append(cur.strip())
            cur = ''
            continue
        cur += ch
    return out + [cur.strip()]


def anchors_a_gamelog_line(check):
    """Whether a grep in this check anchors `$` on a line read straight from
    a gamelog: the grep names the file itself, or every stage before it
    passes lines through and the first one names the file. A pipeline that
    starts in a tool (`report.py`, `track.py`) prints lines of its own."""
    raw = False
    for i, stage in enumerate(stages(check)):
        words = stage.split()
        if not words or not any(w in PASSES_LINES for w in words[:2]):
            raw = False
            continue
        names = 'gamelog' in stage
        raw = names or (raw and i > 0)
        if raw and 'grep' in words[:2] and ANCHORED.search(stage):
            return True
    return False


def stanzas(text):
    """[(run, {key: [values]})] as `runqueue.sh` would flush them."""
    out, cur = [], {}
    for line in text.splitlines() + ['']:
        if line.startswith('#'):
            continue
        if line == '':
            if cur.get('run'):
                out.append((cur['run'][0], cur))
            cur = {}
            continue
        key, _, value = line.partition(': ')
        cur.setdefault(key, []).append(value)
    return out


def anchored_checks(text):
    """[(run, check)] for every check that greps a `$`-anchored pattern."""
    return [(run, check) for run, stanza in stanzas(text)
            for check in stanza.get('check', []) if anchors_a_gamelog_line(check)]


def unmatched_issuers(text):
    """The runs whose stanza has an `@` line and no `issuesmatch.py` check."""
    return [run for run, stanza in stanzas(text)
            if any(re.match(r'\d+ @', c) for c in stanza.get('cmd', []))
            and not any('issuesmatch.py' in c for c in stanza.get('check', []))]


RUN316 = '''run: 316
tag: britishisles-largetown
check: grep -a -m1 "MAP_STYLE" "$L/gamelog-run316.txt" | grep -q "MAP_STYLE 22$"
'''

RUN340 = '''run: 340
tag: staged
cmd: 620 @queueup 0 2005 551 1
check: python3 tools/trace/report.py "$L/rontrace-run340.log" summary | grep -c "INFO 17"
'''


# **The held-out map is measured, never debugged against** (DECISIONS 41,
# 53 and 54; parked 986, the eighteenth pass). Its stanzas are pinned by
# run: a capture on map 9 that is not here is an item opened against the
# one map that says whether the rules generalise, and it fails until a
# steering pass adds its run — which is the pass deciding to measure.
HELD_OUT_MAP = '9'
HELD_OUT_RUNS = {
    '348',  # item 972, DECISIONS 53 §4: run33's shape, one diff, 1 and 0
}


def held_out_runs(text):
    return {run for run, stanza in stanzas(text)
            if stanza.get('mapstyle', [''])[0] == HELD_OUT_MAP}


class HeldOut(unittest.TestCase):
    def test_a_capture_on_the_held_out_map_is_found(self):
        text = 'run: 1\nmapstyle: 9\n\nrun: 2\nmapstyle: 14\n\nrun: 3\nmapstyle: 9\n'
        self.assertEqual(held_out_runs(text), {'1', '3'})

    def test_the_held_out_map_s_captures_are_the_pinned_ones(self):
        self.assertEqual(held_out_runs(CAPTURES.read_text()), HELD_OUT_RUNS)


class StanzaLint(unittest.TestCase):
    def test_an_anchored_grep_is_found(self):
        self.assertEqual([run for run, _ in anchored_checks(RUN316)], ['316'])

    def test_a_dollar_that_is_a_variable_or_an_awk_field_is_not_an_anchor(self):
        quiet = ('run: 1\ntag: t\n'
                 'check: grep -ao "BEGIN FRAME [0-9]*" "$HOME/ron-data/x.txt" | '
                 'awk \'/FRAME/{f=$3} END{print f; exit !(f>0)}\'\n')
        self.assertEqual(anchored_checks(quiet), [])

    def test_an_anchor_on_a_tool_s_own_lines_is_not_a_gamelog_s(self):
        # report.py prints its lines without the dump's `\r`; twenty-five
        # stanzas anchor a function's name on them, rightly.
        tool = ('run: 1\ntag: t\n'
                'check: python3 tools/trace/report.py "$L/rontrace-run1.log" entered | '
                'grep -E "CommandManager::issue_flight$|CommandManager::issue_buildmask$"\n')
        self.assertEqual(anchored_checks(tool), [])

    def test_an_anchor_in_the_grep_that_names_the_gamelog_is_found(self):
        direct = 'run: 2\ntag: t\ncheck: grep -a -c "seed 12345$" "$L/gamelog-run2-x.txt"\n'
        self.assertEqual([run for run, _ in anchored_checks(direct)], ['2'])

    def test_an_issuer_stanza_without_the_matcher_is_found(self):
        self.assertEqual(unmatched_issuers(RUN340), ['340'])
        matched = RUN340 + 'check: python3 tools/gamelog/issuesmatch.py --none-refused rontrace-run340.log\n'
        self.assertEqual(unmatched_issuers(matched), [])

    def test_no_check_in_the_capture_file_anchors_a_gamelog_line(self):
        self.assertEqual(anchored_checks(CAPTURES.read_text()), [],
                         'a gamelog line ends in \\r: `$` never matches it (parked 941)')

    def test_every_issuer_stanza_in_the_capture_file_carries_the_matcher(self):
        self.assertEqual(sorted(unmatched_issuers(CAPTURES.read_text())),
                         sorted(WITHOUT_THE_MATCHER),
                         'an `@` line is invisible to cmdsran.py (parked 961); '
                         'a stanza with no golden twin takes --none-refused (966)')


def issue(frame, line, refusal=0, before=0, after=12, named=1):
    return (17, frame, line | (refusal << 16), before, after, named)


class NoneRefused(unittest.TestCase):
    def verdict(self, found, *extra):
        argv = ['issuesmatch.py', '--none-refused', '/abs/rontrace.log', *extra]
        with patch.object(issuesmatch, 'records', return_value=found), \
                patch('sys.argv', argv), patch('builtins.print'):
            return issuesmatch.main()

    def test_every_line_issued_is_a_pass(self):
        self.assertEqual(self.verdict([issue(620, 0), (18, 620, 0, 7, 100, 200), issue(640, 1)]), 0)

    def test_one_refusal_is_a_failure(self):
        # run314's shape: refusal 2 on every issuer.
        self.assertEqual(self.verdict([issue(620, 0), issue(640, 1, refusal=2)]), 1)

    def test_a_trace_with_no_issue_record_is_a_failure(self):
        self.assertEqual(self.verdict([]), 1)

    def test_a_count_that_is_asked_for_is_held_to(self):
        found = [issue(620, 0), issue(640, 1)]
        self.assertEqual(self.verdict(found, '2'), 0)
        self.assertEqual(self.verdict(found, '3'), 1)


if __name__ == '__main__':
    unittest.main()
