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
