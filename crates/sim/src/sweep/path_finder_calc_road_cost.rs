//! `PathFinder::calc_road_cost@00686300` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::calc_road_cost@00686300`, run by `tools/emu/sweep_obj.py` on its row `path_finder_calc_road_cost` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 5 argument(s), the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 68 random bytes at 0x00e85ec0; argument 0 names 64 random bytes; callee 0x00a39d70 (2 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x008544a0 (4 args) stubbed; callee 0x006b53f0 (3 args) stubbed; the word at `c0+0x138` points at 128 random bytes less `0` (placed); the word at `c0+0x134` points at 112 random bytes less `0` (placed); the word at `p0+0x20` points at 64 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 answers 0..65535 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 3 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 writes a value in -70000..70000 through its argument 0, printed after the answers (`o` in a place); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x10 (`p` windows after the pointed buffers, before the places); 4 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x10 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_calc_road_cost") else {
        return;
    };
    let obj = super::Obj {
        name: "calc_road_cost",
        full: "PathFinder::calc_road_cost@00686300",
        cells: 3,
        calls: true,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The port's own `roads::road_cost_steps`, its callees answered as the stubs were:
            // `Random::get(0, 0xffff)` (stub 0), `LeaderData::is_ally` (stub 1), `find_tcoord_z`
            // (stub 2, the short it writes through its first argument is `a[9]`) and `was_seen`
            // (stub 3, the half-tile it is handed through two locals). The log holds one token
            // a stub, in stub order, `-` for a callee the arm never reaches.
            use crate::roads::{Leader, RoadTile, RoadWeights, road_cost_steps};
            let tile_of = |x: i32| (x >> 6) / 3;
            let (x, y) = (rd(&p[0], 0), rd(&p[0], 4));
            let (px, py) = (rd(&p[15], 0), rd(&p[15], 4));
            let (tx, ty) = (tile_of(x), tile_of(y));
            let (ptx, pty) = (tile_of(px), tile_of(py));
            let stride = rd(&c[0], 0x18);
            let mask = |tx: i32, ty: i32| {
                let o = 2 * (ty * stride + tx) as usize;
                u16::from_le_bytes([p[13][o], p[13][o + 1]])
            };
            let cell = 0x1c * ((ty >> 2) * rd(&c[0], 0) + (tx >> 2)) as usize;
            let w = RoadWeights {
                base: rd(&c[2], 0x20),
                sea_avoided: rd(&c[2], 0x24),
                sea_enter: rd(&c[2], 0x28),
                foreign: rd(&c[2], 0x2c),
                unowned: rd(&c[2], 0x30),
                ground: rd(&c[2], 0x34),
                rough: rd(&c[2], 0x38),
                climb: rd(&c[2], 0x3c),
            };
            let tok: std::cell::RefCell<[String; 4]> =
                std::cell::RefCell::new(["-".into(), "-".into(), "-".into(), "-".into()]);
            let (total, z_val) = road_cost_steps(
                &w,
                rd(&c[2], 0xc) != 0,
                a[3] & 1 != 0,
                RoadTile {
                    at: (tx, ty),
                    mask: mask(tx, ty),
                    parent_mask: mask(ptx, pty),
                    cell_flags: u16::from(p[14][cell]),
                    owner: i32::from(p[14][cell + 0xf] as i8),
                    parent_z: i32::from(i16::from_le_bytes([p[15][0x1c], p[15][0x1d]])),
                },
                (a[1], a[2]),
                || {
                    tok.borrow_mut()[0] = "14908044,0,65535".into();
                    a[5]
                },
                |owner| {
                    let o = owner as usize;
                    Leader {
                        who: rd(&p[1 + 2 * o], 0),
                        fwd: rd(&p[2 + 2 * o], 4 * a[1] as usize),
                        back: rd(&p[9 + a[1] as usize], 4 * rd(&p[1 + 2 * o], 0) as usize),
                    }
                },
                |owner, other| {
                    tok.borrow_mut()[1] = format!("{},{}", 0xe3a390 + owner * 0x6eec, other);
                    a[6] != 0
                },
                || {
                    tok.borrow_mut()[2] = format!("12628208,sp-0x40,{tx},{ty},0");
                    a[9]
                },
                |hx, hy| {
                    let hex = |v: i32| {
                        v.to_le_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    };
                    tok.borrow_mut()[3] =
                        format!("c0+0x0,sp-0x3c,sp-0x40,{} {} {}", a[1], hex(hx), hex(hy));
                    a[8] != 0
                },
            );
            for t in tok.into_inner() {
                log.extend(t.split_whitespace().map(String::from));
            }
            p[0][0x1c..0x1e].copy_from_slice(&(z_val as i16).to_le_bytes());
            i64::from(total)
        }
    });
}
