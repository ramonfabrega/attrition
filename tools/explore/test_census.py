"""The census reads the traces the blind list pins (parked 938, 939, 942).

Its recipe named three traces for nine days while eighty-six more stood
beside them, so the entered column was a sample (7,180 of 7,666, item 923);
and the recipe that replaced it, a `$(for …)` over run names, splits on the
spaces in this machine's archive path and has never run as written (item
935). `--pin` reads `rondata::blind::TRACES` from the source and joins each
name to the archive directory itself, so the recipe is one flag and the
list is the pin's.
"""
import importlib.util
import os
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('census', ROOT / 'tools/census.py')
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)


class PinnedTraces(unittest.TestCase):
    def test_the_list_is_the_pin_s_in_the_pin_s_order(self):
        names = census.pinned_traces()
        source = (ROOT / 'crates/rondata/src/blind.rs').read_text()
        body = source[source.index('pub const TRACES'):]
        body = body[:body.index('\n];')]
        self.assertEqual(names, re.findall(r'"(rontrace-[^"]+\.log)"', body))
        self.assertEqual(names[0], 'rontrace-run29.log')
        self.assertGreaterEqual(len(names), 46)
        self.assertEqual(len(names), len(set(names)))

    def test_a_comment_that_names_a_trace_is_not_a_trace(self):
        text = ('pub const TRACES: &[&str] = &[\n'
                '    "rontrace-run29.log",\n'
                '    // "rontrace-run53.log" was the recipe\'s, and enters nothing run29 does not\n'
                '    "rontrace-run16.log", // ch1\n'
                '];\n')
        with tempfile.TemporaryDirectory() as tmp:
            pin = Path(tmp) / 'blind.rs'
            pin.write_text(text)
            self.assertEqual(census.pinned_traces(pin),
                             ['rontrace-run29.log', 'rontrace-run16.log'])

    def test_an_archive_path_with_spaces_is_one_path_a_trace(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / 'Microsoft Games' / 'Rise of Nations' / 'Logs'
            archive.mkdir(parents=True)
            for name in ('rontrace-run29.log', 'rontrace-run16.log'):
                (archive / name).touch()
            with patch.object(census, 'pinned_traces',
                              return_value=['rontrace-run29.log', 'rontrace-run16.log']):
                logs = census.pinned_logs(str(archive))
            self.assertEqual(logs, [str(archive / 'rontrace-run29.log'),
                                    str(archive / 'rontrace-run16.log')])

    def test_a_pinned_trace_that_is_not_on_disk_is_named(self):
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(census, 'pinned_traces', return_value=['rontrace-run29.log']), \
                self.assertRaises(SystemExit) as stop:
            census.pinned_logs(tmp)
        self.assertIn('rontrace-run29.log', str(stop.exception))

    def test_an_operator_s_citation_is_counted(self):
        # Parked 1722: the pattern could not spell `operator<` or
        # `operator[]`, so 1708 cited `tcoord::operator@00472b30` as a
        # workaround and every operator was uncountable as backed.
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'X.md').write_text(
                '`TCoord::operator<@00472b30` and `Objects::operator[]@00512340` '
                'and `Unit::operator=@00612340`, and `plain@00401000`.\n')
            self.assertEqual(census.cited_in(tmp), {0x472b30, 0x512340, 0x612340, 0x401000})

    def test_a_sweep_chunk_citing_a_callee_its_script_hooks_is_named(self):
        # Parked 2012, Loop 2025: 1998's first draft cited the callee its
        # script hands `Harness(exe, {ADDR: …})`, and the census read 461 for
        # ten. The audit names the chunk, the script and the address; a
        # chunk that cites only what it sweeps is clean. Made to fail first
        # on the pair below, before `hooked_in_sweeps` existed.
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp, 'src'); emu = Path(tmp, 'emu')
            src.mkdir(); emu.mkdir()
            Path(emu, 'sweep_a_b.py').write_text(
                'from hooks import Harness\nh = Harness(argv[1], {0x00512340: ("Objects::at", 1, 0)})\n')
            Path(emu, 'sweep_c_d.py').write_text('h = Harness(argv[1], {})\n')
            Path(src, 'a.rs').write_text(
                '/// `A::b@00401000`, run by `tools/emu/sweep_a_b.py` under unicorn;\n'
                '/// its callee `Objects::at@00512340` is answered by the script.\n'
                '#[test]\nfn x() { let _ = 1; }\n'
                '/// `C::d@00402000`, run by `tools/emu/sweep_c_d.py` under unicorn.\n'
                '#[test]\nfn y() { let _ = 2; }\n')
            self.assertEqual(census.hooked_in_sweeps(str(src), str(emu)),
                             [('a.rs', 'sweep_a_b.py', 0x512340)])
        # Item 2341: an `obj` sweep's script hands `Harness` a comprehension
        # over `STUBS = [('call', ADDR, N), …]` (ADDR decimal), not a literal
        # `{0x…: …}`, and a chunk cites a callee by name as often as by
        # address — `AnimalData::get_speed`'s test named `UnitData::speed`,
        # its stub 0, and the census read 814 for ten. Made to fail first.
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp, 'src'); emu = Path(tmp, 'emu')
            src.mkdir(); emu.mkdir()
            Path(emu, 'sweep_e_f.py').write_text(
                "STUBS = [('call', 5317440, 1), ('slot', 20, 0)]\n"
                'h = Harness(argv[1], {a: (f"{a:x}", n) for k, a, n in STUBS if k == "call"})\n')
            Path(src, 'e.rs').write_text(
                '/// `E::f@00401000`, run by `tools/emu/sweep_e_f.py` under unicorn.\n'
                '#[test]\nfn z() { /* `a[1]` is `Objects::at`\'s answer */ }\n')
            by_name = {'E::f': [0x401000], 'Objects::at': [0x512340]}
            self.assertEqual(census.hooked_in_sweeps(str(src), str(emu), by_name),
                             [('e.rs', 'sweep_e_f.py', 0x512340)])
            # Without the names only the address citations are read.
            self.assertEqual(census.hooked_in_sweeps(str(src), str(emu)), [])
        # The tree itself, measured at the thirty-second pass: none of the
        # 501 landed sweeps cites a callee its script hooks.
        self.assertEqual(census.hooked_in_sweeps(str(ROOT / 'crates/sim/src'), str(ROOT / 'tools/emu')), [])

    def test_the_hooked_audit_exits_red_on_an_address_row_alone(self):
        # Loop 3312 (the forty-fifth pass): `--hooked` exited 1 at 62, 72, 89
        # and 109 chunks across four passes, and every row was the name
        # form — a port's comment naming the callee its stub answers for,
        # which `backed` already subtracts (the thirty-eighth pass, above).
        # The address form is pinned at zero on the tree (the test above);
        # the audit's exit is red on that form alone, and the name count is
        # printed as what it is. Made to fail first.
        rows = [('a.rs', 's.py', 0x512340), ('b.rs', 't.py', 0x512344)]
        code, line = census.hooked_verdict(rows, by_addr=[])
        self.assertEqual(code, 0)
        self.assertIn('2 by name', line)
        self.assertIn('0 by address', line)
        code, line = census.hooked_verdict(rows, by_addr=[rows[0]])
        self.assertEqual(code, 1)
        self.assertIn('1 by address', line)

    def test_a_hooked_callee_a_chunk_names_is_not_backed_by_that_chunk(self):
        # The thirty-eighth pass: sixteen landed sweeps name the callee their
        # script stubs in the port's own comment (`// Map::point (stub 0)
        # answers a[4..9]`), and `sweep_backed` counted each as backed beside
        # the function swept — a function the sweep answers for, never runs.
        # The count subtracts a chunk's hooked callees by address and by
        # name; the `--hooked` audit stays a lint. Made to fail first.
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp, 'src'); emu = Path(tmp, 'emu')
            src.mkdir(); emu.mkdir()
            Path(emu, 'sweep_m_s.py').write_text(
                "STUBS = [('call', 5317440, 3)]\n"
                'h = Harness(argv[1], {a: (f"{a:x}", n) for k, a, n in STUBS if k == "call"})\n')
            Path(src, 'm.rs').write_text(
                '/// `M::s@00401000`, run by `tools/emu/sweep_m_s.py` under unicorn.\n'
                '#[test]\nfn z() { /* `Objects::at` (stub 0) answers a[4]; `Other::f@00402000` is read */ }\n')
            by_name = {'M::s': [0x401000], 'Objects::at': [0x512340], 'Other::f': [0x402000]}
            self.assertEqual(census.sweep_backed(str(src), by_name, str(emu)), {0x401000, 0x402000})
            # Without the scripts' directory nothing is subtracted (the old count).
            self.assertEqual(census.sweep_backed(str(src), by_name), {0x401000, 0x402000, 0x512340})

    def test_the_trampoline_table_is_not_shadowed_by_the_sweep_regex(self):
        # The thirty-second pass named its `Harness(exe, {…})` regex `HOOKS`,
        # the trampolined functions' table's name, so `--pin --never` died
        # on `HOOKS.items()` for a whole tranche; found at the thirty-third.
        self.assertIsInstance(census.HOOKS, dict)
        self.assertIn(0x591ef0, census.HOOKS)
        self.assertTrue(census.HOOKED_SCRIPT.search('Harness(exe, {0x401000: f})'))

    def test_the_unbacked_list_skips_the_dead_weight_by_default(self):
        # Parked 1861, the thirtieth pass: destructors, `close`/`init`/
        # `clear`, loggers and `dynamic_initializer`s fill the shortest-first
        # list and every sweep lane skipped them by hand, ~300 bytes deep.
        entries = [(0x10, 'Army::~Army', 'c:\\rise\\game\\army.cpp', 93),
                   (0x20, 'Units::init', 'c:\\rise\\game\\units.cpp', 94),
                   (0x30, 'Guy::close', 'c:\\rise\\game\\guy.cpp', 90),
                   (0x40, 'Lands::clear', 'c:\\rise\\game\\lands.cpp', 110),
                   (0x50, 'Storm::registration', 'c:\\rise\\game\\storm.cpp', 110),
                   (0x60, 'LeaderData::log_data', 'c:\\rise\\game\\leader.cpp', 120),
                   (0x70, "`dynamic initializer for 'x''", 'c:\\rise\\game\\x.cpp', 50),
                   (0x80, 'Cliff::close_class', 'c:\\rise\\game\\cliff.cpp', 98),
                   (0x90, 'World::adjust_fog', 'c:\\rise\\game\\world.cpp', 106),
                   (0xa0, 'BorderWData::BorderWData', 'c:\\rise\\game\\borders.cpp', 140),
                   (0xb0, 'TerrainOil::one_time_init', 'c:\\rise\\game\\terrainoil.cpp', 127)]
        with patch.object(census, 'layer_of', return_value='simulation'):
            rows = census.rank_unbacked(entries, set(), set())
        marks = {name: mark for _, _, name, mark, _ in rows}
        for name in ('Army::~Army', 'Units::init', 'Guy::close', 'Lands::clear', 'Storm::registration',
                     'LeaderData::log_data', "`dynamic initializer for 'x''", 'Cliff::close_class',
                     'BorderWData::BorderWData', 'TerrainOil::one_time_init'):
            self.assertEqual(marks[name], 'skip', name)
        self.assertEqual(marks['World::adjust_fog'], '-')
        self.assertEqual([r[2] for r in census.worth_sweeping(rows)], ['World::adjust_fog'])

    def test_a_renderer_or_editor_body_is_marked_ui_and_hidden(self):
        # Loop 2819 (the thirty-eighth pass): the harvest lane's batches were
        # `draw_thick_diamond`, `Nuke::draw_ctw`, `Game::run_unit_editor`,
        # `WorldMap::on_right_down` — renderer and editor bodies the layer
        # rules file under simulation by their source file — and the
        # `--unbacked` head at the pass was twelve more of them. A body a
        # name rule reads as a renderer's, an input handler's, a sound's, a
        # debug or cheat action's or an editor's is marked `ui` and hidden
        # with the dead weight; the layer and the score are untouched.
        entries = [(0x10, 'NukeOut::draw_underlays', 'c:\\rise\\game\\nuke.cpp', 419),
                   (0x20, 'TerrainMap::on_mouse_move', 'c:\\rise\\game\\terrainmap.cpp', 468),
                   (0x30, 'Game::post_game_screens', 'c:\\rise\\game\\game.cpp', 510),
                   (0x40, 'Syllable::say', 'c:\\rise\\game\\syllable.cpp', 557),
                   (0x50, 'debug_say', 'c:\\rise\\game\\support.cpp', 560),
                   (0x60, 'WorldMap::setup_render_window', 'c:\\rise\\game\\worldmap.cpp', 565),
                   (0x70, 'Game::action_cheat_zero_techs', 'c:\\rise\\game\\game.cpp', 663),
                   (0x80, 'ObjectOut::play_gather_sound', 'c:\\rise\\game\\object.cpp', 663),
                   (0x90, 'RiverDoobers::add_anchor_doob_poly', 'c:\\rise\\game\\riverdoobers.cpp', 675),
                   (0xa0, 'Game::run_unit_editor', 'c:\\rise\\game\\game.cpp', 700),
                   (0xb0, 'MiscAccess::world_to_screen', 'c:\\rise\\game\\miscaccess.cpp', 300),
                   (0xc0, 'RiverDoober::add_for_render', 'c:\\rise\\game\\riverdoober.cpp', 300),
                   (0xd0, 'Nuke::setup', 'c:\\rise\\game\\nuke.cpp', 682),
                   (0xe0, 'TurnControl::do_frame', 'c:\\rise\\game\\turncontrol.cpp', 606),
                   (0xf0, 'Unit::draw_order_lines', 'c:\\rise\\game\\unit.cpp', 200),
                   (0x100, 'Map::stamp', 'c:\\rise\\game\\map.cpp', 200)]
        with patch.object(census, 'layer_of', return_value='simulation'):
            rows = census.rank_unbacked(entries, set(), set())
        marks = {name: mark for _, _, name, mark, _ in rows}
        for name in ('NukeOut::draw_underlays', 'TerrainMap::on_mouse_move', 'Game::post_game_screens',
                     'Syllable::say', 'debug_say', 'WorldMap::setup_render_window',
                     'Game::action_cheat_zero_techs', 'ObjectOut::play_gather_sound',
                     'RiverDoobers::add_anchor_doob_poly', 'Game::run_unit_editor',
                     'MiscAccess::world_to_screen', 'RiverDoober::add_for_render', 'Unit::draw_order_lines'):
            self.assertEqual(marks[name], 'ui', name)
        for name in ('Nuke::setup', 'TurnControl::do_frame', 'Map::stamp'):
            self.assertEqual(marks[name], '-', name)
        self.assertEqual(sorted(r[2] for r in census.worth_sweeping(rows)),
                         ['Map::stamp', 'Nuke::setup', 'TurnControl::do_frame'])

    def test_a_function_a_sibling_lane_s_tree_cites_is_claimed_and_hidden(self):
        # Loop 2926 (the thirty-ninth pass): 2877 re-swept 2863's
        # `kill_next_order` and 2910 re-swept 2900's `find_goody_at` and
        # `get_gpiece` — `--unbacked` cannot see a function another live
        # lane is building. `--claimed DIR` reads that lane's worktree:
        # a sweep test there cites the function, so it is claimed here.
        with tempfile.TemporaryDirectory() as tmp:
            sweep = Path(tmp) / 'crates/sim/src/sweep'
            sweep.mkdir(parents=True)
            (sweep / 'group_kill_next_order.rs').write_text(
                '/// `Group::kill_next_order@00707c80`, run by `tools/emu/sweep_group_kill_next_order.py` '
                'under unicorn\n#[test]\nfn group_kill_next_order() {}\n')
            by_name = {'Group::kill_next_order': [0x707c80], 'World::adjust_fog': [0x90]}
            claimed = census.claimed_in([tmp, str(Path(tmp) / 'no-such-tree')], by_name)
            self.assertEqual(claimed, {0x707c80})
        entries = [(0x707c80, 'Group::kill_next_order', 'c:\\rise\\game\\groups.cpp', 120),
                   (0x90, 'World::adjust_fog', 'c:\\rise\\game\\world.cpp', 106)]
        with patch.object(census, 'layer_of', return_value='simulation'):
            rows = census.rank_unbacked(entries, set(), set(), claimed={0x707c80})
        marks = {name: mark for _, _, name, mark, _ in rows}
        self.assertEqual(marks['Group::kill_next_order'], 'claimed')
        self.assertEqual(marks['World::adjust_fog'], '-')
        self.assertEqual([r[2] for r in census.worth_sweeping(rows)], ['World::adjust_fog'])

    def test_an_address_a_sweep_test_cites_without_its_class_is_named(self):
        # Loop 2843 (2837's line): a three-forms-in-one test named
        # `is_seen_wcoord@…` bare and the census scored two of three until the
        # class was spelled. `cited_bare` lists each such citation.
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp, 'src'); src.mkdir()
            Path(src, 'a.rs').write_text(
                '/// `World::is_seen@00401000` and `is_seen_wcoord@00401010`, run by `tools/emu/sweep_a.py`.\n'
                '#[test]\nfn x() { /* `is_seen_coord@00401020` too; `World::is_seen_coord@00401020` once */ }\n')
            self.assertEqual(census.cited_bare(str(src)),
                             [('a.rs', 'is_seen_wcoord', 0x401010)])
            # A free function's name is whole as it stands.
            Path(src, 'b.rs').write_text(
                '/// `angle_diff@00402000`, run by `tools/emu/sweep_angle_diff.py`.\n#[test]\nfn y() {}\n')
            self.assertEqual(census.cited_bare(str(src), {'angle_diff': [0x402000], 'World::is_seen': [0x401000]}),
                             [('a.rs', 'is_seen_wcoord', 0x401010)])

    def test_a_backticked_initializer_is_dead_weight(self):
        # Loop 2394 (2379's line, the thirty-fourth pass): the export spells
        # the compiler's initializers `` `dynamic_initializer_for_'X'' `` and
        # the pattern matched them only after `^` or `::`, so the forty
        # shortest rows of `--unbacked` were initializers every lane read
        # past by eye.
        entries = [(0x10, "`dynamic_initializer_for_'profile_init''", 'c:\\rise\\game\\profile.cpp', 40),
                   (0x20, "`dynamic_atexit_destructor_for_'profile_init''", 'c:\\rise\\game\\profile.cpp', 41),
                   (0x25, "`dynamic_initializer_for_'MiscAccess::intro''", 'c:\\rise\\game\\miscaccess.cpp', 181),
                   (0x30, 'World::adjust_fog', 'c:\\rise\\game\\world.cpp', 106)]
        with patch.object(census, 'layer_of', return_value='simulation'):
            rows = census.rank_unbacked(entries, set(), set())
        marks = {name: mark for _, _, name, mark, _ in rows}
        self.assertEqual(marks["`dynamic_initializer_for_'profile_init''"], 'skip')
        self.assertEqual(marks["`dynamic_atexit_destructor_for_'profile_init''"], 'skip')
        # The quoted name carries its own `::`, so the short name is `intro''`.
        self.assertEqual(marks["`dynamic_initializer_for_'MiscAccess::intro''"], 'skip')
        self.assertEqual([r[2] for r in census.worth_sweeping(rows)], ['World::adjust_fog'])

    def test_the_port_mark_names_the_function_s_own_class(self):
        # Loop 2286 (2221, 2276; the thirty-fourth pass): `BorderNodes::find`
        # was marked `port` because this crate defines a `fn find` somewhere,
        # and a sweep lane told to take port consumers took a function nobody
        # ports. `port` now needs the crate to spell the function's own
        # `Class::method`; a bare name match is `name`.
        entries = [(0x10, 'BorderNodes::find', 'c:\\rise\\game\\borders.cpp', 120),
                   (0x20, 'Caravan::trade_value', 'c:\\rise\\game\\caravan.cpp', 130),
                   (0x30, 'World::adjust_fog', 'c:\\rise\\game\\world.cpp', 106)]
        with patch.object(census, 'layer_of', return_value='simulation'):
            rows = census.rank_unbacked(entries, set(), {'find', 'trade_value'},
                                        spelled={'Caravan::trade_value'})
        marks = {name: mark for _, _, name, mark, _ in rows}
        self.assertEqual(marks['Caravan::trade_value'], 'port')
        self.assertEqual(marks['BorderNodes::find'], 'name')
        self.assertEqual(marks['World::adjust_fog'], '-')

    def test_the_spelled_names_are_read_off_the_crate_outside_sweep(self):
        # The same row: the spellings are every `Class::method` the crate's
        # text carries outside `sweep/` — a SEAM, a doc comment, a citation.
        with tempfile.TemporaryDirectory() as d:
            src = Path(d)
            (src / 'caravan.rs').write_text("/// the original's `Caravan::trade_value@00612340`, read here\n"
                                            "pub fn trade_value() {}\n")
            (src / 'sweep').mkdir()
            (src / 'sweep' / 'x.rs').write_text("// BorderNodes::find under the emulator\n")
            spelled = census.port_spellings(str(src))
        self.assertIn('Caravan::trade_value', spelled)
        self.assertNotIn('BorderNodes::find', spelled)

    def test_a_comment_alone_is_not_a_consumer(self):
        # Loop 2420 (2400's Loop line, parked 2401; the thirty-fifth pass):
        # six of seventeen candidates carried a `port` mark from a comment
        # that named the function beside no port of it — `Farms::add_animals`
        # named in a note on the per-farm function, `Unit::go_inside` in a
        # SEAM. A spelling counts when it sits on a code line, or in the doc
        # comment attached to the `fn` whose name is the method's own.
        with tempfile.TemporaryDirectory() as d:
            src = Path(d)
            (src / 'farms.rs').write_text("/// `Farms::add_animals@008d92d0` loops the farms; this is the per-farm one\n"
                                          "pub fn add_animal_to(f: u32) {}\n\n"
                                          "// SEAM: `Unit::go_inside@0061a2e0` is not modelled\n"
                                          "fn go_inside() {}\n")
            (src / 'caravan.rs').write_text("/// `Caravan::trade_value@00612340`\n#[inline]\npub fn trade_value() {}\n"
                                            "let v = world_adjust_fog(); // World::adjust_fog's answer\n")
            spelled = census.port_spellings(str(src))
        self.assertEqual(sorted(spelled), ['Caravan::trade_value', 'World::adjust_fog'])

    def test_the_pdb_default_resolves_the_main_checkout_from_a_worktree(self):
        # Parked 1792: `../game` is absent under `.claude/worktrees/`, so
        # every lane passed `--pdb <main checkout>/game/sbl/rise.pdb` by hand.
        # The release gate exports RON_INSTALL for every child; this test is
        # specifically about the fallback, so hold an empty value for it.
        with patch.dict(os.environ, {'RON_INSTALL': ''}):
            path = census.default_pdb(str(ROOT / 'tools'))
        self.assertTrue(path.endswith(os.path.join('game', 'sbl', 'rise.pdb')), path)
        self.assertNotIn('.claude/worktrees', path.replace(str(ROOT), '') if '.claude/worktrees' in str(ROOT) else '')
        with tempfile.TemporaryDirectory() as tmp, patch.dict(os.environ):
            # The gate sets RON_INSTALL, which wins by design; this asks the fallback.
            os.environ.pop('RON_INSTALL', None)
            Path(tmp, 'game', 'sbl').mkdir(parents=True)
            Path(tmp, 'game', 'sbl', 'rise.pdb').write_text('')
            self.assertEqual(census.default_pdb(os.path.join(tmp, 'tools')),
                             os.path.join(tmp, 'tools', '..', 'game', 'sbl', 'rise.pdb'))
            os.environ['RON_INSTALL'] = '/x/install'
            self.assertEqual(census.default_pdb(os.path.join(tmp, 'tools')), '/x/install/sbl/rise.pdb')

    def test_the_census_document_gives_the_flag_as_its_recipe(self):
        text = (ROOT / 'docs/CENSUS.md').read_text()
        self.assertIn('tools/census.py --top 30 --never --pin', text)
        self.assertNotIn('$(for f in', text,
                         'the loop splits on the archive path\'s spaces (parked 942)')


if __name__ == '__main__':
    unittest.main()
