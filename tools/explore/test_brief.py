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


class TheAlwaysSet(unittest.TestCase):
    def test_every_kind_s_lane_gate_runs_the_blind_pin(self):
        # Loop 2989 (2976's booking gate, red on its first commit): a
        # document lane cited nineteen functions no trace enters, and its
        # lane gate (`--tests nations::`) never ran
        # `the_blind_list_is_the_pinned_residue`, so the re-pin fell to the
        # booking gate's second commit. The pin is 0.4 s; every kind runs it.
        for kind in brief.ALWAYS_TESTS:
            if kind in brief.WHOLE:
                # Loop 4022: a structure lane gates whole, `blind::` among
                # every test; it names no filter.
                self.assertEqual(brief.filters(kind, []), [], kind)
                continue
            self.assertIn('blind::', brief.filters(kind, ['nations::'] if kind != 'sweep' else []), kind)
        self.assertEqual(set(brief.ALWAYS_TESTS), set(brief.KINDS))
        # A reading lane (DECISIONS 84) moves no word: its gate is the blind
        # pin alone, and it names no filter of its own.
        self.assertEqual(brief.filters('reading', []), ['blind::'])
        self.assertNotIn('first_parting', brief.filters('reading', []))

    def test_a_word_lane_s_gate_runs_the_audited_guard(self):
        # Loop 3880: a melee or pair word that closes whole is held to an
        # `audited!` row or an `OWED` line, and 3862's and 3873's lane gates
        # never ran the guard, so each took a second commit and a re-gate at
        # the booking gate. The guard alone, never the module: `diff::audit::`
        # also reads the long words' dumps.
        guard = 'audit::every_whole_scoreboard_word_is_audited'
        for kind in ('residue', 'chapter'):
            self.assertIn(guard, brief.filters(kind, ['airbase_melee::']), kind)
            self.assertNotIn('diff::audit::', brief.ALWAYS_TESTS[kind], kind)
        for kind in ('reading', 'coverage', 'sweep'):
            self.assertNotIn(guard, brief.filters(kind, []), kind)


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
        # one; it may only fall. The twenty-seventh pass added a kind
        # (`sweep`) no word lane reads, so the measure is each kind's own
        # sections — the brief a worker reads — never the file.
        body = brief.sections(brief.FRAME.read_text())
        for kind, names in brief.KINDS.items():
            self.assertLessEqual(sum(len(body[n]) for n in names), brief.FRAME_CEILING, kind)
        # The reading kind (DECISIONS 84) is one new section on the shared
        # four, and the section stays a page.
        self.assertIn('reading', brief.KINDS)
        self.assertLessEqual(len(body['reading-item']), 1600)

    def test_every_kind_s_sections_are_in_the_frame(self):
        have = set(brief.sections(brief.FRAME.read_text()))
        for kind, names in brief.KINDS.items():
            self.assertEqual([n for n in names if n not in have], [], kind)


