//! The card probe's records (item 4165): what the original hands the device
//! at each draw, read from a `rontrace.log` written by a `RON_CARD_PROBE`
//! build of `tools/trace/` (`tools/trace/card.h` has the record layout).
//!
//! The executable never calls Direct3D: `d3dgl.dll`'s `D3D11GLSys` turns the
//! fixed-function state into a pixel-shader permutation and four constant
//! buffers. At its `render` entry the probe copies `GraphSys::current_state`
//! (the `RenderState`), the context's shadows of `PerScene`, `PerObject`,
//! `PerMaterial` and `LightData`, the card's `mShaderOptions`, and each
//! stage's texture by its cache entry's name. `docs/RENDERER.md` §3.14 is
//! what the records establish.
//!
//! The words are kept as the log holds them; a float is its bits until a
//! caller asks for it.

/// The record kind the probe writes.
pub const K_CARD: u32 = 40;

/// `ShaderCompilationOptions`, by value (`sbl/d3dgl.pdb`, its enum's field
/// list): the defines a pixel shader permutation is compiled with.
pub const OPTIONS: [&str; 32] = [
    "DEFAULT",
    "ENABLE_LIGHTING",
    "ENABLE_FOG",
    "ENABLE_ALPHA_TEST",
    "ENABLE_TEXTURE_FACTOR",
    "DISABLE_WORLD_TRANSFORM",
    "DISABLE_VIEW_TRANSFORM",
    "ENABLE_VERTEX_COLOR_DIFFUSE",
    "ENABLE_VERTEX_COLOR_SPECULAR",
    "TRANSFORM_SCREEN_SPACE",
    "USE_UV0",
    "USE_UV1",
    "USE_UV2",
    "POSITION_XYZ",
    "TEX0_USE_UV1",
    "TEX1_USE_UV1",
    "HAS_TEX0",
    "HAS_TEX1",
    "NORMAL_MAP",
    "SPEC_MAP",
    "GLOSS_MAP",
    "TL_FAKE_LIGHT",
    "DELTA_UV",
    "ANIMATE_WATER_NORMALS",
    "SPEC_MODEL_TORRANCE",
    "SCREEN_REFLECTIONS",
    "NO_HDR",
    "TEX0_NOT_BOUND",
    "TEX1_NOT_BOUND",
    "DIFFUSE_AS_SPEC",
    "POINT_SPRITE",
    "NUKE_FLASH",
];

/// `TextureOp` (`rise.pdb`): `TOP_SELECTARG2` and `TOP_BLENDTEXTUREALPHA`,
/// the two a figure's stage 0 takes.
pub const TOP_SELECTARG2: i32 = 2;
pub const TOP_MODULATE: i32 = 3;
/// `TOP_MODULATE2X` (`sbl/d3dgl.pdb`'s enumerator): the ocean floor's
/// stage 0 (item 4168).
pub const TOP_MODULATE2X: i32 = 4;
pub const TOP_BLENDTEXTUREALPHA: i32 = 12;
/// `TextureArg`: `TA_CURRENT`, `TA_DIFFUSE`, `TA_TEXTURE`, `TA_TFACTOR`.
pub const TA_CURRENT: i32 = 0;
pub const TA_DIFFUSE: i32 = 1;
pub const TA_TEXTURE: i32 = 5;
pub const TA_TFACTOR: i32 = 6;

/// One stream locked by `get_internal_stream` and dumped at its release.
#[derive(Clone, Debug, Default)]
pub struct Stream {
    pub lit: u32,
    pub vertices: u32,
    pub stride: u32,
    /// The draw index (in its pass) the stream was released before.
    pub before: u32,
    pub bytes: Vec<u8>,
}

/// One `D3D11GLSys::render` call and the device's state at it.
#[derive(Clone, Debug)]
pub struct CardDraw {
    pub pass: u32,
    /// The sim-frame the pass was drawn after.
    pub frame: i32,
    /// The draw's place in its pass.
    pub index: u32,
    /// 0 the indexed `render`, 1 the plain one.
    pub site: u32,
    /// The call's stack arguments, read where the call left them.
    pub args: [u32; 5],
    pub args_checked: bool,
    /// The executable's return addresses above d3dgl's wrapper.
    pub frames: [u32; 4],
    /// `GraphSys::current_state`, `RenderState` +0 .. +0x168.
    pub state: Vec<u8>,
    /// `D3D11Context` +0x70 .. +0x2c0: `PerScene`, `PerObject`,
    /// `PerMaterial`, `LightData`.
    pub cbuf: Vec<u8>,
    pub options: Vec<u32>,
    pub light_enabled: bool,
    pub textures: [u32; 2],
    pub names: [String; 2],
    pub stream: Option<Stream>,
}

