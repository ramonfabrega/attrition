//! **The national borders, drawn** (item 4306, `docs/RENDERER.md` §3.29).

#[cfg(test)]
pub(crate) mod tests {
    use crate::ground::tests::{PixelTiles, Probe, ground_why, probe_at, run1564};
    use std::path::PathBuf;

    /// **A border's pixel, by colour** (item 4306): the band is a team's
    /// border colour at `BorderSpline::border_alpha` 64 over the ground,
    /// so it reads bluer than the mean of red and green while red keeps
    /// pace with green — the lake fails the second, sand and grass the
    /// first. run1564's band reads (154, 148, 169) over sand (192, 181,
    /// 117).
    pub(crate) fn band(rgb: [u8; 3]) -> bool {
        let [r, g, b] = rgb.map(i32::from);
        b > (r + g) / 2 && r + 2 >= g && r > 110
    }

    /// **Where a border can be drawn**: each pixel whose cell, or one of
    /// its eight neighbours, is held by another owner than it, on the
    /// simulation's own grid (`Sim::observe_territory`), as
    /// `Border::sort_regular@00880960` adds an edge only between two
    /// cells whose owners differ.
    pub(crate) fn near_an_edge(p: &mut Probe, tiles: &PixelTiles) -> Vec<bool> {
        let t = p.world.runner.territory();
        let at = |x: i32, y: i32| {
            (x >= 0 && y >= 0 && x < t.width && y < t.height)
                .then(|| t.owner[(y * t.width + x) as usize])
        };
        tiles
            .iter()
            .map(|q| {
                let Some((tx, ty)) = *q else { return false };
                let (cx, cy) = (tx.div_euclid(4), ty.div_euclid(4));
                let Some(me) = at(cx, cy) else { return false };
                (-1..=1).any(|dy| (-1..=1).any(|dx| at(cx + dx, cy + dy).is_some_and(|o| o != me)))
            })
            .collect()
    }

    fn still(p: &mut Probe, name: &str) -> Vec<u8> {
        let size = (p.photo.width as u32, p.photo.height as u32);
        let camera = p.camera;
        let s = crate::still_of(
            &mut p.world,
            &camera,
            size,
            crate::gpu::SilhouetteOrder::Before,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        if let Some(dir) = std::env::var_os("RON_STILL_DIR") {
            let out = PathBuf::from(dir).join(format!("still-{name}.bmp"));
            std::fs::write(&out, crate::gpu::bmp(&s, size)).expect("the still writes");
        }
        s
    }

    /// The row on one still: `(both, theirs, ours)` — the pixels near an
    /// edge the photograph and the still both call a border's, the
    /// photograph's alone counted, and the still's alone counted.
    fn row(p: &Probe, hood: &[bool], ours: &[u8]) -> [usize; 3] {
        let px = |rgba: &[u8], i: usize| [rgba[i * 4], rgba[i * 4 + 1], rgba[i * 4 + 2]];
        let mut out = [0; 3];
        for (i, _) in hood.iter().enumerate().filter(|(_, h)| **h) {
            let (a, b) = (band(px(&p.photo.rgba, i)), band(px(ours, i)));
            out[0] += usize::from(a && b);
            out[1] += usize::from(a);
            out[2] += usize::from(b);
        }
        out
    }

    /// **`RUN1564_3279_BORDER`** (item 4306): on run1564's photograph,
    /// of the pixels within a cell of an owner's edge, `(both, theirs,
    /// ours)` as [`row`] counts them. Minted at the item's first act with
    /// no border drawn: 0 of 1,440 (1,433 of them east of x 1800, the
    /// photographed band), and none of ours.
    pub const RUN1564_3279_BORDER: [usize; 3] = [0, 1440, 0];

    #[test]
    fn the_border_band_on_run1564() {
        let Some(mut p) = probe_at(run1564()) else {
            return;
        };
        let (_, tiles, _) = ground_why(&mut p);
        let hood = near_an_edge(&mut p, &tiles);
        let s = still(&mut p, "border");
        let got = row(&p, &hood, &s);
        println!(
            "RUN1564_3279_BORDER: {got:?}, hood {}",
            hood.iter().filter(|h| **h).count()
        );
        assert_eq!(got, RUN1564_3279_BORDER);
    }
}