class ABriefIsComposed(unittest.TestCase):
    def compose(self, item, kind='residue', **kw):
        args = dict(queue=QUEUE, frame=brief.FRAME.read_text(), base='abc1234',
                    model='Opus 5.5', runs=['run423', 'run424'], sections=[], read=[], note='',
                    tests=['sahara_coverage::', 'floors::'])
        args.update(kw)
        return brief.compose(item, kind, **args)

    def test_a_sweep_or_chapter_brief_s_gate_takes_no_widen_and_a_residue_brief_s_does_not(self):
        # DECISIONS 83: the AI lane's gate widens to every timed test on a sim
        # path, because it moves pins; the other kinds leave the timed suite
        # to the booking gate, as item 1567 meant.
        queue = QUEUE.replace("1131. **Chapter thirty-eight's word", "1131. **The sweep's next batch")
        self.assertIn(' --no-widen`', self.compose(1131, 'sweep', queue=queue, tests=[]))
        self.assertIn(' --no-widen`', self.compose(1131, 'chapter'))
        self.assertNotIn('--no-widen', self.compose(1133))
        # A reading lane (DECISIONS 84) runs no capture and moves no pin
        # but the blind list: the fast suite and `blind::`, no widening.
        self.assertIn('--tests blind:: --no-widen`', self.compose(1131, 'reading', tests=[]))

    def test_a_note_s_lane_gate_without_tests_or_whole_is_refused(self):
        # Loop 3489: `--lane --item N` with no `--tests` is refused by the
        # gate (exit 2), so a note that briefs it hands the lane a gate that
        # cannot run, and the lane ran the booking gate instead.
        for bad in ('The gate: `python3 tools/release_gate.py ~/game --lane --item 3486`.',
                    'Gate with `python3 tools/release_gate.py ~/game --test-threads 4\n  --lane --item 3486`, backgrounded.',
                    'gate: python3 tools/release_gate.py ~/game --lane --item 3486'):
            with self.subTest(note=bad), self.assertRaises(ValueError) as caught:
                self.compose(1133, note=bad)
            self.assertIn('Loop 3489', str(caught.exception))
            self.assertIn('--whole', str(caught.exception))
        for good in ('The gate: `python3 tools/release_gate.py ~/game --lane --whole --item 3486`.',
                     'Gate with `python3 tools/release_gate.py ~/game --lane --item 3486\n  --tests poolshot::`.',
                     'No gate flag named here; a lane-gate is the frame\'s.'):
            with self.subTest(note=good):
                self.assertIn('## This item', self.compose(1133, note=good))

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

    def test_a_finding_parks_in_the_lane_s_own_journal(self):
        # DECISIONS 125: the parked file is gone. Every lane appended to it
        # and it grew to 1.49 MB; the brief sends a finding that names no
        # score to the item's own journal, with or without a slot letter.
        for text in (self.compose(1131, 'chapter', park='C'), self.compose(1131, 'chapter')):
            reserved = text.split('## Reserved for you')[1].split('\n## ')[0]
            self.assertIn('`- **1131a. <title>**`', reserved)
            self.assertIn('## Parked', reserved)
            self.assertIn('There is no parked file', reserved)
            self.assertNotIn('docs/PARKED.md', text)
            self.assertNotIn('park slot', reserved)

    def test_the_seams_are_read_for_the_lane(self):
        # Parked 1743: `seams.py` ran in one of nine word lanes' journals and
        # `standing.py` in none, though the frame's rows name both. The
        # brief carries `seams.py --item N`'s own print, so the step is done
        # before the lane reads it.
        text = self.compose(1133, seams=lambda item: f'SEAM a.rs:1 the thing item {item} lacks\n' * 3)
        section = text.split('## Seams, read for you')[1].split('\n## ')[0]
        self.assertIn('SEAM a.rs:1 the thing item 1133 lacks', section)
        self.assertIn('seams.py --item 1133', section)
        self.assertNotIn('## Seams, read for you', self.compose(1133, seams=None))
        long = self.compose(1133, seams=lambda item: '\n'.join(f'line {i}' for i in range(300)))
        section = long.split('## Seams, read for you')[1].split('\n## ')[0]
        self.assertIn('line 0', section)
        self.assertNotIn('line 299', section)
        self.assertIn('more lines', section)

    def test_a_seams_print_that_is_a_usage_error_is_said_in_one_line(self):
        # Loop 3537: `seams.py --item 3529` exited 2 on `no name given` and
        # the brief carried argparse's usage text as if it were the print;
        # 3540's brief carried it again. A non-zero exit is one line that
        # says so and names what to run.
        usage = 'usage: seams.py [-h] [--item ITEM]\nseams.py: error: no name given: pass names, --item'
        line = brief.seams_print(2, usage, 3529)
        self.assertNotIn('usage:', line)
        self.assertIn('exited 2', line)
        self.assertIn('no name given', line)
        self.assertIn('seams.py', line)
        self.assertEqual(brief.seams_print(0, 'SEAM a.rs:1 x\n', 3529), 'SEAM a.rs:1 x')
        self.assertEqual(brief.seams_print(0, '', 3529), '(seams.py printed nothing)')

    def test_a_chapter_s_rows_are_a_chapter_s(self):
        self.assertIn('issuer chapter', self.compose(1131, 'chapter'))
        self.assertNotIn('issuer chapter', self.compose(1133))

    def test_no_placeholder_survives(self):
        for kind, item in (('residue', 1133), ('chapter', 1131), ('reading', 1131)):
            text = self.compose(item, kind)
            self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])
            self.assertIn('abc1234', text)
            self.assertIn('Opus 5.5', text)
            self.assertIn(f'item-{item}.md', text)

    def test_coverage_is_always_among_a_word_s_filters(self):
        # Parked 1623: 1611's lane gate ran `sahara_coverage:: floors::
        # coverage_pair::` and its widening had put run718's window in the
        # coverage driver, whose two pins went red on the booking gate.
        text = self.compose(1133)
        self.assertIn('--tests sahara_coverage:: floors:: coverage::', text)
        self.assertNotIn('{tests}', text)
        with self.assertRaisesRegex(ValueError, '--tests'):
            self.compose(1133, tests=[])
        self.assertEqual(brief.filters('residue', ['coverage::', 'x::']),
                         ['coverage::', 'x::', 'first_parting', 'battery_floors', 'blind::',
                          'audit::every_whole_scoreboard_word_is_audited'])

    def test_every_lobby_s_first_parting_is_among_a_word_s_filters(self):
        # Parked 1910 and 2000 (the thirty-first pass): 1937's fix moved
        # Great Sahara and broke the coverage pair 4730 -> 4206, which took
        # a capture to read, and 1870's moved four coverage-pair widenings
        # no filter named; the lobbies' `first_parting` tests run together
        # in a dozen seconds, so a word's lane gate always runs them. A
        # sweep's does not: it moves no word.
        self.assertIn('first_parting', brief.filters('residue', ['x::']))
        self.assertIn('first_parting', brief.filters('chapter', ['golden::']))
        self.assertNotIn('first_parting', brief.filters('sweep', []))
        # And the held-out battery's floors (DECISIONS 77): a word lane's change can lower an unseen lobby.
        self.assertEqual(brief.filters('residue', ['first_parting', 'coverage::']),
                         ['first_parting', 'coverage::', 'battery_floors', 'blind::',
                          'audit::every_whole_scoreboard_word_is_audited'])
        self.assertIn('first_parting', self.compose(1133))

    def test_a_sweep_brief_has_its_section_and_no_booking(self):
        # Parked 1629: a fanned batch's brief says where its subagents'
        # worktrees are cut and what a stale shared target binary looks like.
        queue = QUEUE.replace('1131. **Chapter thirty-eight\'s word: frame 878, ours 7 draws against 5**\n'
                              '    (1113). No mechanism is named.',
                              '1131. **The sweep\'s third batch: ten never-backed simulation functions**\n'
                              '    (DECISIONS 63 (iv)). Packet-free first.')
        self.assertNotEqual(queue, QUEUE)
        text = self.compose(1131, 'sweep', queue=queue, tests=[])
        self.assertIn('## The sweep', text)
        self.assertIn('scaffold commit', text)
        self.assertIn('--tests sweep::', text)
        self.assertNotIn('The frame and the draw delta are the booking', text)
        self.assertNotIn('## Before any reading', text)
        self.assertIn('the sweep track', text)
        self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])

    def test_a_reading_brief_has_its_section_and_no_capture(self):
        # DECISIONS 84: a reading lane reads the decompile for the functions
        # its document cites and no trace enters, and writes claims that
        # each name their falsifying capture; it takes no capture, no packet
        # and no emulator run, so its brief carries none of those rows.
        text = self.compose(1131, 'reading', tests=[], runs=[])
        self.assertIn('## A reading item', text)
        self.assertIn('the reading track', text)
        section = text.split('## A reading item')[1].split('\n## ')[0]
        for word in ('NEVER', 'reading-only', 'falsif', '--tests blind:: --no-widen', 'loaders'):
            self.assertIn(word, section)
        for gone in ('## A capture or a packet', '## Before any reading', '## Reading the original',
                     '## Build', '## The sweep', '## Seams, read for you', 'waitrun.sh'):
            self.assertNotIn(gone, text)
        self.assertNotIn('## Seams, read for you',
                         self.compose(1131, 'reading', tests=[], runs=[], seams=lambda item: 'SEAM x'))
        self.assertIn('## Landing', text)
        self.assertLess(text.index('## A reading item'), text.index('## Landing'))
        self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])

    def test_a_sweep_brief_names_the_other_sweep_lane_s_worktree_for_claims(self):
        # Loop 2926 (the thirty-ninth pass): 2877 re-swept 2863's function
        # and 2910 re-swept 2900's two, a gate and a half each; the census
        # cannot see what a live lane is building, so the brief hands the
        # lane the other sweep lane's worktree for `--claimed`.
        queue = QUEUE.replace('1131. **Chapter thirty-eight\'s word: frame 878, ours 7 draws against 5**\n'
                              '    (1113). No mechanism is named.',
                              '1131. **The sweep\'s third batch: ten never-backed simulation functions**\n'
                              '    (DECISIONS 63 (iv)). Packet-free first.')
        queue = queue.replace('1133. **', '1133. **The sweep\'s fourth batch: ', 1)
        text = self.compose(1131, 'sweep', queue=queue, tests=[])
        self.assertIn('--claimed ' + brief.lane_worktree(1133), text)
        self.assertNotIn('att-1131', text)
        self.assertIn('--unbacked --claimed', text)
        residue = self.compose(1133, 'residue', queue=queue)
        self.assertNotIn('--claimed', residue)
        self.assertTrue(brief.lane_worktree(7).endswith('/att-7'))

    def test_a_filter_whose_only_matches_are_ignored_tests_is_refused(self):
        # Loop 3227: three briefs named `battery_measure`, an `#[ignore]`
        # test, and 3221's lane gate exited 2 on it.
        with self.assertRaises(ValueError) as e:
            brief.filters('residue', ['battery_measure'])
        self.assertIn('battery_measure', str(e.exception))
        self.assertIn('#[ignore]', str(e.exception))
        # A filter with a live match composes; one that matches no `fn` is
        # a module path (`coverage::`) and composes too.
        self.assertIn('battery_floors', brief.filters('residue', ['battery_floors']))
        self.assertIn('nations::', brief.filters('residue', ['nations::']))

    def test_a_filter_that_matches_no_test_is_refused(self):
        # Loop 3818: the sixty-fourth's commander named `map_resources` for
        # 3807 and 3811, a module with no tests, and 3811's first lane gate
        # exited 2 on it. A nonsense name is refused the same way.
        for t in ('map_resources', 'no_such_test_name_3818'):
            with self.assertRaises(ValueError) as e:
                brief.filters('residue', [t])
            self.assertIn(t, str(e.exception))
            self.assertIn('matches no test', str(e.exception))
        # A filter that matches a live test passes, by fn name or module path:
        # `map_lakes` names tests of `sim::map_lakes`, `golden::` a module.
        # A test a `macro_rules!` writes (`diff::audit`'s `audited!`) and one
        # a `#[path]` module holds are found too.
        for t in ('map_lakes', 'golden::', 'diff::harness::tests::', 'sweep::',
                  'audit::american_enlightenment_pair_corrections_change_nothing'):
            self.assertIn(t, brief.filters('residue', [t]))

    def test_a_sim_only_filter_is_noted_on_stderr_and_composes(self):
        # Loop 3823: `map_lakes` and `terrain_groups` are tests in
        # `crates/sim` alone, and on a lane that touches no sim path the
        # gate leaves that crate to the booking gate (Loop 3653) — 3819's
        # first lane gate exited 2 on them. A note, not a refusal: the
        # commander cannot know at booking whether the lane will touch sim.
        import contextlib
        import io
        err = io.StringIO()
        with contextlib.redirect_stderr(err):
            out = brief.filters('residue', ['mapgen', 'map_lakes', 'terrain_groups'])
        self.assertEqual(out[:3], ['mapgen', 'map_lakes', 'terrain_groups'])
        self.assertEqual(err.getvalue().splitlines(), [
            f'note: --tests {t} matches tests in crates/sim only; moot on a lane that touches no sim '
            'path (Loop 3823)' for t in ('map_lakes', 'terrain_groups')])
        self.assertEqual(brief.sim_only(['mapgen', 'coverage::', 'map_lakes'], brief.crate_tests()),
                         ['map_lakes'])

    def test_a_run_number_goes_through_runs_and_is_reserved_once(self):
        # Loop 3176 (the commander, at 3162's merge): two notes offered run
        # 923 "if needed" and a third brief reserved it with `--runs`; two
        # captures landed under one number. A note that names a run the
        # `--runs` list does not carry is refused, and a run another brief
        # on the box names is refused. Red first: both composed.
        with self.assertRaises(ValueError) as e:
            self.compose(1133, note='Take run 925 if the first capture is short.')
        self.assertIn('925', str(e.exception))
        self.compose(1133, note='Take run 423 if the first capture is short.')  # in --runs: fine
        others = {'brief1127.md': '## Reserved for you\n\n- **Run numbers**: run424.\n',
                  'brief1131.md': '- **Run numbers**: none; ask for one before a capture.\n'}
        with self.assertRaises(ValueError) as e:
            self.compose(1133, briefs=others)
        self.assertIn('run424', str(e.exception))
        self.assertIn('brief1127.md', str(e.exception))
        self.compose(1133, briefs={'brief1133.md': '- **Run numbers**: run423, run424.\n'})  # its own earlier brief
        self.compose(1133, briefs={'brief1127.md': '- **Run numbers**: run500.\n'})

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
            kind = 'chapter' if 'Chapter' in headline else 'sweep' if 'sweep' in headline.lower() else 'residue'
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


