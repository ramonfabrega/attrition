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
        # item 4170: every walk of state is the checksum track's, the
        # readers' `walk` class; a walk that is not a data walk is not
        self.assertEqual(census.named_group('Unit::walk_data'), 'the checksum track')
        self.assertEqual(census.named_group('Balance::walk_rules_data'), 'the checksum track')
        self.assertIsNone(census.named_group('StripBuilder::walk_strip'))
        self.assertEqual(len(census.UNMODELLED), 66)

    def test_the_line_names_them_inside_open(self):
        totals = Counter(port=3, replaced=2, layout=1, name=1, open=4, dead=5, functions=16)
        totals["named:the start track"] = 2
        line = census.territory_line(totals)
        self.assertTrue(line.startswith("Territory: simulation ported 3 · replaced by design 2 · open 6 of 16 (in it: "),
                        line)
        self.assertIn("the start track 2 · the checksum track 0 · the rules' 1:1 finish 0", line)


class TheReadersClasses(unittest.TestCase):
    """Item 4170: the readers' classes, each landed only where no open row
    it reaches was ruled a twin, absorbed, a differing twin or unmodelled."""

    def test_the_renderers_bodies_are_ui(self):
        for name in ('Border::build_border', 'Borders::process_all', 'StripBuilder::walk_strip',
                     'TerrainVis::squeeze_cache', 'WorldMapFog::fill_colors', 'Surf::add_wave',
                     'GuyData::get_render_x', 'Nuke::on_field_change', 'Tribe::get_name',
                     'Game::learn_as_you_play_popup', 'Game::action_screen_shot', 'Game::get_render_mode'):
            self.assertTrue(census.is_ui_body(name), name)

    def test_the_editor_ctw_the_shell_and_multiplayer_are_cut(self):
        for name in ('Signposts::add', 'Group::dbg_pack_or_unpack', 'Mountains::verify_template',
                     'MapConquest::make_continents', 'ReinforcementArmy::place_army', 'UnitBalance::write_header',
                     'Game::run_campaign', 'SyncPoint::sync', 'Game::OnFindLeaderboard',
                     'Player::is_connection', 'std::pair<a,b>::pair'):
            self.assertTrue(census.is_cut(name), name)

    def test_the_conflicts_that_narrowed_them_stay_unmarked(self):
        # ported: a `Doober::` and a `gpiece` class would have taken them
        for name in ('Doober::inc_time', 'Doober::has_doobers', 'Unit::update_gpiece', 'UnitData::get_gpiece'):
            self.assertFalse(census.is_ui_body(name), name)
        # a differing twin: `dbg_` is two names, never a prefix
        self.assertFalse(census.is_cut('Unit::dbg_jump_to_action'))
        # the rules beside the renderers
        for name in ('Unit::think', 'Map::make', 'Game::is_nomad', 'Player::is_alive'):
            self.assertFalse(census.is_ui_body(name) or census.is_cut(name), name)


def pe(sections):
    """A PE32 image, base 0x400000, of `(name, rva, bytes)` sections."""
    import struct
    head = bytearray(0x400)
    struct.pack_into('<I', head, 0x3C, 0x80)
    head[0x80:0x84] = b'PE\0\0'
    struct.pack_into('<HH', head, 0x84, 0x14C, len(sections))
    struct.pack_into('<H', head, 0x80 + 20, 0xE0)
    struct.pack_into('<I', head, 0x80 + 24 + 28, 0x400000)
    raw, out = 0x400, bytearray()
    for i, (name, rva, data) in enumerate(sections):
        o = 0x80 + 24 + 0xE0 + 40 * i
        head[o:o + 8] = name.encode().ljust(8, b'\0')
        struct.pack_into('<IIII', head, o + 8, len(data), rva, len(data), raw + len(out))
        out += data
    return bytes(head) + bytes(out)


class TheDeadScan(unittest.TestCase):
    """Item 4170: a function nothing in the image reaches is dead; a call, a
    jump, a vtable's pointer or an RVA anywhere is a reference."""

    def scan(self, image, targets, starts):
        import os
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, 'image.exe')
            with open(path, 'wb') as f:
                f.write(image)
            return census.unreferenced(path, targets, starts)

    def test_only_the_unreached_is_dead(self):
        import struct
        # .text at 0x401000: A (0x401000) calls B (0x401010); C (0x401020)
        # is only a vtable's slot; D (0x401030) only an RVA in .data; E
        # (0x401040) nothing; F (0x401050) only `.reloc` bytes; G (0x401060)
        # calls itself.
        text = bytearray(0x80)
        text[0:5] = b'\xe8' + struct.pack('<i', 0x401010 - 0x401005)
        text[0x60:0x65] = b'\xe8' + struct.pack('<i', 0x401060 - 0x401065)
        rdata = struct.pack('<II', 0x401020, 0)
        data = struct.pack('<I', 0x401030 - 0x400000)
        reloc = struct.pack('<I', 0x401050)
        image = pe([('.text', 0x1000, bytes(text)), ('.rdata', 0x2000, rdata), ('.data', 0x3000, data),
                    ('.reloc', 0x4000, reloc)])
        starts = [0x401000, 0x401010, 0x401020, 0x401030, 0x401040, 0x401050, 0x401060]
        dead = self.scan(image, set(starts[1:]), starts)
        self.assertEqual(dead, {0x401040, 0x401050, 0x401060})

    def test_a_dead_function_is_dead_but_a_port_stays_a_port(self):
        self.assertEqual(census.territory_kind(1, 'Unit::think', set(), {1}, set(), set(), unref={1}), 'dead')
        self.assertEqual(census.territory_kind(1, 'Unit::think', set(), {1}, set(), {'Unit::think'}, unref={1}),
                         'port')


if __name__ == '__main__':
    unittest.main()
