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
}