# Loop 4022: the structure tranche's and the renderer's items, in the queue's
# own words as 3e4ea444a booked them, which the commander briefed by hand.
QUEUE_TRACKS = QUEUE.replace('## How to maintain this file', """4008. **The guards and the shape counter** (a `structure` item; STRUCTURE §5.1): `no_float.rs` scans `crates/sim/src` recursively
(it reads the flat directory only and already misses `do_attack/live.rs`); `docs_guard`'s seam table and test-file list read nested
paths; a guard pins that every `crates/sim/src/…` path `docs/` cites exists (2,909 citations in 657 files) and `tools/paths.py`
rewrites citations when a file moves; `tools/shape.py` prints the `Structure:` line's six counts and `rondata::diff` parses the line
against them. Red first on the tree as it stands. Lane A, first; gate whole.

4018. **The camera, proven against the original's function** (a `render` item; RENDERER §6 item 2, §3.3): a pure camera and
projection in a new `render` crate, written from reading `BaseCamera::update_transforms` and `Camera::update_transforms`, never
transcribed; checked against `MiscAccess::world_to_screen` under unicorn on synthesized camera states (a sweep, as `crates/sim`
runs its sweeps; `tools/emu/callfn.py`) and on a packet on disk (grep the archive first: run144's is cited in FORMATS). Score:
camera states in register, K of the sweep, every point within one pixel. Needs nothing from `sim`. Lane B, first.

## How to maintain this file""")


