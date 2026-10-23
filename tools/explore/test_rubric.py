"""The rubric (`tools/rubric.py`, the forty-eighth pass): a steering pass's
scorecard by mechanic, by age and by unit type. Each table is fed small
fixtures here: a fake index for the first, a few dump lines for the other
two, CRLF as the original writes them."""
import importlib.util
import pathlib
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
spec = importlib.util.spec_from_file_location('rubric', ROOT / 'tools' / 'rubric.py')
rubric = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rubric)

SIM = 'e:\\agent\\_work\\2\\s\\main\\game\\'


class ByMechanic(unittest.TestCase):
    def rows(self):
        entries = [
            (0x1000, 'Farms::add', SIM + 'farms.cpp', 100),        # backed by doc
            (0x1010, 'Farms::grow', SIM + 'farms.h', 50),          # backed by sweep, the header
            (0x1020, 'Farms::~Farms', SIM + 'farms.cpp', 10),      # dead weight
            (0x2000, 'Wall::place', SIM + 'wall.cpp', 300),        # unmarked
            (0x2010, 'Wall::draw', SIM + 'wall.cpp', 40),          # ui
            (0x2020, 'Wall::operator==', SIM + 'wall.cpp', 5),     # operator
            (0x2030, 'Wall::mend', SIM + 'wall.cpp', 60),          # carried: the crate names it
            (0x3000, 'IfaceWin::show', SIM + 'ifacewin.cpp', 999), # interface, not counted
        ]
        cites = {0x1000: {'FARMS.md', 'journal/x.md'}, 0x2000: {'WALLS.md', 'QUEUE.md'}}
        return rubric.mechanic_rows(entries, {0x1000, 0x1010}, {0x1000}, {'mend'}, set(), cites)

    def test_a_header_sits_with_its_cpp_and_full_means_backed_or_exempt(self):
        rows = self.rows()
        self.assertEqual(set(rows), {'farms', 'wall'})
        f = rows['farms']
        self.assertEqual(f['files'], {'farms.cpp', 'farms.h'})
        self.assertEqual((f['functions'], f['backed'], f['doc'], f['sweep'], f['skip']), (3, 2, 1, 1, 1))
        self.assertTrue(f['full'])
        self.assertEqual((f['bytes'], f['backed_bytes'], f['unbacked_bytes']), (160, 150, 10))

    def test_the_rest_is_split_by_the_census_mark(self):
        w = self.rows()['wall']
        self.assertEqual((w['unmarked'], w['ui'], w['operator'], w['carried'], w['backed']), (1, 1, 1, 1, 0))
        self.assertFalse(w['full'])
        self.assertEqual(w['unbacked_bytes'], 405)

    def test_the_docs_cell_names_mechanic_documents_and_counts_the_rest(self):
        rows = self.rows()
        self.assertEqual(rubric.docs_cell(rows['farms']['docs']), 'FARMS.md +1')
        self.assertEqual(rubric.docs_cell(rows['wall']['docs']), 'WALLS.md +1')

    def test_the_summary_counts_the_full_groups(self):
        s = rubric.mechanics_summary(self.rows())
        self.assertEqual((s['groups'], s['full'], s['backed'], s['functions'], s['unmarked']), (2, 1, 2, 7, 1))


DUMP = (b" BEGIN UNITDATA\r\n  BEGIN GUY\r\n   type 69\r\n  BEGIN GUY\r\n   type 69\r\n"
        b" leader_flags 7\r\n BEGIN LEADERDATA\r\n  who 0\r\n  tribe 4\r\n"  # start: no ages_get
        b" leader_flags 19\r\n BEGIN LEADERDATA\r\n  who 1\r\n"
        b" BEGIN FRAME 10\r\n"
        b" leader_flags 7\r\n BEGIN LEADERDATA\r\n  who 0\r\n  ages_get() 2\r\n  ages_get() 9\r\n"
        b" leader_flags 19\r\n BEGIN LEADERDATA\r\n  who 1\r\n  ages_get() 1\r\n"
        b" leader_flags 0\r\n BEGIN LEADERDATA\r\n  who 2\r\n  ages_get() 0\r\n"
        b"  BEGIN GUY\r\n   type 50\r\n"
        b" BEGIN FRAME 11\r\n"
        b" leader_flags 7\r\n BEGIN LEADERDATA\r\n  who 0\r\n  ages_get() 2\r\n")


