//! **The opening camera** (item 4437, `docs/RENDERER.md` §3.31): where the
//! client's view stands when a lobby starts.
//!
//! `Game::init@0058c480` ends its load with `Game::zoom_to_first_unit@0058db30`,
//! which reads the console player's object 2000 ([`rondata::script::CAPITAL`],
//! the capital a start places): while it lives (`flags & 1`) the camera's
//! `loc` becomes its point (`+0x10`, `+0x14`, each `^ 0x63637`) through
//! `Camera::set_loc@00844a60` (skipped under `scroll_lock_flags & 1`), and
//! otherwise half the world's `xs` and `ys`, which are counted in cells
//! (`WorldData +0`, `+4`; 60 and 60 on run1570's packet) and land as world
//! units, near the map's corner. The zoom becomes level 5,
//! `fixed_zooms[5]`, the pinned camera's own `46a2f59c` on every packet on
//! disk. Nothing else of the camera moves.

use sim::observe::ObjId;
use sim::observe::{Scene, TerrainScene};

/// **The opening camera**: [`crate::pinned_camera`] with `loc` set as
/// `zoom_to_first_unit` sets it for slot 0, the client's console player.
pub fn camera(scene: &Scene, terrain: &TerrainScene) -> render::Camera {
    let mut c = crate::pinned_camera();
    let capital = ObjId {
        who: 0,
        o: rondata::script::CAPITAL,
    };
    let (x, y) = match scene.buildings.iter().find(|b| b.id == capital) {
        Some(b) => (b.pos.x, b.pos.y),
        None => (terrain.width / 2, terrain.height / 2),
    };
    (c.loc_x, c.loc_y) = (x as f32, y as f32);
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The capital in the opening frame** (item 4437, `docs/RENDERER.md`
    /// §3.31): of the lobbies on disk ([`rondata::diff::START_LOBBIES`]), those
    /// whose start, built from the page ([`rondata::free::FreeRunner::from_choices`]
    /// on the events that set the lobby), puts slot 0's capital — its one city
    /// centre on frame 0 — at a pixel inside the 1920×1080 screen
    /// ([`crate::pinned_screen`]) under the client's opening camera
    /// ([`super::camera`]). Minted at its found value, 95, on [`crate::pinned_camera`]
    /// (run1559b's packet's camera on Great Lakes, [`CAPITAL_IN_FRAME_PINNED`]);
    /// 95 → 161 on [`super::camera`].
    const CAPITAL_IN_FRAME: usize = 161;

    /// The same count on [`crate::pinned_camera`], the camera every lobby
    /// opened on before item 4437: the comparison seen able to fail.
    const CAPITAL_IN_FRAME_PINNED: usize = 95;

    /// **The packets that never moved the camera** (item 4437): of the 55
    /// `frame-snapshot.bin` under `~/ron-golden/*/map-14/`, the 21 whose
    /// camera `loc` is the console player's object 2000 to the bit, all on
    /// Great Lakes' harness lobby, whose capital stands at (3168, 30816):
    /// `45460000 46f0c000` on each. The other 34 were moved by the lanes'
    /// cursor pins (an edge scroll in real time: run1559b's x by −1063.72,
    /// run1441's y by −1071.32, the diagonal pins' ±535.66) or a console
    /// `camera` line (run1564, run1665). Read off each packet by the camera
    /// at `MiscAccess::camera` (`0x00c06200`) `+0x304`, `+0x308` and the
    /// object list at `GameAccess::objects` (`0x00c0618c`) `+0x14 + who·0x1c`,
    /// entry 2000; zoom `+0x2f0` is `fixed_zooms[5]` (`+0x334`) on all 55.
    const UNMOVED_PACKETS: [&str; 21] = [
        "ch95pkt", "run1327", "run1332a", "run1351", "run1351c", "run1417", "run1426b", "run1570",
        "run1586", "run1587", "run1588", "run1589", "run1590", "run1591", "run1592", "run1652",
        "run1653", "run839", "run938b", "run943c", "run946",
    ];

    /// Their `loc`, as the packets hold it: `(45460000, 46f0c000)`.
    const UNMOVED_LOC: (u32, u32) = (0x4546_0000, 0x46f0_c000);

    use rondata::lobby::{Choices, Lobby, Roster, events_for, lobbies_on_disk};
    use std::path::Path;

    /// Slot 0's capital on frame 0 of `scene`, as its anchor pixel under
    /// `camera`: the one building of slot 0 that is a city.
    fn capital_pixel(
        camera: &render::Camera,
        scene: &sim::observe::Scene,
    ) -> Option<((i32, i32), (i32, i32))> {
        let cities: Vec<_> = scene
            .buildings
            .iter()
            .filter(|b| b.id.who == 0 && b.city)
            .collect();
        let [b] = cities.as_slice() else {
            return None;
        };
        let p = render::camera::world_to_screen(
            &camera.to_screen_matrix(),
            &crate::pinned_screen(),
            b.pos.x,
            b.pos.y,
            b.z,
            None,
        );
        Some(((b.pos.x, b.pos.y), (p.x, p.y)))
    }

    fn inside((x, y): (i32, i32)) -> bool {
        let s = crate::pinned_screen();
        (0..s.scaled_xs).contains(&x) && (0..s.scaled_ys).contains(&y)
    }

    /// **The count**: every lobby on disk started from its page, slot 0's
    /// capital projected at the opening camera, against the screen.
    #[test]
    fn the_capital_in_the_opening_frame() {
        let root = std::env::var_os("RON_INSTALL")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "../../game".into());
        let install = rondata::Install::new(&root);
        if !install.looks_valid() {
            println!("skipping: no install at {}", root.display());
            return;
        }
        let dir = crate::world::gamelog_dir();
        if !dir.is_dir() {
            println!("skipping: no gamelog directory at {}", dir.display());
            return;
        }
        let roster = Roster::read(Path::new(&root)).expect("the install's roster");
        let loaded = rondata::load::load(&install).expect("the install loads");
        let lobbies = lobbies_on_disk(&dir, rondata::corpus::Ledger::of_tree()).expect("the logs");
        assert_eq!(lobbies.len(), rondata::diff::START_LOBBIES);
        let pinned = crate::pinned_camera();
        let (mut found, mut found_pinned) = (0, 0);
        let mut outside = Vec::new();
        for l in &lobbies {
            let block = Lobby::parse(&l.block).expect("the dump's block parses");
            let mut page = Choices::blank();
            for e in events_for(&block, &roster) {
                page.apply(&e, &roster).expect("the page takes the lobby");
            }
            let (runner, _) =
                rondata::free::FreeRunner::from_choices(&loaded, &page, sim::Tuning::RON);
            let scene = runner.scene(Some(0));
            let opening = camera(&scene, &runner.terrain());
            let style = block.get("MAP_STYLE").unwrap_or("?").trim().to_string();
            let (Some((pos, pixel)), Some((_, before))) = (
                capital_pixel(&opening, &scene),
                capital_pixel(&pinned, &scene),
            ) else {
                outside.push(format!("{} style {style}: no one capital", l.captures[0]));
                continue;
            };
            found_pinned += usize::from(inside(before));
            if inside(pixel) {
                found += 1;
            } else {
                outside.push(format!(
                    "{} style {style}: capital {pos:?} at {pixel:?}",
                    l.captures[0]
                ));
            }
        }
        println!(
            "OPENING capital in frame {found} of {} (pinned {found_pinned}); outside: {outside:#?}",
            lobbies.len()
        );
        assert_eq!(
            found_pinned, CAPITAL_IN_FRAME_PINNED,
            "CAPITAL_IN_FRAME_PINNED"
        );
        assert_eq!(found, CAPITAL_IN_FRAME, "CAPITAL_IN_FRAME");
    }

    /// **The opening camera is the packets' own** on the harness lobby:
    /// its start built from the lobby's capture, slot 0's camera `loc` to
    /// the bit what the 21 unmoved packets hold ([`UNMOVED_LOC`]), and
    /// the capital at the screen's centre column. Seen red with the
    /// pinned camera: `loc` `4503847c`, run1559b's edge-scrolled x.
    #[test]
    fn the_opening_camera_is_the_unmoved_packets_camera() {
        let Ok(r) = crate::input::tests::lobby() else {
            println!("skipping: no lobby");
            return;
        };
        let scene = r.scene(Some(0));
        let c = camera(&scene, &r.terrain());
        println!(
            "OPENING harness loc {:08x} {:08x}",
            c.loc_x.to_bits(),
            c.loc_y.to_bits()
        );
        assert_eq!(
            (c.loc_x.to_bits(), c.loc_y.to_bits()),
            UNMOVED_LOC,
            "the {} unmoved packets' loc",
            UNMOVED_PACKETS.len()
        );
        let (_, (px, _)) = capital_pixel(&c, &scene).expect("one capital");
        assert_eq!(px, crate::pinned_screen().scaled_xs / 2);
        let pinned = crate::pinned_camera();
        assert_ne!(
            pinned.loc_x.to_bits(),
            UNMOVED_LOC.0,
            "the pinned camera was moved"
        );
    }
}