class AStructureOrRenderItemHasItsKind(unittest.TestCase):
    """Loop 4022 (the seventy-fourth pass, DECISIONS 114 §4): `KINDS` knew
    residue, chapter, sweep, reading and coverage only, so the commander
    briefed 4008's structure lane and 4018's render lane by hand from
    `docs/STRUCTURE.md` §1 and `docs/RENDERER.md` §6. Red first: both
    composed `no such kind of brief`."""

    def compose(self, item, kind, **kw):
        args = dict(queue=QUEUE_TRACKS, frame=brief.FRAME.read_text(), base='abc1234',
                    model='Opus 5.5', runs=[], sections=[], read=[], note='', tests=[])
        args.update(kw)
        return brief.compose(item, kind, **args)

    def test_a_structure_brief_carries_the_rule_the_counter_and_a_whole_gate(self):
        text = self.compose(4008, 'structure', sections=['docs/STRUCTURE.md §5.1'])
        self.assertIn('the structure track', text)
        self.assertIn('4008. **The guards and the shape counter**', text)
        section = text.split('## A structure item')[1].split('\n## ')[0]
        for word in ('No behaviour change', 'every floor unchanged', 'python3 tools/shape.py',
                     '`Structure:`', 'random stream', 'docs/STRUCTURE.md'):
            self.assertIn(word, section)
        landing = text.split('## Landing')[1]
        self.assertIn('--lane --item 4008 --whole`', landing)
        self.assertNotIn('--tests', landing)
        for gone in ('## Before any reading', '## A capture or a packet', '## Build', '## The sweep',
                     '## Seams, read for you'):
            self.assertNotIn(gone, text)
        self.assertLess(text.index('## A structure item'), text.index('## Landing'))
        self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])
        # Gated whole: a filter named for it is refused, never silently dropped.
        with self.assertRaisesRegex(ValueError, '--whole'):
            self.compose(4008, 'structure', tests=['coverage::'])
        self.assertEqual(brief.filters('structure', []), [])

    def test_a_brief_names_the_method_for_the_kinds_that_use_it_and_the_driver(self):
        # The seventy-sixth pass (DECISIONS 116): `CLAUDE.md` keeps the long
        # middle's method in a sentence a rule and `docs/METHOD.md` the
        # clauses, so the kinds that chase a word are told to read it and a
        # structure or render lane is not. And the row a lane reports to is
        # `commander`, the driver since DECISIONS 126; `steer` is the pass.
        for kind in sorted(brief.KINDS):
            text = self.compose(4008, kind) if kind in brief.NO_WORD else \
                self.compose(4008, kind, tests=['coverage::'])
            named = '`docs/METHOD.md`' in text.split('## Read first')[1].split('\n## ')[0]
            self.assertEqual(named, kind in brief.METHOD_KINDS, kind)
            self.assertNotIn('`steer`', text, kind)
            self.assertIn('I am `commander`, the driver', text, kind)
        self.assertEqual(brief.METHOD_KINDS, {'residue', 'chapter', 'coverage', 'reading', 'sweep'})

    def test_a_render_brief_carries_the_counter_the_rungs_the_pins_and_a_scoped_gate(self):
        text = self.compose(4018, 'render', sections=['docs/RENDERER.md §3.4'])
        self.assertIn('the render track', text)
        self.assertIn('4018. **The camera, proven against the original', text)
        section = text.split('## A render item')[1].split('\n## ')[0]
        for word in ('K of M', 'docs/RENDERER.md', '§5.4', 'decode', 'in register', 'packet',
                     'camera_oracle.py', 'gpiece', 'value diff', 'item-4026.md',
                     'crates/rondata/src/free/tests.rs', 'RENDER_K', 'RENDER_M', 'FRAMES_IN_REGISTER',
                     'M_PIECES_DECODING', 'M_BUILDING_TYPES_DECODING', 'crates/rondata/src/artdata.rs',
                     '`PIECES_DECODING`', 'RON_INSTALL', '`uv`', 'skipping:', '--whole'):
            self.assertIn(word, section)
        landing = text.split('## Landing')[1]
        self.assertIn('--lane --item 4018 --tests free:: artdata::', landing)
        self.assertIn('coverage:: census::', landing)
        self.assertIn('--no-widen`', landing)
        for gone in ('## Before any reading', '## Build', '## The sweep', '## Seams, read for you'):
            self.assertNotIn(gone, text)
        self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])
        self.assertNotIn('first_parting', brief.filters('render', []))

    def test_every_kind_s_always_filter_matches_a_live_test(self):
        # The pass's render ruling named `render::`, which no test name
        # carries — cargo prints the render crate's tests crate-less
        # (`camera::tests::…`) — and the gate's `unmatched_filters` names
        # such a filter. A kind's own filters are held to `filters`'s rule.
        fns = brief.test_fns()
        for kind, always in brief.ALWAYS_TESTS.items():
            for t in always:
                self.assertTrue(any(t in n and not ign for n, ign in fns.items()), (kind, t))

    def test_the_new_kinds_are_kinds_and_each_section_is_a_page(self):
        body = brief.sections(brief.FRAME.read_text())
        for kind in ('structure', 'render'):
            self.assertIn(kind, brief.KINDS)
            self.assertIn(kind, body)
            self.assertLessEqual(len(body[kind]), 2400, kind)