class ByAgeAndType(unittest.TestCase):
    def test_a_block_takes_the_flags_written_before_it_and_its_first_age(self):
        leaders, types = rubric.scan(DUMP)
        self.assertEqual(leaders, [(10, 0, 7, 2), (10, 1, 19, 1), (10, 2, 0, 0), (11, 0, 7, 2)])
        self.assertEqual(types, {69: 2, 50: 1})

    def test_frames_count_per_age_and_an_inactive_slot_counts_nothing(self):
        import os
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, 'gamelog-run1-x.txt')
            with open(p, 'wb') as f:
                f.write(DUMP)
            per_age, types, read, rows, other = rubric.walk([p])
        self.assertEqual((read, rows, dict(other)), (1, 4, {}))
        self.assertEqual(len(per_age[2]['human']), 2)  # frames 10 and 11
        self.assertEqual(len(per_age[1]['ai']), 1)
        self.assertEqual(len(per_age[0]['any']), 0)    # who 2's slot is inactive
        self.assertEqual(sum(1 for a in range(8) if not per_age[a]['any']), 6)

    def test_a_trace_finds_its_dumps_by_run_stem(self):
        import os
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            for n in ('gamelog-run16-attrition.txt', 'gamelog-run16b-cmd.txt', 'gamelog-run7.txt'):
                open(os.path.join(d, n), 'w').close()
            got = {t: [os.path.basename(p) for p in fs]
                   for t, fs in rubric.dumps_for(['rontrace-run16.log', 'rontrace-run7.log',
                                                  'rontrace-run9.log'], d)}
        self.assertEqual(got, {'rontrace-run16.log': ['gamelog-run16-attrition.txt'],
                               'rontrace-run7.log': ['gamelog-run7.txt'], 'rontrace-run9.log': []})

    def test_unit_rules_index_from_fifty_and_the_types_dump_gives_ages(self):
        xml = ('<ROOT><UNIT><NAME>Citizen</NAME><GRAPH>CITIZENS</GRAPH><DOMAIN>Land</DOMAIN>'
               '<CAT>Civilian</CAT></UNIT><UNIT><NAME>Galley</NAME><DOMAIN>Sea</DOMAIN></UNIT></ROOT>')
        self.assertEqual(rubric.unit_rules(xml), [(50, 'Citizen', 'CITIZENS', 'Land', 'Civilian'),
                                                  (51, 'Galley', '', 'Sea', '')])
        tdump = (b"  BEGIN UNITTYPE\r\n   BEGIN OBJECTTYPE\r\n    BEGIN TYPE\r\n     type 50\r\n"
                 b"    age 0\r\n  BEGIN UNITTYPE\r\n    BEGIN TYPE\r\n     type 51\r\n    age 2\r\n"
                 b"  BEGIN UNITTYPE\r\n     type 50\r\n    age 7\r\n")
        self.assertEqual(rubric.loaded_ages(tdump), {50: 0, 51: 2})


if __name__ == '__main__':
    unittest.main()


class TheLedger(unittest.TestCase):
    """The forty-ninth pass (Ramon): the rubric is shown at every steering
    pass against the pass before, never computed per landing — a row a pass
    in `docs/audit/rubric.tsv`, and `--delta` the table."""

    def row(self, label, held=(0, 1), seen=30):
        mech = dict(full=36, groups=112, backed=1696, functions=3611, byte_share=37.94)
        per_age = {x: {"any": x in held} for x in range(8)}
        return rubric.ledger_row('2026-10-21', label, mech, per_age, seen, 364, 58)

    def test_a_row_names_the_ages_never_held(self):
        r = self.row('the forty-eighth pass')
        self.assertEqual((r['ages_held'], r['ages_never'], r['byte_share']), (2, '2,3,4,5,6,7', 37.9))
        self.assertEqual(self.row('x', held=range(8))['ages_never'], 'none')

    def test_the_ledger_round_trips_and_the_delta_reads_the_last_two(self):
        import tempfile, os
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'rubric.tsv')
            self.assertEqual(rubric.read_ledger(path), [])
            rubric.append_ledger(path, self.row('the forty-eighth pass'))
            rubric.append_ledger(path, self.row('the forty-ninth pass', held=range(8), seen=70))
            rows = rubric.read_ledger(path)
            self.assertEqual([r['pass'] for r in rows], ['the forty-eighth pass', 'the forty-ninth pass'])
            self.assertEqual(rows[1]['types_seen'], 70)
            table = rubric.delta_table(rows[0], rows[1])
        self.assertIn('the forty-ninth pass against the forty-eighth pass', table)
        line = [l for l in table.splitlines() if l.startswith('unit types seen')][0]
        self.assertEqual(line.split()[-2:], ['70', '30'])
        line = [l for l in table.splitlines() if l.startswith('ages never held')][0]
        self.assertEqual(line.split()[-2:], ['none', '2,3,4,5,6,7'])

    def test_a_first_row_has_no_earlier_pass(self):
        table = rubric.delta_table(None, self.row('the forty-ninth pass'))
        self.assertIn('against no earlier pass', table)
        self.assertTrue(all(l.split()[-1] == '-' for l in table.splitlines()[2:]))


