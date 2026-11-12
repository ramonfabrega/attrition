//! **The paved base under a city** (item 4190; `docs/RENDERER.md`, the
//! section 4190 names): a building's footprint weathering,
//! `Terrain::add_weathering@00850120`, drawn by
//! `TerrainOut::render_weathering@00852f70` — not `RoadsOut` (draft;
//! `docs/journal/2026-11-12-item-4190.md`).

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn card_trace_path() -> PathBuf {
        std::env::var_os("RON_CARD_TRACE")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME").unwrap_or_default();
                PathBuf::from(home).join("ron-golden/run1566b/map-14/rontrace.log")
            })
    }

    /// **Draw 90 is the weathering's** (item 4190): on run1566b's card
    /// trace, the pass after 3279, draw 90 binds `Modern_Weathering.tga`
    /// and `age0_22.tga` (`tilesets.xml` `<COMMON><WEATHERING>`, not
    /// `roads.xml`) under enables `0x1c204`, lit, stage 0
    /// `MODULATE(TEXTURE, DIFFUSE)` with A `SELECTARG1(TEXTURE)`, stage 1
    /// disabled — so `age0_22`'s alpha never reaches the pixel. Its return
    /// address lies in `RoadsOut::render` because the card flushes one draw
    /// late: `Scene::render@008b3270` calls `render_weathering(0, 3)` just
    /// before `render_roads`, and draw 91 (`modern_road_pieces.tga`, enables
    /// `0x20004`, `RoadsOut`'s own state) flushes in `Game::loop_render`.
    #[test]
    fn draw_90_is_the_weathering_s() {
        use rondata::trace::card;
        let Ok(bytes) = std::fs::read(card_trace_path()) else {
            println!("skipping: no card trace at {}", card_trace_path().display());
            return;
        };
        let all = card::draws(&bytes);
        let at = |i: u32| {
            all.iter()
                .find(|d| d.frame == 3279 && d.index == i)
                .expect("the pass after 3279 holds the draw")
        };
        let (w, r) = (at(90), at(91));
        assert_eq!(
            [w.file(0), w.file(1)],
            ["Modern_Weathering.tga", "age0_22.tga"]
        );
        assert_eq!((w.enables(), w.src_blend(), w.dst_blend()), (0x1c204, 5, 6));
        assert_eq!(
            w.colour_op(0),
            (card::TOP_MODULATE, card::TA_TEXTURE, card::TA_DIFFUSE)
        );
        assert_eq!(w.colour_op(1).0, 0, "stage 1 disabled");
        assert_eq!(w.alpha_op(0), (1, card::TA_TEXTURE, card::TA_CURRENT));
        assert!(w.has("ENABLE_LIGHTING") && w.has("TEX1_USE_UV1"));
        // The flush: draw 90 returns into `RoadsOut::render`
        // (0x8943d0..0x894720), draw 91 into `Game::loop_render`.
        assert!((0x8943d0..0x894720).contains(&w.frames[0]));
        assert_eq!(r.file(0), "modern_road_pieces.tga");
        assert_eq!(r.enables(), 0x20004);
        assert_eq!(r.frames[0], 0x591a45);
    }
}
