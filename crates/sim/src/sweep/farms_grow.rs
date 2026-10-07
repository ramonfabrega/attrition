//! `Farms::grow@008d91c0` against [`crate::farms::Farm::grow`].

use crate::farms::{CUT, Farm, GROWING, RIPE};

/// Ours for one in-range row: the cell's state and adds after
/// [`Farm::grow`] on cell `dx * 4 + dy`.
fn ours(dy: i64, dx: i64, state0: i64, adds0: i64) -> (u8, i32) {
    let cell = (dx * 4 + dy) as usize;
    let mut farm = Farm::default();
    farm.state[cell] = state0 as u8;
    farm.adds[cell] = adds0 as i32;
    farm.grow(cell);
    (farm.state[cell], farm.adds[cell])
}

/// The rows where the two part: `(farm, dy, dx, state0, adds0, ours,
/// original)`. Every one is `grow` on a **cut** cell that has decayed at
/// least once — the first, `1 2 3 3 198`, a cut cell one `0.01f` under
/// `1.0f`, is ours `0x3f7eb845` and the original's `0x3f7eb852`. The state
/// agrees on all of them (growing, and ripe never); the float does not: the
/// original holds `(1.0f − k·0.01f) + 0.005f`, the port's `201 − 2k` adds
/// stand for `201 − 2k` adds of `0.005f` from zero, a few ulps lower. The
/// port's one caller grows only a growing or an empty cell (`orders.rs`,
/// `do_gather`'s farm branch), so no path of the port reaches these.
#[rustfmt::skip]
const PARTS: &[(i64, i64, i64, i64, i64, i64, i64)] = &[
    (1, 2, 3, 3, 198, 0x1_3f7e_b845, 0x1_3f7e_b852),
    (2, 3, 0, 3, 196, 0x1_3f7c_28e9, 0x1_3f7c_28f6),
    (3, 0, 1, 3, 194, 0x1_3f79_998d, 0x1_3f79_999a),
    (4, 1, 2, 3, 192, 0x1_3f77_0a31, 0x1_3f77_0a3e),
    (5, 2, 3, 3, 190, 0x1_3f74_7ad5, 0x1_3f74_7ae2),
    (6, 3, 0, 3, 188, 0x1_3f71_eb79, 0x1_3f71_eb86),
    (7, 0, 1, 3, 186, 0x1_3f6f_5c1d, 0x1_3f6f_5c2a),
    (8, 1, 2, 3, 184, 0x1_3f6c_ccc1, 0x1_3f6c_ccce),
    (9, 2, 3, 3, 182, 0x1_3f6a_3d65, 0x1_3f6a_3d72),
    (10, 3, 0, 3, 180, 0x1_3f67_ae09, 0x1_3f67_ae16),
    (0, 0, 1, 3, 178, 0x1_3f65_1ead, 0x1_3f65_1eba),
    (1, 1, 2, 3, 176, 0x1_3f62_8f51, 0x1_3f62_8f5e),
    (2, 2, 3, 3, 174, 0x1_3f5f_fff5, 0x1_3f60_0002),
    (3, 3, 0, 3, 172, 0x1_3f5d_7099, 0x1_3f5d_70a6),
    (4, 0, 1, 3, 170, 0x1_3f5a_e13d, 0x1_3f5a_e14a),
    (5, 1, 2, 3, 168, 0x1_3f58_51e1, 0x1_3f58_51ee),
    (6, 2, 3, 3, 166, 0x1_3f55_c285, 0x1_3f55_c292),
    (7, 3, 0, 3, 164, 0x1_3f53_3329, 0x1_3f53_3336),
    (8, 0, 1, 3, 162, 0x1_3f50_a3cd, 0x1_3f50_a3da),
    (9, 1, 2, 3, 160, 0x1_3f4e_1471, 0x1_3f4e_147e),
    (10, 2, 3, 3, 158, 0x1_3f4b_8515, 0x1_3f4b_8522),
    (0, 3, 0, 3, 156, 0x1_3f48_f5b9, 0x1_3f48_f5c6),
    (1, 0, 1, 3, 154, 0x1_3f46_665d, 0x1_3f46_666a),
    (2, 1, 2, 3, 152, 0x1_3f43_d701, 0x1_3f43_d70e),
    (3, 2, 3, 3, 150, 0x1_3f41_47a5, 0x1_3f41_47b2),
    (4, 3, 0, 3, 148, 0x1_3f3e_b849, 0x1_3f3e_b856),
    (5, 0, 1, 3, 146, 0x1_3f3c_28ed, 0x1_3f3c_28fa),
    (6, 1, 2, 3, 144, 0x1_3f39_9991, 0x1_3f39_999e),
    (7, 2, 3, 3, 142, 0x1_3f37_0a35, 0x1_3f37_0a42),
    (8, 3, 0, 3, 140, 0x1_3f34_7ad9, 0x1_3f34_7ae6),
    (9, 0, 1, 3, 138, 0x1_3f31_eb7d, 0x1_3f31_eb8a),
    (10, 1, 2, 3, 136, 0x1_3f2f_5c21, 0x1_3f2f_5c2e),
    (0, 2, 3, 3, 134, 0x1_3f2c_ccc5, 0x1_3f2c_ccd2),
    (1, 3, 0, 3, 132, 0x1_3f2a_3d69, 0x1_3f2a_3d76),
    (2, 0, 1, 3, 130, 0x1_3f27_ae0d, 0x1_3f27_ae1a),
    (3, 1, 2, 3, 128, 0x1_3f25_1eb1, 0x1_3f25_1ebe),
    (4, 2, 3, 3, 126, 0x1_3f22_8f55, 0x1_3f22_8f62),
    (5, 3, 0, 3, 124, 0x1_3f1f_fff9, 0x1_3f20_0006),
    (6, 0, 1, 3, 122, 0x1_3f1d_709d, 0x1_3f1d_70aa),
    (7, 1, 2, 3, 120, 0x1_3f1a_e141, 0x1_3f1a_e14e),
    (8, 2, 3, 3, 118, 0x1_3f18_51e5, 0x1_3f18_51f2),
    (9, 3, 0, 3, 116, 0x1_3f15_c289, 0x1_3f15_c296),
    (10, 0, 1, 3, 114, 0x1_3f13_332d, 0x1_3f13_333a),
    (0, 1, 2, 3, 112, 0x1_3f10_a3d1, 0x1_3f10_a3de),
    (1, 2, 3, 3, 110, 0x1_3f0e_1475, 0x1_3f0e_1482),
    (2, 3, 0, 3, 108, 0x1_3f0b_8519, 0x1_3f0b_8526),
    (3, 0, 1, 3, 106, 0x1_3f08_f5bd, 0x1_3f08_f5ca),
    (4, 1, 2, 3, 104, 0x1_3f06_6661, 0x1_3f06_666e),
    (5, 2, 3, 3, 102, 0x1_3f03_d705, 0x1_3f03_d712),
    (6, 3, 0, 3, 100, 0x1_3f01_47a9, 0x1_3f01_47b6),
    (7, 0, 1, 3, 98, 0x1_3efd_7099, 0x1_3efd_70b4),
    (8, 1, 2, 3, 96, 0x1_3ef8_51e1, 0x1_3ef8_51fc),
    (9, 2, 3, 3, 94, 0x1_3ef3_3329, 0x1_3ef3_3344),
    (10, 3, 0, 3, 92, 0x1_3eee_1471, 0x1_3eee_148c),
    (0, 0, 1, 3, 90, 0x1_3ee8_f5b9, 0x1_3ee8_f5d4),
    (1, 1, 2, 3, 88, 0x1_3ee3_d701, 0x1_3ee3_d71c),
    (2, 2, 3, 3, 86, 0x1_3ede_b849, 0x1_3ede_b864),
    (3, 3, 0, 3, 84, 0x1_3ed9_9991, 0x1_3ed9_99ac),
    (4, 0, 1, 3, 82, 0x1_3ed4_7ad9, 0x1_3ed4_7af4),
    (5, 1, 2, 3, 80, 0x1_3ecf_5c21, 0x1_3ecf_5c3c),
    (6, 2, 3, 3, 78, 0x1_3eca_3d69, 0x1_3eca_3d84),
    (7, 3, 0, 3, 76, 0x1_3ec5_1eb1, 0x1_3ec5_1ecc),
    (8, 0, 1, 3, 74, 0x1_3ebf_fff9, 0x1_3ec0_0014),
    (9, 1, 2, 3, 72, 0x1_3eba_e141, 0x1_3eba_e15c),
    (10, 2, 3, 3, 70, 0x1_3eb5_c289, 0x1_3eb5_c2a4),
    (0, 3, 0, 3, 68, 0x1_3eb0_a3d1, 0x1_3eb0_a3ec),
    (1, 0, 1, 3, 66, 0x1_3eab_8519, 0x1_3eab_8534),
    (2, 1, 2, 3, 64, 0x1_3ea6_6661, 0x1_3ea6_667c),
    (3, 2, 3, 3, 62, 0x1_3ea1_47a9, 0x1_3ea1_47c4),
    (4, 3, 0, 3, 60, 0x1_3e9c_28f1, 0x1_3e9c_290c),
    (5, 0, 1, 3, 58, 0x1_3e97_0a39, 0x1_3e97_0a54),
    (6, 1, 2, 3, 56, 0x1_3e91_eb81, 0x1_3e91_eb9c),
    (7, 2, 3, 3, 54, 0x1_3e8c_ccc9, 0x1_3e8c_cce4),
    (8, 3, 0, 3, 52, 0x1_3e87_ae11, 0x1_3e87_ae2c),
    (9, 0, 1, 3, 50, 0x1_3e82_8f59, 0x1_3e82_8f74),
    (10, 1, 2, 3, 48, 0x1_3e7a_e141, 0x1_3e7a_e177),
    (0, 2, 3, 3, 46, 0x1_3e70_a3d1, 0x1_3e70_a406),
    (1, 3, 0, 3, 44, 0x1_3e66_6661, 0x1_3e66_6695),
    (2, 0, 1, 3, 42, 0x1_3e5c_28f1, 0x1_3e5c_2924),
    (3, 1, 2, 3, 40, 0x1_3e51_eb81, 0x1_3e51_ebb3),
    (4, 2, 3, 3, 38, 0x1_3e47_ae11, 0x1_3e47_ae42),
    (5, 3, 0, 3, 36, 0x1_3e3d_70a1, 0x1_3e3d_70d1),
    (6, 0, 1, 3, 34, 0x1_3e33_3331, 0x1_3e33_3360),
    (7, 1, 2, 3, 32, 0x1_3e28_f5c1, 0x1_3e28_f5ef),
    (8, 2, 3, 3, 30, 0x1_3e1e_b851, 0x1_3e1e_b87e),
    (9, 3, 0, 3, 28, 0x1_3e14_7ae1, 0x1_3e14_7b0d),
    (10, 0, 1, 3, 26, 0x1_3e0a_3d71, 0x1_3e0a_3d9c),
    (0, 1, 2, 3, 24, 0x1_3e00_0001, 0x1_3e00_002c),
    (1, 2, 3, 3, 22, 0x1_3deb_8521, 0x1_3deb_8577),
    (2, 3, 0, 3, 20, 0x1_3dd7_0a3f, 0x1_3dd7_0a96),
    (3, 0, 1, 3, 18, 0x1_3dc2_8f5d, 0x1_3dc2_8fb5),
    (4, 1, 2, 3, 16, 0x1_3dae_147b, 0x1_3dae_14d4),
    (5, 2, 3, 3, 14, 0x1_3d99_9999, 0x1_3d99_99f3),
    (6, 3, 0, 3, 12, 0x1_3d85_1eb7, 0x1_3d85_1f12),
    (7, 0, 1, 3, 10, 0x1_3d61_47ac, 0x1_3d61_4861),
    (8, 1, 2, 3, 8, 0x1_3d38_51ea, 0x1_3d38_529f),
    (9, 2, 3, 3, 6, 0x1_3d0f_5c28, 0x1_3d0f_5cdd),
    (10, 3, 0, 3, 4, 0x1_3ccc_cccc, 0x1_3ccc_ce34),
    (0, 0, 1, 3, 2, 0x1_3c75_c28f, 0x1_3c75_c55f),
    (1, 1, 2, 3, 0, 0x1_3ba3_d70a, 0x1_3ba3_dcaa),
    (34, 2, 1, 3, 134, 0x1_3f2c_ccc5, 0x1_3f2c_ccd2),
    (32, 1, 3, 3, 2, 0x1_3c75_c28f, 0x1_3c75_c55f),
    (1, 2, 1, 3, 186, 0x1_3f6f_5c1d, 0x1_3f6f_5c2a),
    (20, 1, 2, 3, 20, 0x1_3dd7_0a3f, 0x1_3dd7_0a96),
    (0, 0, 2, 3, 24, 0x1_3e00_0001, 0x1_3e00_002c),
    (61, 2, 1, 3, 38, 0x1_3e47_ae11, 0x1_3e47_ae42),
    (63, 0, 2, 3, 86, 0x1_3ede_b849, 0x1_3ede_b864),
    (14, 0, 3, 3, 118, 0x1_3f18_51e5, 0x1_3f18_51f2),
    (60, 0, 1, 3, 126, 0x1_3f22_8f55, 0x1_3f22_8f62),
    (34, 1, 3, 3, 78, 0x1_3eca_3d69, 0x1_3eca_3d84),
    (63, 1, 1, 3, 128, 0x1_3f25_1eb1, 0x1_3f25_1ebe),
    (25, 0, 3, 3, 156, 0x1_3f48_f5b9, 0x1_3f48_f5c6),
    (26, 1, 3, 3, 8, 0x1_3d38_51ea, 0x1_3d38_529f),
    (45, 0, 0, 3, 22, 0x1_3deb_8521, 0x1_3deb_8577),
    (41, 1, 2, 3, 178, 0x1_3f65_1ead, 0x1_3f65_1eba),
    (4, 3, 3, 3, 22, 0x1_3deb_8521, 0x1_3deb_8577),
    (15, 3, 1, 3, 42, 0x1_3e5c_28f1, 0x1_3e5c_2924),
    (34, 3, 2, 3, 154, 0x1_3f46_665d, 0x1_3f46_666a),
    (54, 2, 1, 3, 22, 0x1_3deb_8521, 0x1_3deb_8577),
    (34, 0, 2, 3, 78, 0x1_3eca_3d69, 0x1_3eca_3d84),
    (51, 0, 0, 3, 26, 0x1_3e0a_3d71, 0x1_3e0a_3d9c),
    (54, 3, 3, 3, 100, 0x1_3f01_47a9, 0x1_3f01_47b6),
    (17, 2, 3, 3, 94, 0x1_3ef3_3329, 0x1_3ef3_3344),
    (20, 3, 2, 3, 94, 0x1_3ef3_3329, 0x1_3ef3_3344),
    (0, 1, 3, 3, 182, 0x1_3f6a_3d65, 0x1_3f6a_3d72),
    (58, 3, 2, 3, 120, 0x1_3f1a_e141, 0x1_3f1a_e14e),
    (42, 2, 2, 3, 170, 0x1_3f5a_e13d, 0x1_3f5a_e14a),
    (35, 0, 2, 3, 174, 0x1_3f5f_fff5, 0x1_3f60_0002),
    (30, 3, 3, 3, 136, 0x1_3f2f_5c21, 0x1_3f2f_5c2e),
    (41, 1, 0, 3, 184, 0x1_3f6c_ccc1, 0x1_3f6c_ccce),
    (37, 0, 2, 3, 12, 0x1_3d85_1eb7, 0x1_3d85_1f12),
    (12, 1, 0, 3, 182, 0x1_3f6a_3d65, 0x1_3f6a_3d72),
    (33, 3, 1, 3, 92, 0x1_3eee_1471, 0x1_3eee_148c),
    (50, 2, 0, 3, 44, 0x1_3e66_6661, 0x1_3e66_6695),
    (51, 2, 1, 3, 122, 0x1_3f1d_709d, 0x1_3f1d_70aa),
    (57, 0, 3, 3, 78, 0x1_3eca_3d69, 0x1_3eca_3d84),
    (45, 3, 2, 3, 166, 0x1_3f55_c285, 0x1_3f55_c292),
    (56, 3, 3, 3, 46, 0x1_3e70_a3d1, 0x1_3e70_a406),
    (1, 1, 1, 3, 182, 0x1_3f6a_3d65, 0x1_3f6a_3d72),
    (19, 0, 1, 3, 76, 0x1_3ec5_1eb1, 0x1_3ec5_1ecc),
    (42, 1, 0, 3, 100, 0x1_3f01_47a9, 0x1_3f01_47b6),
    (4, 3, 1, 3, 182, 0x1_3f6a_3d65, 0x1_3f6a_3d72),
    (43, 1, 1, 3, 140, 0x1_3f34_7ad9, 0x1_3f34_7ae6),
    (9, 3, 3, 3, 136, 0x1_3f2f_5c21, 0x1_3f2f_5c2e),
    (55, 1, 0, 3, 118, 0x1_3f18_51e5, 0x1_3f18_51f2),
    (21, 0, 2, 3, 64, 0x1_3ea6_6661, 0x1_3ea6_667c),
    (32, 0, 3, 3, 168, 0x1_3f58_51e1, 0x1_3f58_51ee),
    (3, 1, 3, 3, 46, 0x1_3e70_a3d1, 0x1_3e70_a406),
    (8, 0, 1, 3, 88, 0x1_3ee3_d701, 0x1_3ee3_d71c),
    (39, 0, 3, 3, 166, 0x1_3f55_c285, 0x1_3f55_c292),
    (20, 1, 0, 3, 76, 0x1_3ec5_1eb1, 0x1_3ec5_1ecc),
    (46, 0, 2, 3, 22, 0x1_3deb_8521, 0x1_3deb_8577),
    (53, 3, 2, 3, 26, 0x1_3e0a_3d71, 0x1_3e0a_3d9c),
    (21, 3, 3, 3, 144, 0x1_3f39_9991, 0x1_3f39_999e),
    (6, 0, 0, 3, 156, 0x1_3f48_f5b9, 0x1_3f48_f5c6),
    (19, 3, 1, 3, 2, 0x1_3c75_c28f, 0x1_3c75_c55f),
    (12, 0, 1, 3, 160, 0x1_3f4e_1471, 0x1_3f4e_147e),
    (25, 0, 2, 3, 150, 0x1_3f41_47a5, 0x1_3f41_47b2),
    (39, 3, 0, 3, 86, 0x1_3ede_b849, 0x1_3ede_b864),
    (10, 3, 2, 3, 136, 0x1_3f2f_5c21, 0x1_3f2f_5c2e),
    (45, 2, 3, 3, 174, 0x1_3f5f_fff5, 0x1_3f60_0002),
    (59, 3, 3, 3, 4, 0x1_3ccc_cccc, 0x1_3ccc_ce34),
    (52, 0, 3, 3, 90, 0x1_3ee8_f5b9, 0x1_3ee8_f5d4),
    (36, 3, 2, 3, 38, 0x1_3e47_ae11, 0x1_3e47_ae42),
    (41, 1, 2, 3, 12, 0x1_3d85_1eb7, 0x1_3d85_1f12),
];