# The steering pass of 2026-11-12: item 4034 in the queue's own words, and
# 4149's booking as eb7ca871e wrote it, an input item briefed as a render lane.
QUEUE_START_INPUT = QUEUE.replace('## How to maintain this file', """4034. **The start block, every lobby on disk** (filler behind the milestone; START.md): the nations' civ-specific buildings, the Large Town
list, the Americans' farms, the Lakota and Korean citizen arms, the Spanish scouts (ORDERS §9.3) — each lobby a row of the
counter, cheapest first, the value diff per lobby; `Start: N of N` closes the track's first number; rules N (parked 4033a).

4149. **The original plays a file minted by clicks at the opening camera** (the milestone's step 4, the person's half; a render lane's, its
capture the next run; `docs/CLIENT_INPUT.md` §2, §4; `docs/RECGAME.md` §9; parked 4145a): at the camera the client opens with one pixel covers 39 to 46
points and two script targets part by a few, so the script cannot be clicked there. Score: `INPUT_PLAYED`.

## How to maintain this file""")


class AStartOrInputItemHasItsKind(unittest.TestCase):
    """The steering pass of 2026-11-12: `KINDS` had no `start` and no `input`,
    so item 4034 (the start block on every lobby) could not be handed to a
    lane, and the client's input items (the orders track) were briefed as
    render lanes (4149). Red first: both composed `no such kind of brief`."""

    def compose(self, item, kind, **kw):
        args = dict(queue=QUEUE_START_INPUT, frame=brief.FRAME.read_text(), base='abc1234',
                    model='Opus 5.5', runs=[], sections=[], read=[], note='', tests=[])
        args.update(kw)
        return brief.compose(item, kind, **args)

    def test_a_start_brief_carries_the_counter_the_oracle_and_a_whole_gate(self):
        text = self.compose(4034, 'start', sections=['docs/START.md §5'])
        self.assertIn('the start track', text)
        self.assertIn('4034. **The start block, every lobby on disk**', text)
        section = text.split('## A start item')[1].split('\n## ')[0]
        for word in ('`Start: <built> of <N> lobbies`', 'docs/START.md', '§5', 'START_BUILT', 'START_LOBBIES',
                     'GAMEINFO', 'rondata::diff::start', 'delta in the comment', '`Start:` line is mine',
                     '[Start Game]', 'build_sim', 'RON_START_DIFF=1', '=all', 'tools/start_lobbies.py',
                     'No capture is booked', '§3', '`bucket`', '`ages`', '`techs`', 'quiet',
                     'dump-backed', 'census', 'first parting', 'value diff', '§1',
                     'check_all_level=14', 'grep the archive', 'whole'):
            self.assertIn(word, section)
        landing = text.split('## Landing')[1]
        self.assertIn('--lane --item 4034 --whole`', landing)
        self.assertNotIn('--tests', landing)
        for gone in ('## Before any reading', '## A capture or a packet', '## Build', '## The sweep',
                     '## Seams, read for you', '`docs/METHOD.md`'):
            self.assertNotIn(gone, text)
        self.assertLess(text.index('## A start item'), text.index('## Landing'))
        self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])
        # Gated whole: a filter named for it is refused, never silently dropped.
        with self.assertRaisesRegex(ValueError, '--whole'):
            self.compose(4034, 'start', tests=['coverage::'])
        self.assertEqual(brief.filters('start', []), [])

    def test_an_input_brief_carries_the_method_in_order_and_a_scoped_gate(self):
        text = self.compose(4149, 'input', sections=['docs/CLIENT_INPUT.md §6'])
        self.assertIn('the orders track', text)
        self.assertIn('4149. **The original plays a file minted by clicks', text)
        section = text.split('## An input item')[1].split('\n## ')[0]
        for word in ('docs/CLIENT_INPUT.md', 'field for field', 'world_to_screen', 'rondata::script',
                     '`script()`', '`clicked()`', 'INPUT_CLICKED', 'red once', 'recgame::minted_packages',
                     'round-tripped', 'docs/RECGAME.md', '§9.3', 'click-free', 'per-frame', '§5',
                     'value diff', 'Ours, not the original', 'never called a reading', 'No float', '§1',
                     '`INPUT_*`', '`*_WALKED`', '`Recording:` line is mine', '--whole', '`command`'):
            self.assertIn(word, section)
        # The method's six steps, in its order.
        steps = [section.index(w) for w in ('stage 0', 'rondata::script', 'INPUT_CLICKED',
                                            'recgame::minted_packages', '§9.3', 'effect')]
        self.assertEqual(steps, sorted(steps))
        landing = text.split('## Landing')[1]
        self.assertIn('--lane --item 4149 --tests input:: minted:: recgame:: coverage:: blind:: --no-widen`',
                      landing)
        self.assertLess(text.index('## An input item'), text.index('## A capture or a packet'))
        self.assertLess(text.index('## A capture or a packet'), text.index('## Landing'))
        for gone in ('## Before any reading', '## Build', '## The sweep', '## Seams, read for you',
                     '`docs/METHOD.md`'):
            self.assertNotIn(gone, text)
        self.assertEqual(re.findall(r'\{[a-z]+\}', text), [])
        self.assertNotIn('first_parting', brief.filters('input', []))

    def test_an_item_the_queue_does_not_book_is_refused_for_either_kind(self):
        for kind in ('start', 'input'):
            with self.assertRaisesRegex(ValueError, 'books no item 9999'):
                self.compose(9999, kind)

    def test_the_two_kinds_are_kinds_and_each_section_is_a_page(self):
        body = brief.sections(brief.FRAME.read_text())
        for kind in ('start', 'input'):
            self.assertIn(kind, brief.KINDS)
            self.assertIn(kind, body)
            self.assertLessEqual(len(body[kind]), 2400, kind)
            self.assertIn(kind, brief.NO_WORD)
            self.assertNotIn(kind, brief.METHOD_KINDS)
        self.assertIn('start', brief.WHOLE)
        self.assertNotIn('input', brief.WHOLE)
        self.assertEqual(brief.TRACKS['start'], 'the start track')
        self.assertEqual(brief.TRACKS['input'], 'the orders track')


class TheFrameNamesRowsTheWayCccDoes(unittest.TestCase):
    def test_every_ccc_verb_the_frame_spells_names_a_row_by_lane(self):
        # Loop 4023 (4021's journal): the frame's landing row spelt `ccc
        # update att-<n>`, the worktree's name; the roster keys rows by
        # `--name lane-<N>`, and `ccc update att-4021` answered "no session
        # in the roster". Red first on that row.
        frame = brief.FRAME.read_text()
        calls = re.findall(r'`ccc ([a-z]+)((?: --?[a-z-]+)*) ([^`\s]+)', frame)
        self.assertTrue(calls, 'the frame spells no ccc verb with a row')
        bad = [f'ccc {verb} {row}' for verb, _, row in calls if not row.startswith('lane-')]
        self.assertEqual(bad, [])
        # And the brief spells the lane's own row.
        text = brief.compose(1133, 'residue', queue=QUEUE, frame=frame, base='abc1234', model='Opus 5.5',
                             runs=[], sections=[], read=[], note='', tests=['floors::'])
        self.assertIn('`ccc update lane-1133`', text)
        self.assertNotIn('ccc update att-', text)


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
