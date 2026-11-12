"""The territory map's kinds and marks (item 4039; DECISIONS 112 §3 and
114 §6): a port first, then the census's excluded marks, then a layout
(defect A); a port is the census's own `is_port_backed`, so a twin spelled on
a code line of a fn by another name counts and a doc comment on a neighbour
still does not (Loop 2286/2420); the groups the readers proposed carry their
owner or reversal trigger; the start track's, the checksum track's and the
rules' 1:1 finish are named inside `open` and never moved out of it."""
import importlib.util
import pathlib
import unittest
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('census', ROOT / 'tools' / 'census.py')
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)


def kind(name, by_doc=(), by_sweep=(), port=(), spelled=()):
    return census.territory_kind(1, name, set(by_doc), set(by_sweep), set(port), set(spelled))


class TheKinds(unittest.TestCase):
    def test_a_swept_out_body_is_the_census_mark_not_a_layout(self):
        # defect A: `backed` was asked before the marks, so 82 swept bodies
        # the census excludes read `layout` and counted open
        self.assertEqual(kind('FarmsOut::get_name', by_sweep={1}), 'ui')
        self.assertEqual(kind('MiningList::clear', by_sweep={1}), 'dead')
        self.assertEqual(kind('TCoord::operator<', by_sweep={1}), 'op')

    def test_a_swept_body_with_no_mark_and_no_consumer_is_a_layout(self):
        self.assertEqual(kind('Unit::think', by_sweep={1}), 'layout')
        self.assertEqual(kind('Unit::think'), 'open')
        self.assertEqual(kind('Unit::think', port={'think'}), 'name')

    def test_a_port_stays_a_port_whatever_its_name_says(self):
        self.assertEqual(kind('GroupOut::issue_stance', by_doc={1}), 'port')
        self.assertEqual(kind('MiningList::clear', by_sweep={1}, spelled={'MiningList::clear'}), 'port')

    def test_a_twin_spelled_on_a_code_line_of_another_fn_is_a_port(self):
        # the census's own `port` column already counted it; the map asked
        # for a `fn` of the method's own name too, so a twin never did
        self.assertEqual(kind('Balance::return_modifier', by_sweep={1}, spelled={'Balance::return_modifier'}),
                         'port')

    def test_a_coverage_section_s_diff_is_a_port(self):
        self.assertEqual(kind('Guy::move', by_doc={1}), 'port')

    def test_a_doc_comment_on_a_neighbour_spells_nothing(self):
        text = ("/// `Balance::return_modifier@00581ca0`: the table's cell.\n"
                "pub fn pct_of(&self) -> i32 {\n    0\n}\n")
        self.assertNotIn('Balance::return_modifier', census.spelled_in(text))
        text = text.replace("    0\n}", "    0\n} // twin of Balance::return_modifier")
        self.assertIn('Balance::return_modifier', census.spelled_in(text))
        self.assertIn('Balance::return_modifier', census.spelled_in(
            "/// `Balance::return_modifier`\npub fn return_modifier() {}\n"))


class TheMarks(unittest.TestCase):
    def test_the_readers_rows_join_their_groups(self):
        for name, label in (('Game::loop', 'the turn pump'),
                            ('Types::unit_key', 'the XML loaders'),
                            ('Map::scale_number', 'the XML loaders'),
                            ('Balance::lookup_absolute_name', 'the XML loaders'),
                            ('Tribe::parse', 'the XML loaders'),
                            ('GameAccessConst::get_scary_console_who', 'the scripting VM'),
                            ('ObjectData::in_a_ship', 'the scripting VM'),
                            ('Game::run_playback', 'the recorded-game playback'),
                            ('RecordGame::read_header', 'the recorded-game playback')):
            self.assertEqual(census.replaced_group('x', name), label, name)
        for name in ('Game::loop_render', 'Types::unit_key_of', 'Type::set_name_x', 'Setup::build_game',
                     'World::walk_data', 'Map::make'):
            self.assertIsNone(census.replaced_group('x', name), name)

    def test_every_group_carries_its_owner_or_its_trigger(self):
        for label, _stems, _rule, owner in census.REPLACED_BY_DESIGN:
            self.assertTrue(owner.startswith(('reopened', 'owner')), label)
        owners = {label: owner for label, _s, _r, owner in census.REPLACED_BY_DESIGN}
        self.assertIn('client::input', owners['the command packaging and manager'])
        self.assertIn('networking', owners['the turn pump'])

    def test_the_tracks_and_the_fourth_mark_name_their_rows(self):
        self.assertEqual(census.named_group('Setup::build_game'), 'the start track')
        self.assertEqual(census.named_group('Setup::get_starting_citizens'), 'the start track')
        self.assertEqual(census.named_group('World::walk_data'), 'the checksum track')
        self.assertEqual(census.named_group('SubObject::must_walk'), 'the checksum track')
        self.assertEqual(census.named_group('Map::make'), "the rules' 1:1 finish")
        self.assertIsNone(census.named_group('Setup::init_wild_life'))
        self.assertIsNone(census.named_group('Unit::walk_data'))
        self.assertEqual(len(census.UNMODELLED), 66)

    def test_the_line_names_them_inside_open(self):
        totals = Counter(port=3, replaced=2, layout=1, name=1, open=4, dead=5, functions=16)
        totals["named:the start track"] = 2
        line = census.territory_line(totals)
        self.assertTrue(line.startswith("Territory: simulation ported 3 · replaced by design 2 · open 6 of 16 (in it: "),
                        line)
        self.assertIn("the start track 2 · the checksum track 0 · the rules' 1:1 finish 0", line)


if __name__ == '__main__':
    unittest.main()