/// `Farms::grow@008d91c0`, run by `tools/emu/sweep_farms_grow.py`: every row
/// `<farm> <dy> <dx> <state0> <adds0> -> <state1 · 2³² + percent1's bits>`
/// is the original's answer, and [`Farm::grow`] on cell `dx * 4 + dy` must
/// give the same state and the same float — or be one of [`PARTS`], with
/// both answers as written there. A row with `dy` or `dx` at or past 4
/// prints the bytes the call changed: the original's guard says none, and
/// the port has no such cell — its one caller clamps the farmer's cell into
/// the farm first (`orders.rs`, `do_gather`'s farm branch).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_grow.py") else {
        return;
    };
    // The port's `(state, adds)` as the float the original holds: `adds`
    // single-precision adds of `0.005f` from zero, the exact `1.0f` for a
    // ripe cell, `1.0f` less `(200 − adds) / 2` subtractions of `0.01f` for
    // a cut one (`crates/sim/src/farms.rs`' module comment). Inside the
    // `#[test]`, where the no-float lint stands aside.
    let percent_bits = |state: u8, adds: i32| -> u32 {
        let v = match state {
            RIPE => 1.0f32,
            CUT => (0..(200 - adds) / 2).fold(1.0f32, |v, _| v - 0.01f32),
            _ => (0..adds).fold(0.0f32, |v, _| v + 0.005f32),
        };
        v.to_bits()
    };
    let (mut agreed, mut parted) = (0, 0);
    for line in rows.lines() {
        let (args, original) = super::row(line);
        let [farm, dy, dx, state0, adds0] = args[..] else {
            panic!("row {line:?}: five arguments");
        };
        if dy >= 4 || dx >= 4 {
            assert_eq!(original, 0, "row {line:?}: the guard wrote");
            agreed += 1;
            continue;
        }
        let (state, adds) = ours(dy, dx, state0, adds0);
        assert!(state == GROWING || state == RIPE, "row {line:?}");
        let ours = (i64::from(state) << 32) | i64::from(percent_bits(state, adds));
        let part = PARTS
            .iter()
            .find(|p| (p.0, p.1, p.2, p.3, p.4) == (farm, dy, dx, state0, adds0));
        match part {
            Some(&(.., our, their)) => {
                assert_ne!(our, their, "row {line:?}: a part that agrees");
                assert_eq!(
                    (ours, original),
                    (our, their),
                    "row {line:?}: the part moved"
                );
                parted += 1;
            }
            None => assert_eq!(
                ours,
                original,
                "row {line:?}: ours state {} bits {:#x}, original state {} bits {:#x}",
                ours >> 32,
                ours & 0xffff_ffff,
                original >> 32,
                original & 0xffff_ffff
            ),
        }
        if part.is_none() {
            agreed += 1;
        }
    }
    eprintln!("{agreed} rows agree, {parted} part");
    assert_eq!(parted, PARTS.len(), "every part is a row");
    assert!(agreed >= 200, "{agreed} rows");
}
