//! **The opening camera** (item 4437, `docs/RENDERER.md` §3.6): where the
//! client's view stands when a lobby starts.

/// **The capital in the opening frame** (item 4437, `docs/RENDERER.md`
/// §3.6): of the lobbies on disk ([`rondata::diff::START_LOBBIES`]), those
/// whose start, built from the page ([`rondata::free::FreeRunner::from_choices`]
/// on the events that set the lobby), puts slot 0's capital — its one city
/// centre on frame 0 — at a pixel inside the 1920×1080 screen
/// ([`crate::pinned_screen`]) under the client's opening camera. Minted at
/// its found value on [`crate::pinned_camera`], run1559b's packet's camera
/// on Great Lakes.
pub const CAPITAL_IN_FRAME: usize = 95;

#[cfg(test)]
mod tests {
    use super::*;
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
        let camera = crate::pinned_camera();
        let mut found = 0;
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
            let style = block.get("MAP_STYLE").unwrap_or("?").trim().to_string();
            let Some((pos, pixel)) = capital_pixel(&camera, &scene) else {
                outside.push(format!("{} style {style}: no one capital", l.captures[0]));
                continue;
            };
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
            "OPENING capital in frame {found} of {}; outside: {outside:#?}",
            lobbies.len()
        );
        assert_eq!(found, CAPITAL_IN_FRAME, "CAPITAL_IN_FRAME");
    }
}