impl CardDraw {
    fn i32_at(&self, off: usize) -> i32 {
        i32::from_le_bytes(self.state[off..off + 4].try_into().unwrap())
    }
    fn f32_at(&self, off: usize) -> f32 {
        f32::from_le_bytes(self.cbuf[off..off + 4].try_into().unwrap())
    }
    fn f32s<const N: usize>(&self, off: usize) -> [f32; N] {
        std::array::from_fn(|i| self.f32_at(off + 4 * i))
    }
    /// `RenderState::enables`.
    pub fn enables(&self) -> u32 {
        self.i32_at(0x50) as u32
    }
    /// `RenderState::tfactor`, its bytes red, green, blue, alpha.
    pub fn tfactor(&self) -> [u8; 4] {
        self.state[0x5c..0x60].try_into().unwrap()
    }
    pub fn src_blend(&self) -> i32 {
        self.i32_at(0x74)
    }
    pub fn dst_blend(&self) -> i32 {
        self.i32_at(0x78)
    }
    pub fn alpha_func(&self) -> i32 {
        self.i32_at(0x7c)
    }
    pub fn alpha_ref(&self) -> i32 {
        self.i32_at(0x80)
    }
    /// Stage `s`'s colour operation and its two arguments.
    pub fn colour_op(&self, s: usize) -> (i32, i32, i32) {
        (
            self.i32_at(0xac + 4 * s),
            self.i32_at(0xb4 + 4 * s),
            self.i32_at(0xbc + 4 * s),
        )
    }
    /// Stage `s`'s alpha operation and its two arguments.
    pub fn alpha_op(&self, s: usize) -> (i32, i32, i32) {
        (
            self.i32_at(0xc4 + 4 * s),
            self.i32_at(0xcc + 4 * s),
            self.i32_at(0xd4 + 4 * s),
        )
    }
    /// `PerScene::gView` and `gProjection`, as the buffer holds them: the
    /// translation in each row's fourth word.
    pub fn view(&self) -> [f32; 16] {
        self.f32s(0)
    }
    pub fn projection(&self) -> [f32; 16] {
        self.f32s(0x40)
    }
    /// `PerScene::gAmbientLight`.
    pub fn scene_ambient(&self) -> [f32; 4] {
        self.f32s(0xc0)
    }
    /// `PerObject::gWorld`.
    pub fn world(&self) -> [f32; 16] {
        self.f32s(0xe0)
    }
    /// `PerMaterial`'s ambient, diffuse, specular, emissive and texture factor.
    pub fn material(&self) -> [[f32; 4]; 5] {
        std::array::from_fn(|i| self.f32s(0x160 + 0x10 * i))
    }
    /// `LightData::gLight`: direction, ambient, diffuse, specular.
    pub fn light(&self) -> [[f32; 4]; 4] {
        std::array::from_fn(|i| self.f32s(0x1f0 + 0x10 * i))
    }
    /// The draw's point in the world: `gWorld`'s translation.
    pub fn origin(&self) -> [f32; 3] {
        let w = self.world();
        [w[3], w[7], w[11]]
    }
    /// The options by name.
    pub fn option_names(&self) -> Vec<&'static str> {
        self.options
            .iter()
            .map(|&o| OPTIONS.get(o as usize).copied().unwrap_or("?"))
            .collect()
    }
    pub fn has(&self, name: &str) -> bool {
        self.option_names().contains(&name)
    }
    /// A file name's last component, without its directory.
    pub fn file(&self, stage: usize) -> &str {
        let n = &self.names[stage];
        n.rsplit(['\\', '/']).next().unwrap_or(n)
    }
    /// The draw's world point on a `w`×`h` screen, through the pass's own
    /// view and projection (`(x′ + 1)·w/2`, `(1 − y′)·h/2`).
    pub fn screen(&self, p: [f32; 3], w: f32, h: f32) -> (f32, f32) {
        let mv = |m: &[f32; 16], v: [f32; 4]| -> [f32; 4] {
            std::array::from_fn(|r| (0..4).map(|k| m[4 * r + k] * v[k]).sum())
        };
        let c = mv(
            &self.projection(),
            mv(&self.view(), [p[0], p[1], p[2], 1.0]),
        );
        ((c[0] / c[3] + 1.0) * w / 2.0, (1.0 - c[1] / c[3]) * h / 2.0)
    }
}

fn name_of(raw: &[u8]) -> String {
    let utf16 = raw.len() > 1 && raw[1] == 0 && raw[0] != 0;
    let s: String = if utf16 {
        let w: Vec<u16> = raw
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        String::from_utf16_lossy(&w)
    } else {
        raw.iter().map(|&b| b as char).collect()
    };
    s.split('\0').next().unwrap_or("").to_string()
}

