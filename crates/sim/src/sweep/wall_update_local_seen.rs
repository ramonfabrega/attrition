//! `Wall::update_local_seen@0063ed50` under the emulator against the port (item 3304; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Wall::update_local_seen@0063ed50`, run by `tools/emu/sweep_wall_update_local_seen.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 512 random bytes; callee 0x00643440 (2 args) stubbed; callee 0x006b4bb0 (4 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 0 writes a value in 0..40 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in 0..40 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
/// `ever_seen | visible | (1 << owner)`: `+0x62`, the signed byte at `+0x40`, and the owner byte
/// `+0x9` masked to five bits, as the original builds the word it hands `set_seen2`.
fn plain_mask(m: &[u8]) -> i32 {
    i32::from(m[0x62]) | i32::from(m[0x40] as i8) | (1i32 << (m[9] & 0x1f))
}

/// `Wall::update_local_seen@0063ed50`, run by `tools/emu/sweep_wall_update_local_seen.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 512 random bytes; callee 0x00643440 (2 args) stubbed; callee 0x006b4bb0 (4 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); slot +0x50 laid as 0x00472360 or its stub, a flag after the answers; stub 2 answered from [0, 1, 1, 1] (90%); stub 3 answered from [0, 0, 0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 0 writes a value in 0..14 through its argument 0, printed after the answers (`o` in a place); stub 0 writes a value in 0..14 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_update_local_seen.py") else {
        return;
    };
    let mut n = 0;
    let mut checked = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let mut calls: Vec<(i32, i32, i32, i32)> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Stub answers: `a[0]` `WallData::tile_corner` (unused), `a[1]` `World::set_seen2`
            // (unused), `a[2]` slot `+0x2c` (`BuildData::is_wonder`), `a[3]` the type's slot `+0xfc`
            // (`is_fort`), `a[4]` slot `+0x50` (`is_started`), `a[7]` whether slot `+0x50` held the real
            // `is_started` (`flags & 2`), `a[5]`/`a[6]` the corner `tile_corner` writes. Windows: c0 the
            // world (`+0x18`/`+0x1c` its tile extents), p0 the wall's type (`x_size` `+0x234`, `y_size`
            // `+0x238`). There is no crate method over raw bytes, so this is an inline copy; the
            // crate's own `Sim::update_local_seen_build` is held to the same rows below.
            let wonder_slot = a[2] != 0;
            let fort = a[3] != 0;
            let started = if a[7] != 0 { m[8] & 2 != 0 } else { a[4] != 0 };
            let (plane_both, mask) = if wonder_slot && m[8] & 0x20 == 0 && !fort {
                (true, if started { 0xff } else { plain_mask(m) })
            } else {
                (false, plain_mask(m))
            };
            let (cx, cy) = (a[5], a[6]);
            let (xs, ys) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
            let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let (mut t_call, mut t_x, mut t_y) = (Vec::new(), Vec::new(), Vec::new());
            let le = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            let mut i = -1;
            while xs > -2 && i <= xs {
                let mut j = -1;
                while ys > -2 && j <= ys {
                    let (tx, ty) = (cx + i, cy + j);
                    if tx >= 0 && ty >= 0 && tx < w && ty < h {
                        t_call.push(format!(
                            "{j},sp-0xc,sp-0x10,{mask},{}",
                            i32::from(!plane_both)
                        ));
                        calls.push((tx >> 1, ty >> 1, mask, i32::from(!plane_both)));
                        t_x.push(le(tx >> 1));
                        t_y.push(le(ty >> 1));
                    }
                    j += 1;
                }
                i += 1;
            }
            for t in [t_call, t_x, t_y] {
                log.push(if t.is_empty() {
                    "-".to_string()
                } else {
                    t.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_local_seen — the recorded calls, row `{line}`"
        );
        // The crate's own method, on the rows it can be handed: a player owner, a non-negative
        // footprint, and a world whose tile extents (`+0x18`/`+0x1c`) are the fog grid's doubled,
        // which is the only bound `World::set_seen2_only` keeps. The stubbed `set_seen2` is
        // replayed onto an empty grid and the crate's fog must be exactly that.
        {
            let (cx, cy) = (a[5], a[6]);
            let (xs, ys) = (rd(&p[0], 0x234), rd(&p[0], 0x238));
            let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            if m[9] < 8 && xs >= 0 && ys >= 0 && w >= 4 && h >= 4 && w % 4 == 0 && h % 4 == 0 {
                let (fw, fh) = (w / 2, h / 2);
                let mut world = crate::World::new(w / 4, h / 4);
                assert!(world.set_fog(vec![0; (fw * fh) as usize]));
                assert!(world.set_fog_now(vec![0; (fw * fh) as usize]));
                let mut sim = crate::Sim::new(crate::Tuning::RON, world, 8);
                let wonder_slot = a[2] != 0;
                let fort = a[3] != 0;
                let city = m[8] & 0x20 != 0;
                let ident = if city {
                    crate::build::Ident::Village
                } else if fort {
                    crate::build::Ident::Fort
                } else {
                    crate::build::Ident::Farm
                };
                let ty = sim.add_build_type(crate::build::BuildType {
                    ident,
                    wonder: wonder_slot,
                    x_size: xs,
                    y_size: ys,
                    ..crate::build::BuildType::default()
                });
                // A position whose tile is `corner + size / 2`, so `tile_corner` lands on `(cx, cy)`.
                let pos = crate::Pos::new((cx + (xs >> 1)) * 192, (cy + (ys >> 1)) * 192);
                let b = sim.add_building(m[9], pos, 0);
                sim.buildings[b].ty = Some(ty);
                sim.buildings[b].alive = true;
                sim.buildings[b].ever_seen = m[0x62];
                sim.buildings[b].visible = m[0x40];
                sim.buildings[b].started = if a[7] != 0 { m[8] & 2 != 0 } else { a[4] != 0 };
                sim.update_local_seen_build(b);
                let mut seen = vec![0u8; (fw * fh) as usize];
                let mut seen2 = vec![0u8; (fw * fh) as usize];
                for &(x, y, mask, flag) in &calls {
                    let i = (y * fw + x) as usize;
                    seen2[i] |= mask as u8;
                    if flag == 0 {
                        seen[i] |= mask as u8;
                    }
                }
                for y in 0..fh {
                    for x in 0..fw {
                        let i = (y * fw + x) as usize;
                        assert_eq!(
                            (sim.world.seen(x, y), sim.world.seen2(x, y)),
                            (Some(seen[i]), Some(seen2[i])),
                            "update_local_seen — the crate's fog at ({x}, {y}), row `{line}`"
                        );
                    }
                }
                checked += 1;
            }
        }
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_local_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_local_seen — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "update_local_seen — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    assert!(
        checked >= 150,
        "only {checked} rows reached the crate's method"
    );
    eprintln!("Wall::update_local_seen@0063ed50: {n} rows agree, {checked} against the crate");
}
