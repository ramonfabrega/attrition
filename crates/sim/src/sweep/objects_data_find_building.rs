//! `ObjectsData::find_building@0065d260` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_building@0065d260`, run by `tools/emu/sweep_objects_data_find_building.py` under unicorn
/// (tools/emu): 528 random bytes of object, 9 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p44 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; the global cell 0x00c0ad5c points at 32 random bytes; the global cell 0x00c0ad60 points at 32 random bytes; 8 random bytes at 0x00cab300 (each word in 0..3); vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; the word at `0xc0ab84+0` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 48 random bytes less `0` (placed); the word at `p8+4` points at 48 random bytes less `0` (placed); the word at `p8+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+28` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 48 random bytes less `0` (placed); the word at `p12+4` points at 48 random bytes less `0` (placed); the word at `p12+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+56` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 48 random bytes less `0` (placed); the word at `p16+4` points at 48 random bytes less `0` (placed); the word at `p16+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+84` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 48 random bytes less `0` (placed); the word at `p20+4` points at 48 random bytes less `0` (placed); the word at `p20+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+112` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 48 random bytes less `0` (placed); the word at `p24+4` points at 48 random bytes less `0` (placed); the word at `p24+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+140` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 48 random bytes less `0` (placed); the word at `p28+4` points at 48 random bytes less `0` (placed); the word at `p28+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+168` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 48 random bytes less `0` (placed); the word at `p32+4` points at 48 random bytes less `0` (placed); the word at `p32+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+196` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 48 random bytes less `0` (placed); the word at `p36+4` points at 48 random bytes less `0` (placed); the word at `p36+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84-28` points at 12 random bytes less `0` (placed); the word at `p40+0` points at 48 random bytes less `0` (placed); the word at `p40+4` points at 48 random bytes less `0` (placed); the word at `p40+8` points at 48 random bytes less `0` (placed); the word at `c0+0x134` points at 1024 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 0] (90%); stub 3 answered from [1, 1, 0] (90%); stub 0 answers 24 values in call order, printed after the answers; stub 1 answers 24 values in call order, printed after the answers; stub 2 answers 24 values in call order, printed after the answers; stub 3 answers 24 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `0x006817f0` run once before the rows (a table builder the startup would have run); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_building.py") else {
        return;
    };
    let mut n = 0;
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `ObjectsData::find_building@0065d260` is inline: the crate has no general building finder (its
                // `find_probe_farm` is one caller's exhaustive arm on `Sim::buildings`, and the cell chains and the
                // per-leader object tables are not modelled). The distance is the crate's own `world::vector_dist`
                // and the ring is `ai_place::circle()` (the real `circle_init` runs under the emulator, `--init-call`).
                // Windows: c0 the world (`+0` width, `+4` height, `+0x134` the cells), c1 `div_3_table` (filled
                // `i / 3`), c2/c3 the two count arrays (`0xc0ad5c`, `0xc0ad60`, a word a leader), c4 `obj_base` (the
                // two start indices); p0..p7 the leaders' flag words (bit 0), p8.. the eight type rows of three
                // objects, each row an array window then its objects (`p[8 + 4 t + 1 + j]`), the row before type 0
                // last (`p[41 + j]`), p44 the cells (stride 0x1c: region short `+4`, head index short `+8`, head
                // type short `+10`). An object keeps its point xor 0x63637 at `+0x10`/`+0x14` and the next
                // index/type shorts at `+0x2c`/`+0x2e`. The arm is the radius: negative or past nine rings of 0x300
                // walks every leader's objects (two start/count passes), else the ring of cells walks the chains.
                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                let (px, py, sidx, who, r, flags, filt, p8, p9) =
                    (a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8]);
                let region_on = flags & 0x200 != 0;
                let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
                let div3 = |v: i32| rd(&c[1], 4 * usize::try_from(v).expect("nonneg"));
                let tile: &[u8] = &p[44];
                let want = if region_on {
                    let ix = usize::try_from(div3(py >> 8) * w + div3(px >> 8)).expect("cell");
                    sh(tile, ix * 0x1c + 4)
                } else {
                    -1
                };
                let mut n = [0usize; 4];
                let mut l: [Vec<String>; 4] = Default::default();
                let mut ask = |k: usize, base: usize| {
                    let v = a[base + n[k].min(23)];
                    n[k] += 1;
                    v
                };
                let oidx = |t: i32, j: i32| {
                    usize::try_from(if t < 0 { 41 + j } else { 9 + 4 * t + j }).expect("object")
                };
                // The tile distance of an object from the point, `vector_dist` of the two `div_3_table[v >> 6]` gaps.
                let tdist = |o: &[u8]| {
                    let ox = div3((rd(o, 0x10) ^ 0x6_3637) >> 6);
                    let oy = div3((rd(o, 0x14) ^ 0x6_3637) >> 6);
                    crate::world::vector_dist(ox - div3(px >> 6), oy - div3(py >> 6))
                };
                wr(m, 0x200, who);
                wr(m, 0x1fc, 99_999_999);
                let mut find_dist = 99_999_999;
                let mut find_who = who;
                let mut res = -1;
                let ring_no = r.wrapping_add(0x2ff) / 0x300;
                if r < 0 || ring_no > 9 {
                    for pl in 0..8i32 {
                        if rd(&p[pl as usize], 0) & 1 == 0 {
                            continue;
                        }
                        l[2].push(format!("{pl},{sidx},{who}"));
                        if ask(2, 61) == 0 {
                            continue;
                        }
                        for pass in 0..2usize {
                            let mut i = rd(&c[4], 4 * pass);
                            while i < rd(&c[2 + pass], 4 * pl as usize) {
                                let oi = oidx(pl, i);
                                let name = format!("p{oi}+0x0");
                                let o: &[u8] = &p[oi];
                                i += 1;
                                l[0].push(name.clone());
                                if ask(0, 13) == 0 {
                                    continue;
                                }
                                l[1].push(name);
                                if ask(1, 37) == 0 {
                                    continue;
                                }
                                if filt != 0 && filt.wrapping_sub(1) as u32 <= 0x16 {
                                    l[3].push(format!("{pl},{},{p8},{p9},{}", i - 1, filt - 1));
                                    if ask(3, 85) == 0 {
                                        continue;
                                    }
                                }
                                if region_on {
                                    let ox = div3((rd(o, 0x10) ^ 0x6_3637) >> 8);
                                    let oy = div3((rd(o, 0x14) ^ 0x6_3637) >> 8);
                                    let ix = usize::try_from(oy * w + ox).expect("cell");
                                    if want != sh(tile, ix * 0x1c + 4) {
                                        continue;
                                    }
                                }
                                let d = tdist(o);
                                if d <= find_dist && (r < 0 || d <= r) {
                                    find_dist = d;
                                    find_who = pl;
                                    res = i - 1;
                                }
                            }
                        }
                    }
                } else {
                    let ring = crate::ai_place::circle();
                    let (cx, cy) = (div3(px >> 8), div3(py >> 8));
                    for q in 0..ring.radius[usize::try_from(ring_no).expect("ring")] {
                        let (nx, ny) = (ring.x[q] + cx, ring.y[q] + cy);
                        if nx < 0 || ny < 0 || nx >= w || ny >= h {
                            continue;
                        }
                        let cell = usize::try_from(w * ny + nx).expect("cell") * 0x1c;
                        let mut cur = sh(tile, cell + 8);
                        let mut ty = sh(tile, cell + 10);
                        if cur < 0 || (region_on && want != sh(tile, cell + 4)) {
                            continue;
                        }
                        while cur >= 0 {
                            let oi = oidx(ty, cur);
                            let name = format!("p{oi}+0x0");
                            let o: &[u8] = &p[oi];
                            let (this_ty, this_cur) = (ty, cur);
                            cur = sh(o, 0x2c);
                            ty = sh(o, 0x2e);
                            if this_ty >= 8 {
                                continue;
                            }
                            if this_ty >= 0 {
                                l[2].push(format!("{this_ty},{sidx},{who}"));
                                if ask(2, 61) == 0 {
                                    continue;
                                }
                            }
                            l[0].push(name.clone());
                            if ask(0, 13) == 0 {
                                continue;
                            }
                            l[1].push(name);
                            if ask(1, 37) == 0 {
                                continue;
                            }
                            if filt != 0 && filt.wrapping_sub(1) as u32 <= 0x16 {
                                l[3].push(format!("{this_ty},{this_cur},{p8},{p9},{}", filt - 1));
                                if ask(3, 85) == 0 {
                                    continue;
                                }
                            }
                            let d = tdist(o);
                            if d <= find_dist && d <= r {
                                find_dist = d;
                                find_who = this_ty;
                                res = this_cur;
                            }
                        }
                    }
                }
                wr(m, 0x1fc, find_dist);
                wr(m, 0x200, find_who);
                for v in l {
                    log.push(if v.is_empty() {
                        "-".to_string()
                    } else {
                        v.join("/")
                    });
                }
                i64::from(res)
            }
        };
        assert_eq!(got, want_eax, "find_building — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_building — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_building — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_building — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "find_building — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("ObjectsData::find_building@0065d260: {n} rows agree");
}