/// Every draw the log's card records hold, in order.
pub fn draws(bytes: &[u8]) -> Vec<CardDraw> {
    let mut out: Vec<CardDraw> = Vec::new();
    let mut names = [vec![0u8; 132], vec![0u8; 132]];
    let mut pending: Option<Stream> = None;
    let mut nopt = 0usize;
    let finish = |d: &mut CardDraw, names: &mut [Vec<u8>; 2], nopt: usize| {
        d.options.truncate(nopt);
        for (name, raw) in d.names.iter_mut().zip(names.iter_mut()) {
            *name = name_of(&raw[..128]);
            raw.iter_mut().for_each(|b| *b = 0);
        }
    };
    for rec in bytes.as_chunks::<32>().0 {
        let w: [u32; 8] =
            std::array::from_fn(|i| u32::from_le_bytes(rec[4 * i..4 * i + 4].try_into().unwrap()));
        if w[0] != K_CARD {
            continue;
        }
        let put = |buf: &mut Vec<u8>, off: u32, words: &[u32]| {
            let off = off as usize;
            for (j, v) in words.iter().enumerate() {
                let at = off + 4 * j;
                if at + 4 <= buf.len() {
                    buf[at..at + 4].copy_from_slice(&v.to_le_bytes());
                }
            }
        };
        match w[1] {
            0 => {
                if let Some(d) = out.last_mut() {
                    finish(d, &mut names, nopt);
                }
                nopt = 0;
                out.push(CardDraw {
                    pass: w[4],
                    frame: w[7] as i32,
                    index: w[3],
                    site: w[2] & 1,
                    args: [0; 5],
                    args_checked: w[2] >> 31 == 1,
                    frames: [0; 4],
                    state: vec![0; 0x170],
                    cbuf: vec![0; 0x250],
                    options: Vec::new(),
                    light_enabled: false,
                    textures: [0; 2],
                    names: [String::new(), String::new()],
                    stream: pending.take(),
                });
            }
            11 => {
                let n = (w[3].saturating_mul(w[5])).min(0x10000) as usize;
                pending = Some(Stream {
                    lit: w[2],
                    vertices: w[3],
                    stride: w[5],
                    before: out.last().map_or(0, |d| d.index + 1),
                    bytes: vec![0; n],
                });
            }
            12 => {
                if let Some(s) = pending.as_mut() {
                    put(&mut s.bytes, w[2], &w[3..7]);
                }
            }
            _ => {
                let Some(d) = out.last_mut() else { continue };
                match w[1] {
                    1 => d.args.copy_from_slice(&w[2..7]),
                    2 => put(&mut d.state, w[2], &w[3..7]),
                    3 => put(&mut d.cbuf, w[2], &w[3..7]),
                    4 => {
                        nopt = w[2] as usize;
                        d.textures = [w[4], w[5]];
                        d.light_enabled = w[6] & 0xff != 0;
                    }
                    5 => d.options.extend_from_slice(&w[3..7]),
                    6 => {
                        if let Some(buf) = names.get_mut(w[2] as usize) {
                            put(buf, w[3], &w[4..7]);
                        }
                    }
                    7 => d.frames.copy_from_slice(&w[2..6]),
                    _ => {}
                }
            }
        }
    }
    if let Some(d) = out.last_mut() {
        finish(d, &mut names, nopt);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(w: [u32; 8]) -> Vec<u8> {
        w.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    /// A draw's records read back field by field, and a record of another
    /// kind between them ignored.
    #[test]
    fn a_draw_reads_back() {
        let mut b = Vec::new();
        b.extend(rec([K_CARD, 0, 1 << 31, 7, 1, 0x91a00d, 0, 3279]));
        b.extend(rec([K_CARD, 1, 0, 0xa9, 0x30fdc524, 2, 0, 3279]));
        b.extend(rec([2, 3279, 0, 0, 0, 0, 0, 3279]));
        // enables at +0x50, tfactor at +0x5c
        let tf = u32::from_le_bytes([0x22, 0x52, 0xff, 0xff]);
        b.extend(rec([K_CARD, 2, 0x50, 0x287, 0, 0, tf, 3279]));
        // stage 0 BLENDTEXTUREALPHA at +0xac
        b.extend(rec([K_CARD, 2, 0xac, 12, 3, 5, 0, 3279]));
        b.extend(rec([
            K_CARD,
            3,
            0xe0,
            1.0f32.to_bits(),
            0,
            0,
            4104.0f32.to_bits(),
            3279,
        ]));
        b.extend(rec([K_CARD, 4, 6, 0, 0x1000, 0x1000, 1, 3279]));
        b.extend(rec([K_CARD, 5, 0, 0, 3, 10, 11, 3279]));
        b.extend(rec([K_CARD, 5, 4, 13, 1, 15, 0, 3279]));
        b.extend(rec([
            K_CARD,
            6,
            0,
            0,
            u32::from_le_bytes(*b"inf."),
            u32::from_le_bytes(*b"tga\0"),
            0,
            3279,
        ]));
        let d = &draws(&b)[0];
        assert_eq!(
            (d.pass, d.index, d.frame, d.site, d.args_checked),
            (1, 7, 3279, 0, true)
        );
        assert_eq!(d.args[1], 0xa9);
        assert_eq!(d.enables(), 0x287);
        assert_eq!(d.tfactor(), [0x22, 0x52, 0xff, 0xff]);
        assert_eq!(d.colour_op(0), (12, 5, 0));
        assert_eq!(d.colour_op(1), (3, 0, 0));
        assert_eq!(d.origin()[0], 4104.0);
        assert_eq!(
            d.option_names(),
            [
                "DEFAULT",
                "ENABLE_ALPHA_TEST",
                "USE_UV0",
                "USE_UV1",
                "POSITION_XYZ",
                "ENABLE_LIGHTING"
            ]
        );
        assert_eq!(d.file(0), "inf.tga");
        assert!(d.light_enabled);
    }
}