# --- the lockstep definition (the fifty-first pass, Ramon 2026-10-22) -------

WORD_MODULE = """
pub(crate) const WHOLE: Lobby = Lobby {
    name: "GreatLakesSpanishTier4",
    map_style: 14,
    start: "gamelog-run1008-greatlakes-spanish-diff4-start.txt",
    long: (
        "gamelog-run1008b-greatlakes-spanish-diff4-24k-trace.txt",
        "rontrace-run1008b.log",
    ),
    length: 14068,
    // a comment with a } brace and a run999 in it
    count: 14_068,
    sequence: 14068,
};
pub(crate) const WHOLE_WORD: i64 = if WHOLE.count < WHOLE.sequence {
    WHOLE.count
} else {
    WHOLE.sequence
};
pub(crate) const BATTERY: &[Lobby] = &[
    Lobby {
        name: "GreatSaharaRussianTier4",
        start: "gamelog-run500-greatsahara-british-start.txt",
        long: (
            "gamelog-run981b-greatsahara-russian-diff4-24k-trace.txt",
            "rontrace-run981b.log",
        ),
        length: 18883,
        count: 18883,
        sequence: 18000,
    },
];
"""

TESTKIT = """
/// ... ours and the original spend the same draws on every frame of
/// run54's trace, 0..23999, and the word is the trace's last frame.
pub(crate) const LONG_WORD_EAST_INDIES: i64 = 24_000;
/// the same draws on every frame of run53's trace, 0..9999.
pub(crate) const LONG_WORD_GREAT_LAKES: i64 = 9_000;
pub(crate) const SECOND_WORD_EAST_INDIES: i64 = 18_140;
pub(crate) const THIRD_WORD: i64 = 100;
pub(crate) const AI_WORDS: &[AiWord] = &[
    AiWord {
        line: "Battery word",
        map: "GreatLakesSpanishTier4",
        word: crate::diff::spanish_lakes_word::WHOLE_WORD,
        length: 14_068,
        endpoint: Some("run1008b_spanish_lakes_closing_state"),
    },
    AiWord {
        line: "Second pair",
        map: "EastIndies",
        word: SECOND_WORD_EAST_INDIES,
        length: 18_140,
        endpoint: Some("run346_is_east_indies_at_toughest_and_its_word_holds"),
    },
    AiWord {
        line: "Third map",
        map: "Open",
        word: THIRD_WORD,
        length: 200,
        endpoint: Some("run470_open_closing_state"),
    },
];
"""

BLIND = """
pub const TRACES: &[&str] = &[
    "rontrace-run29.log",
    // Item 3418: Great Sahara (run470's lobby) at `cover=1` to 15432
    // (`docs/RUNS.md` run1017), run1008b's game on every frame.
    "rontrace-run1017.log",
    // Item 3431: run470's lobby at `cover=1`, its `cover=0` twin's game
    // (run1040, run1042) on every frame.
    "rontrace-run1041.log",
    "rontrace-run1043.log",
    // Item 3423: two retakes: the Mongols (run834's and run346's game on
    // every frame), and the Dutch (run995's).
    "rontrace-run1022.log",
    "rontrace-run1024.log",
    // Item 3446: a carrier staged by console from 600; run1008b's game on
    // every frame before 603 (rngcmp 603 identical).
    "rontrace-run1066.log",
    // Item 3455: run1008b's game before 601.
    "rontrace-run1076.log",
];
"""


