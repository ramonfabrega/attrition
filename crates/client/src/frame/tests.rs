use super::*;

/// The switch, branch by branch: a value of each arm, and a default.
#[test]
fn the_codes_are_age_update_s() {
    assert_eq!(civ_code(0, 0), Some("MD0"));
    assert_eq!(civ_code(3, 2), Some("AS0"));
    assert_eq!(civ_code(4, 0), Some("SA0"));
    assert_eq!(civ_code(8, 1), Some("SA0"));
    assert_eq!(civ_code(6, 0), Some("IRO"));
    assert_eq!(civ_code(9, 0), None);
    assert_eq!(civ_code(1, 3), Some("NE3"));
    assert_eq!(civ_code(8, 4), Some("EIN"));
    assert_eq!(civ_code(9, 4), None);
    assert_eq!(civ_code(9, 5), Some("IND"));
    assert_eq!(civ_code(0, 7), Some("MOD"));
}

/// The layout as interface.xml writes it, on a cut of its own text.
#[test]
fn the_layout_reads_the_window() {
    let xml = r#"<IFACEMINIMAP name="main" left="0" top="0" width="332" height="196">
 <IFACERECT name="map" left="10" top="26" width="317" height="159"/>
 <IFACETEX name="back" texture="TEX_CIV" id="0" left="173" top="315" width="339" height="197"/>
 <BUTTON name="cycle_military" left="11" top="138" width="47" height="47">
 <IFACETEX name="tex" texture="TEX_CIV" left="87" top="51" width="47" height="47"/>
 </BUTTON>
 </IFACEMINIMAP>"#;
    let l = MinimapLayout::parse(xml).expect("the window parses");
    assert_eq!(l.size, (332, 196));
    assert_eq!(l.map, (10, 26, 317, 159));
    assert_eq!(l.back, (173, 315, 339, 197));
    assert_eq!(l.buttons, vec![((11, 138, 47, 47), (87, 51, 47, 47))]);
}

/// **`RUN1814B_3294_MINIMAP_FRAME`** (item 4694; `docs/RENDERER.md`
/// §6.10): on run1814b's frame 3294, the pixels of the minimap's window
/// whose frame texel and its eight neighbours are opaque, `[pixels, the
/// photograph's within two levels of the atlas, ours within two levels of
/// the photograph's with no frame drawn, ours with it, the photograph's
/// mean luma, ours with no frame, ours with it]`, the lumas ×10 and
/// rounded (0.299 R + 0.587 G + 0.114 B).
/// The photograph parts from the atlas on 1,418, every one on the idle
/// citizens' button, which theirs draws dark (4694a).
pub(crate) const RUN1814B_3294_MINIMAP_FRAME: [usize; 7] =
    [32_483, 31_065, 24, 31_065, 944, 1_022, 968];

/// **`RUN1814B_3294_LEFT_WINDOW`** (item 4724; `docs/RENDERER.md`
/// §6.11): on the same frame, the bar's left window's opaque pixels on the
/// frame, `[pixels, the photograph's within two levels of the composition,
/// ours within two levels of the photograph's with no frame drawn, ours
/// with it, the photograph's mean luma, ours with no frame, ours with it]`,
/// the lumas as [`RUN1814B_3294_MINIMAP_FRAME`]'s.
/// The photograph parts from the composition on the five slots that hold
/// a button's picture (4724b), 11,850, and in the clock's box on 217 of
/// 3,800 (item 4727, §6.12): the last digit, ours `9` on 3295's clock and
/// theirs `8`, and the glyphs' faint edges.
/// Item 4727: 45,964 → 49,547, the box and the clock's line; luma ×10 459
/// → 418.
pub(crate) const RUN1814B_3294_LEFT_WINDOW: [usize; 7] = [61_614, 49_547, 0, 49_547, 516, 820, 418];

