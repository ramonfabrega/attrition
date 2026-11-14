//! The surf's meshes (item 4239; `docs/RENDERER.md` §3.27):
//! `Surf::draw@0089d6f0`'s rebuild of the `final_*` lists, one per texture
//! slot, through `SurfWave::draw@0089df10` and
//! `SurfWaveSection::draw@0089e360` (read off the listing; the decompiler
//! lost their floats).

use super::WaveStruct;
use rondata::artdata::SurfClass;

/// `0x00b69428`: a world unit's texcoord on a world-mapped section,
/// 1/576 (three tiles a repeat).
pub const WORLD_TEX_SCALE: f32 = 0.001_736_111_1;

/// `world_texx_speed` `0x00c062b8` and `world_texy_speed` `0x00c062b0`,
/// two each, read off run1564's packet: the scroll a frame of a
/// world-mapped section, by its `world_coord_mapping` less one.
pub const WORLD_TEXX_SPEED: [f32; 2] = [-0.000_833_33, 0.0];
pub const WORLD_TEXY_SPEED: [f32; 2] = [0.0, -0.000_833_33];

/// One `SurfWaveSection`: its model's frames as `SurfWaveSection::init`
/// keeps them.
#[derive(Clone, Debug)]
pub struct Section {
    /// The texture slot it draws into (its `tex0`).
    pub slot: usize,
    /// `switch_node_verts`: each frame's `(x, y)` (`Mot::section_verts`).
    pub verts: Vec<Vec<[f32; 2]>>,
    /// `switch_node_texs0`: each frame's texcoords.
    pub texs: Vec<Vec<[f32; 2]>>,
    /// `switch_node_face`.
    pub faces: Vec<u16>,
    pub timing: usize,
    /// `world_tex_coords`: −1 a model whose texcoords do not move, 0 one
    /// whose texcoords move, above 0 world-mapped on that scroll.
    pub mapping: i32,
}

/// What the meshes are built from: each `<WAVE>`'s sections in slot
/// order, the timelines, and the wave's length.
#[derive(Clone, Debug)]
pub struct SurfArt {
    pub waves: Vec<Vec<Section>>,
    pub timings: Vec<(Vec<f32>, Vec<i32>)>,
    pub slots: usize,
    pub wave_time: i32,
}

impl SurfArt {
    /// The class's sections, their models decoded (`render::mot`).
    pub fn load(class: &SurfClass) -> Result<SurfArt, String> {
        let mut waves = vec![];
        for w in &class.waves {
            let mut sections = vec![];
            for d in &w.sections {
                let bytes =
                    std::fs::read(&d.model).map_err(|e| format!("{}: {e}", d.model.display()))?;
                let mot = render::mot::decode(&bytes)
                    .map_err(|e| format!("{}: {e:?}", d.model.display()))?;
                let mapping = if d.mapping == 0 && mot.texcoords_static() {
                    -1
                } else {
                    d.mapping
                };
                sections.push(Section {
                    slot: d.tex0,
                    verts: (0..mot.frames.len())
                        .map(|f| mot.section_verts(f))
                        .collect(),
                    texs: mot.frames.iter().map(|f| f.texs.clone()).collect(),
                    faces: mot.faces.clone(),
                    timing: d.timing,
                    mapping,
                });
            }
            sections.sort_by_key(|s| s.slot);
            waves.push(sections);
        }
        Ok(SurfArt {
            waves,
            timings: class.timings.clone(),
            slots: class.textures.len(),
            wave_time: class.wave_time,
        })
    }
}

/// One `final_*` list: what the card draws for a texture slot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FinalList {
    pub verts: Vec<[f32; 3]>,
    pub tex0: Vec<[f32; 2]>,
    pub tex1: Vec<[f32; 2]>,
    /// The vertex colour's alpha; its rgb is white.
    pub alpha: Vec<u8>,
    pub faces: Vec<u16>,
}

impl SurfArt {
    /// `Surf::draw`'s rebuild at game frame `frame` (`graphic_dt`
    /// milliseconds into it): every wave and breaker of `drawn`, in order,
    /// into one list per slot.
    pub fn build<'a>(
        &self,
        drawn: impl IntoIterator<Item = &'a WaveStruct>,
        frame: i32,
        graphic_dt: i32,
    ) -> Vec<FinalList> {
        let mut lists = vec![FinalList::default(); self.slots];
        let ff = frame as f32;
        let xoff = WORLD_TEXX_SPEED.map(|s| ff * s);
        let yoff = WORLD_TEXY_SPEED.map(|s| ff * s);
        let span = ((self.wave_time * 200) / 3) as f32;
        for w in drawn {
            if w.time < 0 {
                continue;
            }
            let ms = (w.time * 200) / 3 + graphic_dt;
            let t = ms as f32 / span;
            for s in &self.waves[w.kind] {
                self.section(&mut lists[s.slot], s, w, t, xoff, yoff);
            }
        }
        lists
    }

    /// `SurfWaveSection::draw@0089e360`: the frame pair around `t` on the
    /// section's timeline, the vertices between them laid on the wave's
    /// axes, its alpha. A section whose alpha is below 1 is not drawn.
    fn section(
        &self,
        out: &mut FinalList,
        s: &Section,
        w: &WaveStruct,
        t: f32,
        xoff: [f32; 2],
        yoff: [f32; 2],
    ) {
        let (pct, alpha) = &self.timings[s.timing];
        let n = pct.len();
        let (mut lo, mut hi) = (0usize, n);
        if n != 1 {
            let mut mid = n >> 1;
            loop {
                if pct[mid] < t {
                    lo = mid;
                } else {
                    hi = mid;
                }
                mid = (hi + lo) >> 1;
                if hi - lo == 1 {
                    break;
                }
            }
        }
        let p_hi = if hi < n {
            pct[hi]
        } else {
            hi = 0;
            1.0
        };
        let f = (p_hi - t) / (p_hi - pct[lo]);
        let g = 1.0 - f;
        let a = (alpha[hi] as f32 * g + alpha[lo] as f32 * f) as i32;
        if a < 1 {
            return;
        }
        let base = out.verts.len() as u16;
        let (vl, vh) = (&s.verts[lo], &s.verts[hi]);
        let [ax, ay] = w.x_axis;
        let [bx, by] = w.y_axis;
        for (l, h) in vl.iter().zip(vh) {
            let u = h[0] * g + l[0] * f;
            let v = h[1] * g + l[1] * f;
            let x = u * ax + w.x + v * bx;
            let y = u * ay + w.y + v * by;
            out.verts.push([x, y, 0.0]);
            if s.mapping > 0 {
                let m = (s.mapping - 1) as usize;
                out.tex0
                    .push([x * WORLD_TEX_SCALE + xoff[m], y * WORLD_TEX_SCALE + yoff[m]]);
            }
        }
        match s.mapping {
            0 => {
                for (l, h) in s.texs[lo].iter().zip(&s.texs[hi]) {
                    out.tex0.push([g * h[0] + f * l[0], l[1] * f + h[1] * g]);
                }
            }
            m if m < 0 => out.tex0.extend(&s.texs[lo]),
            _ => {}
        }
        out.alpha
            .extend(std::iter::repeat_n((a & 0xff) as u8, vl.len()));
        out.tex1.extend(&s.texs[0]);
        out.faces.extend(s.faces.iter().map(|&i| i + base));
    }
}