class Lockstep(unittest.TestCase):
    """A dump counts only when its game's word stands at the game's full
    length (Ramon, 2026-10-22): the runs come from the Rust consts."""

    def games(self):
        texts = {'spanish_lakes_word.rs': WORD_MODULE, 'testkit.rs': TESTKIT}
        return rubric.lockstep_games(texts, BLIND)

    def by(self):
        games, _unresolved = self.games()
        return {g['name']: g for g in games}

    def test_a_struct_is_whole_when_both_partings_reach_its_length(self):
        by = self.by()
        self.assertTrue(by['GreatLakesSpanishTier4']['whole'])
        self.assertEqual(by['GreatLakesSpanishTier4']['word'], 14068)
        self.assertFalse(by['GreatSaharaRussianTier4']['whole'])  # 18000 of 18883
        # A start dump of another lobby is a sibling, not the game.
        self.assertEqual(by['GreatSaharaRussianTier4']['runs'], ['run981b'])
        self.assertEqual(by['GreatLakesSpanishTier4']['runs'], ['run1008', 'run1008b'])

    def test_a_word_row_without_a_struct_takes_its_endpoint_s_run(self):
        games, _ = self.games()
        by = {g['name']: g for g in games}
        self.assertEqual((by['EastIndies']['runs'], by['EastIndies']['whole']), (['run346'], True))
        self.assertEqual((by['Open']['runs'], by['Open']['whole']), (['run470'], False))
        # The AI_WORDS row that is the struct's word is not a second game.
        self.assertEqual(sum(1 for g in games if g['name'] == 'GreatLakesSpanishTier4'), 1)

    def test_a_long_word_takes_its_run_and_length_from_its_comment(self):
        by = self.by()
        self.assertEqual((by['LONG_WORD_EAST_INDIES']['runs'], by['LONG_WORD_EAST_INDIES']['length'],
                          by['LONG_WORD_EAST_INDIES']['whole']), (['run54'], 24000, True))
        self.assertFalse(by['LONG_WORD_GREAT_LAKES']['whole'])  # 9000 of 10000

    def test_a_pinned_trace_joins_when_its_comment_names_a_whole_game(self):
        games, _ = self.games()
        runs = rubric.lockstep_runs_of(games)
        self.assertIn('run1017', runs)       # run1008b's game
        self.assertIn('run1022', runs)       # run346's game, the block's
        self.assertIn('run1024', runs)       # same comment block
        self.assertNotIn('run1041', runs)    # its twin is no word
        self.assertNotIn('run1066', runs)    # the game only before its staging
        self.assertNotIn('run1076', runs)
        self.assertNotIn('run29', runs)      # no comment
        self.assertNotIn('run981b', runs)    # not whole
        self.assertNotIn('run470', runs)     # not whole

    def test_the_dumps_are_the_whole_games_own_and_a_refused_take_is_not(self):
        import os
        import tempfile
        games, _ = self.games()
        with tempfile.TemporaryDirectory() as d:
            for n in ('gamelog-run1008-greatlakes-spanish-diff4-start.txt',
                      'gamelog-run1008b-greatlakes-spanish-diff4-24k-trace.txt',
                      'gamelog-run346-islands-toughest-24k-trace.txt',
                      'gamelog-run346-take1-diff0-refused.txt',
                      'gamelog-run1017-greatsahara-window.txt', 'gamelog-run981b-x.txt',
                      'rontrace-run1008b.log', 'rontrace-run1017.log', 'rontrace-run346.log'):
                open(os.path.join(d, n), 'w').close()
            dumps, traces = rubric.lockstep_files(games, d)
            dumps = sorted(os.path.basename(p) for p in dumps)
            traces = sorted(os.path.basename(p) for p in traces)
        self.assertEqual(dumps, ['gamelog-run1008-greatlakes-spanish-diff4-start.txt',
                                 'gamelog-run1008b-greatlakes-spanish-diff4-24k-trace.txt',
                                 'gamelog-run1017-greatsahara-window.txt',
                                 'gamelog-run346-islands-toughest-24k-trace.txt'])
        self.assertEqual(traces, ['rontrace-run1008b.log', 'rontrace-run1017.log', 'rontrace-run346.log'])