/// The rectangles as interface.xml writes them, each form once.
#[test]
fn the_rectangles_resolve() {
    use left::resolve;
    let r = |t: &str| resolve(t, (313, 161));
    assert_eq!(
        r(r#"IFACERECT left="0" top="38" right_inverse="0" height="3""#),
        Some((0, 38, 313, 41))
    );
    assert_eq!(
        r(r#"IFACERECT left="60" top="0" bottom_inverse="0" width="3""#),
        Some((60, 0, 63, 161))
    );
    assert_eq!(
        r(r#"IFACERECT top="35" right_inverse="-1" width="-6" height="7""#),
        Some((308, 35, 314, 42))
    );
    assert_eq!(
        r(r#"IFACERECT left="-3" bottom_inverse="-3" width="6" height="-6""#),
        Some((-3, 158, 3, 164))
    );
    assert_eq!(
        resolve(
            r#"IFACERECT left="0" bottom_inverse="0" right="3" height="-164""#,
            (316, 198)
        ),
        Some((0, 34, 3, 198))
    );
    assert_eq!(r(r#"IFACERECT name="x""#), None);
}

/// The left window off a cut of interface.xml: the strip, the edges, the
/// pause button, then the grid at the left window's corner plus its own.
#[test]
fn the_left_window_lays_out() {
    let xml = r#"<IFACEMAINBASE name="ifacemainbase" left="0" top_inverse="198" percent_width="1000" height="198" show="0">
    <IFACERECT name="left_tile" left="0" bottom_inverse="0" right="3" height="-164"/>
    <IFACERECT name="left_tile_only" left="2" bottom_inverse="0" right="313" height="-3"/>
    <IFACETEX name="left_tile" texture="TEX_CIV" left="1" top="51" width="3" height="93"/>
    <IFACETEX name="left_tile_only" texture="TEX_CIV" left="51" top="0" width="103" height="3"/>
    <IFACETEX name="bar" texture="TEX_CIV" left="390" top="131" width="122" height="34"/>
    <IFACETEX name="time_back" texture="TEX_CIV" left="261" top="108" width="124" height="19"/>
    <IFACERECT name="timer_area2" left="32" top="10" width="200" height="19"/>
    <BUTTON name="pause" left="4" top="10" width="24" height="19">
      <IFACETEX name="tex" texture="TEX_CIV" left="313" top="0" width="24" height="19"/>
    </BUTTON>
    <IFACERECT name="left_window" left="0" top="34" width="316" height="163"/>
    <IFACERECT name="mid_window" left="316" top="1" width="332" height="198"/>
  </IFACEMAINBASE>
  <IFACEOPTIONS name="main" left="3" top="0" width="313" height="161" num_x_slots="5" num_y_slots="4">
    <IFACERECT name="misc_tile" left="60" top="0" bottom_inverse="0" width="3"/>
    <IFACETEX name="misc_tile" texture="TEX_CIV" left="155" top="8" width="3" height="30"/>
    <IFACETEX name="blank_button" texture="TEX_CIV" left="211" top="20" width="60" height="38"/>
    <IFACERECT name="misc" left="58" top="-1" width="7" height="6"/>
    <IFACETEX name="misc" texture="TEX_CIV" left="26" top="10" width="7" height="6"/>
    <POSITIONING x="0" y="-1" inc_y="41">
      <POSITIONING inc_x="63">
        <BUTTON name="butt_00" width="61" height="39"/>
        <BUTTON name="butt_10" width="61" height="39"/>
      </POSITIONING>
      <POSITIONING inc_x="63">
        <BUTTON name="butt_01" width="61" height="39"/>
      </POSITIONING>
    </POSITIONING>
  </IFACEOPTIONS>"#;
    use left::{Fill::*, Piece};
    let l = left::LeftLayout::parse(xml).expect("the window parses");
    assert_eq!(l.size, (316, 198));
    assert_eq!(l.mid_window, (316, 1));
    assert_eq!(l.clock, (32, 10, 232, 29));
    let p = |crop, span, fill| Piece { crop, span, fill };
    let blank = (211, 20, 60, 38);
    assert_eq!(
        l.pieces,
        vec![
            p((390, 131, 122, 34), (0, 0, 316, 34), Tile),
            p((261, 108, 124, 19), (32, 10, 232, 29), Stretch),
            p((1, 51, 3, 93), (0, 34, 3, 198), Tile),
            p((51, 0, 103, 3), (2, 195, 313, 198), Tile),
            p((313, 0, 24, 19), (4, 10, 28, 29), Stretch),
            p((155, 8, 3, 30), (63, 34, 66, 195), Tile),
            p((26, 10, 7, 6), (61, 33, 68, 39), Stretch),
            p(blank, (3, 33, 64, 72), Stretch),
            p(blank, (66, 33, 127, 72), Stretch),
            p(blank, (3, 74, 64, 113), Stretch),
        ]
    );
}

/// The stretch at its own size is the crop, texel for texel; at one wider
/// and taller it reads the atlas past the crop's edge.
#[test]
fn the_stretch_is_unclamped() {
    // A 4 × 1 atlas, 0, 100, 200, 250; the crop is texels 1 and 2.
    let rgba: Vec<u8> = [0u8, 100, 200, 250]
        .iter()
        .flat_map(|&v| [v, v, v, 255])
        .collect();
    let atlas = render::tga::Image {
        width: 4,
        height: 1,
        kind: 2,
        bits: 32,
        footer: false,
        rgba,
    };
    let s = |dw, x| left::stretched(&atlas, (1, 0, 2, 1), (dw, 1), (x, 0))[0];
    assert_eq!((s(2, 0), s(2, 1)), (100, 200));
    // At 3 wide: the sample points 1/6 into texel 0 and 1/6 into texel 3.
    assert_eq!((s(3, 0), s(3, 1), s(3, 2)), (83, 150, 208));
}

#[test]
fn the_minimap_frame_on_run1814b() {
    let photo_path = std::env::var_os("RON_PHOTO_RUN1814B_TGA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                .join("ron-data/lab-experiments/2026-11-23-item-4624/run1814b-3294.tga")
        });
    let Ok(bytes) = std::fs::read(&photo_path) else {
        println!("skipping: no photograph at {}", photo_path.display());
        return;
    };
    let photo = render::tga::decode(&bytes).expect("the photograph decodes");
    let root = std::env::var_os("RON_INSTALL")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("../../game"));
    let mut world = match crate::world::World::open_game(
        &root,
        &crate::world::gamelog_dir(),
        crate::world::Game::Lobby,
    ) {
        Ok(w) => w,
        Err(e) => {
            println!("skipping: {e}");
            return;
        }
    };
    world.run_to(3294);
    world.blend = crate::world::Blend::Last;
    world.run_to(3295);
    world.fog = Some(crate::world::CONSOLE);
    let frame = console_frame(&world, crate::world::CONSOLE).expect("the console's frame");
    let size = (photo.width as i32, photo.height as i32);
    assert_eq!(size, crate::minimap::QUAD_FRAME, "the photograph's frame");
    let o = frame.origin(size);
    assert_eq!(o, (316, 884), "the window's origin");
    // The mask: opaque, its eight neighbours opaque, on the frame.
    let opaque = |x: i32, y: i32| frame.texel(o, x, y).is_some_and(|t| t[3] == 255);
    let mask: Vec<(i32, i32)> = (o.1..size.1)
        .flat_map(|y| (o.0..o.0 + frame.back.w as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            (-1..=1).all(|dy| (-1..=1).all(|dx| y + dy >= size.1 || opaque(x + dx, y + dy)))
        })
        .collect();
    let px = |img: &[u8], (x, y): (i32, i32)| {
        let i = (y as usize * size.0 as usize + x as usize) * 4;
        [img[i], img[i + 1], img[i + 2]]
    };
    let near = |a: [u8; 3], b: [u8; 3]| (0..3).all(|c| a[c].abs_diff(b[c]) <= 2);
    let luma = |img: &[u8]| {
        let s: f64 = mask
            .iter()
            .map(|&p| {
                let [r, g, b] = px(img, p).map(f64::from);
                0.299 * r + 0.587 * g + 0.114 * b
            })
            .sum();
        (s / mask.len() as f64 * 10.0).round() as usize
    };
    let atlas_agrees = mask
        .iter()
        .filter(|&&(x, y)| {
            let t = frame.texel(o, x, y).expect("on the frame");
            near(px(&photo.rgba, (x, y)), [t[0], t[1], t[2]])
        })
        .count();
    let sel = crate::minimap::blips::tests::capital_selected(&world);
    let mut camera = crate::pinned_camera();
    (camera.loc_x, camera.loc_y) = crate::minimap::fog::tests::RUN1814B_LOC;
    let wh = (size.0 as u32, size.1 as u32);
    let still = |world: &mut crate::world::World| {
        crate::still_minimap(
            world,
            &camera,
            wh,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
            &sel,
            &|_| None,
            true,
        )
    };
    // Seen red: the still with no frame drawn.
    let art = world.frame_art.take();
    let bare = still(&mut world);
    world.frame_art = art;
    let framed = still(&mut world);
    if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
        let out = std::path::PathBuf::from(dir).join("still-run1814b-3294-frame.bmp");
        std::fs::write(&out, crate::gpu::bmp(&framed, wh)).expect("the still writes");
    }
    let agree = |img: &[u8]| {
        mask.iter()
            .filter(|&&p| near(px(img, p), px(&photo.rgba, p)))
            .count()
    };
    // Item 4724: the bar's left window, on the same stills.
    let b = frame.bar_origin(size);
    assert_eq!(b, (0, 883), "the bar's origin");
    // Item 4727: the window with the clock's line of the still's frame.
    let line = super::clock_line(&world).expect("the clock's line");
    assert_eq!(line, "Normal: 00:03:39", "the clock's line on 3295");
    let win = frame.left_with(Some(&line));
    let left_texel = |x: i32, y: i32| super::texel_of(&win, b, x, y);
    let left_mask: Vec<(i32, i32)> = (b.1..size.1)
        .flat_map(|y| (b.0..b.0 + frame.left.w as i32).map(move |x| (x, y)))
        .filter(|&(x, y)| left_texel(x, y).is_some_and(|t| t[3] == 255))
        .collect();
    let on = |m: &[(i32, i32)], img: &[u8], other: &[u8]| {
        m.iter()
            .filter(|&&p| near(px(img, p), px(other, p)))
            .count()
    };
    let lum = |m: &[(i32, i32)], img: &[u8]| {
        let s: f64 = m
            .iter()
            .map(|&p| {
                let [r, g, bl] = px(img, p).map(f64::from);
                0.299 * r + 0.587 * g + 0.114 * bl
            })
            .sum();
        (s / m.len() as f64 * 10.0).round() as usize
    };
    let composed = left_mask
        .iter()
        .filter(|&&(x, y)| {
            let t = left_texel(x, y).expect("on the window");
            near(px(&photo.rgba, (x, y)), [t[0], t[1], t[2]])
        })
        .count();
    let left_row = [
        left_mask.len(),
        composed,
        on(&left_mask, &bare, &photo.rgba),
        on(&left_mask, &framed, &photo.rgba),
        lum(&left_mask, &photo.rgba),
        lum(&left_mask, &bare),
        lum(&left_mask, &framed),
    ];
    println!("RUN1814B_3294_LEFT_WINDOW: {left_row:?}");
    let row = [
        mask.len(),
        atlas_agrees,
        agree(&bare),
        agree(&framed),
        luma(&photo.rgba),
        luma(&bare),
        luma(&framed),
    ];
    println!("RUN1814B_3294_MINIMAP_FRAME: {row:?}");
    assert_eq!(row, RUN1814B_3294_MINIMAP_FRAME);
    assert_eq!(left_row, RUN1814B_3294_LEFT_WINDOW);
}