class LockstepBytes(unittest.TestCase):
    """Table 4: of the simulation layer's functions, the share a lockstep
    game's trace entered, by count and by bytes, through the census's own
    `entered_rows`."""

    def test_the_share_counts_simulation_functions_only(self):
        sizes = {1: 100, 2: 300, 3: 600, 4: 5000}
        source = {1: SIM + 'farms.cpp', 2: SIM + 'farms.cpp', 3: SIM + 'wall.cpp',
                  4: SIM + 'ifacewin.cpp'}
        s = rubric.lockstep_share(sizes, source, {2, 3, 4})
        self.assertEqual((s['functions'], s['entered'], s['bytes'], s['entered_bytes']), (3, 2, 1000, 900))
        self.assertEqual(s['bytes_tenths'], 900)
        self.assertEqual(rubric.tenths(900), '90.0')
        self.assertEqual(rubric.tenths(5), '0.5')
        self.assertEqual(rubric.lockstep_share(sizes, source, set())['bytes_tenths'], 0)


class SkirmishTypes(unittest.TestCase):
    """The denominator: a record no nation's tribe bit names
    (`TRIBE_MASK` all zero, so `LeaderData::tribe_can_type` answers 0 for
    every player, `docs/TECH.md`) and that the loader does not file as
    gaia (loaded `age` -1) is out of a skirmish's reach."""

    def test_a_zero_mask_record_is_excluded_unless_it_is_gaia(self):
        xml = ('<UNIT><NAME>Citizen</NAME><TRIBE_MASK>111111101111111111111111</TRIBE_MASK></UNIT>'
               '<UNIT><NAME>General</NAME><GRAPH>GENERALCTWE</GRAPH>'
               '<TRIBE_MASK>000000000000000000000000</TRIBE_MASK></UNIT>'
               '<UNIT><NAME>Wild Bird</NAME><TRIBE_MASK>000000000000000000000000</TRIBE_MASK></UNIT>'
               '<UNIT><NAME>Odd</NAME></UNIT>')
        self.assertEqual(rubric.no_tribe_types(xml, {50: 0, 51: 2, 52: -1, 53: 0}), [51])
        # Without the types dump nothing is called gaia, and the mask alone decides.
        self.assertEqual(rubric.no_tribe_types(xml, {}), [51, 52])


class TheLedgerColumns(unittest.TestCase):
    def test_an_older_row_reads_with_empty_new_cells(self):
        import os
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'rubric.tsv')
            with open(path, 'w') as f:
                f.write('# header\n')
                f.write('date\tpass\tgroups_full\tgroups\tbacked\tfunctions\tbyte_share\tages_held\t'
                        'ages_never\ttypes_seen\ttypes\tdumps\n')
                f.write('2026-10-22\tthe fiftieth pass\t36\t112\t1696\t3611\t37.9\t8\tnone\t74\t364\t112\n')
            mech = dict(full=36, groups=112, backed=1696, functions=3611, byte_share=37.94)
            per_age = {x: {"any": True} for x in range(8)}
            row = rubric.ledger_row('2026-10-22', 'lockstep', mech, per_age, 40, 300, 50,
                                    source='lockstep', lockstep=dict(entered=900, functions=3611,
                                                                     bytes_tenths=123))
            rubric.append_ledger(path, row)
            rows = rubric.read_ledger(path)
            with open(path) as f:
                lines = f.read().splitlines()
        self.assertEqual(lines[0], '# header')
        self.assertEqual(sum(1 for l in lines if l.startswith('date')), 1)
        self.assertEqual(lines[1].split('\t'), list(rubric.LEDGER_COLUMNS))
        self.assertEqual(len(lines[2].split('\t')), len(rubric.LEDGER_COLUMNS))  # padded
        self.assertIsNone(rows[0]['lockstep_bytes'])
        self.assertEqual(rows[0]['types_seen'], 74)
        self.assertEqual(rows[1]['lockstep_bytes'], 12.3)
        self.assertEqual(rows[1]['types_source'], 'lockstep')
        table = rubric.delta_table(rows[0], rows[1])
        line = [l for l in table.splitlines() if l.startswith('lockstep bytes')][0]
        self.assertEqual(line.split()[-2:], ['12.3', '-'])
