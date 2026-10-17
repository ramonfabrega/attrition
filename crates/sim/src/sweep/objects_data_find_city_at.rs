//! `ObjectsData::find_city_at@0065b870` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_city_at@0065b870`, run by `tools/emu/sweep_objects_data_find_city_at.py` under unicorn
/// (tools/emu): 528 random bytes of object, 5 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p42 the place(s), in flag order; the global cell 0x00cae5fc points at 512 random bytes; 320 random bytes at 0x00c097e8; callee 0x006b3b80 (3 args) stubbed; callee 0x00738410 (0 args) stubbed; callee 0x00739390 (3 args) stubbed; vtable slot +0x14 (0 args) stubbed; the word at `0xe85ddc` points at 32 random bytes less `0` (placed); the word at `p8+0` points at 8 random bytes less `0` (placed); the word at `p8+4` points at 8 random bytes less `0` (placed); the word at `p8+8` points at 8 random bytes less `0` (placed); the word at `p8+12` points at 8 random bytes less `0` (placed); the word at `p8+16` points at 8 random bytes less `0` (placed); the word at `p8+20` points at 8 random bytes less `0` (placed); the word at `p8+24` points at 8 random bytes less `0` (placed); the word at `p8+28` points at 8 random bytes less `0` (placed); the word at `0xc0aaa0` points at 32 random bytes less `0` (placed); the word at `p17+0` points at 720 random bytes less `0` (placed); the word at `p17+4` points at 720 random bytes less `0` (placed); the word at `p17+8` points at 720 random bytes less `0` (placed); the word at `p17+12` points at 720 random bytes less `0` (placed); the word at `p17+16` points at 720 random bytes less `0` (placed); the word at `p17+20` points at 720 random bytes less `0` (placed); the word at `p17+24` points at 720 random bytes less `0` (placed); the word at `p17+28` points at 720 random bytes less `0` (placed); the word at `0xc09970` points at 12 random bytes less `0` (placed); the word at `p26+0` points at 32 random bytes less `0` (placed); the word at `p26+4` points at 32 random bytes less `0` (placed); the word at `p26+8` points at 32 random bytes less `0` (placed); the word at `0xc0998c` points at 12 random bytes less `0` (placed); the word at `p30+0` points at 32 random bytes less `0` (placed); the word at `p30+4` points at 32 random bytes less `0` (placed); the word at `p30+8` points at 32 random bytes less `0` (placed); the word at `0xc099a8` points at 12 random bytes less `0` (placed); the word at `p34+0` points at 32 random bytes less `0` (placed); the word at `p34+4` points at 32 random bytes less `0` (placed); the word at `p34+8` points at 32 random bytes less `0` (placed); the word at `0xc099c4` points at 12 random bytes less `0` (placed); the word at `p38+0` points at 32 random bytes less `0` (placed); the word at `p38+4` points at 32 random bytes less `0` (placed); the word at `p38+8` points at 32 random bytes less `0` (placed); the word at `c1+0x138` points at 512 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; stub 3 answers 3 values in call order, printed after the answers; stub 1 answers 0..30 every row; stub 2 answers 0..1 every row; stub 3 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 256 bytes under the call is zeroed before each row; 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x408:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_city_at.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let d3 = |i: i32| rd(&c[0], 4 * i as usize);
            let le = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
            let (x, y, who, ty, p5) = (a[0], a[1], a[2], a[3], a[4]);
            wr(m, 0x1fc, 99_999_999);
            let wu = who as usize;
            let (w, h) = (rd(&c[1], 0x18), rd(&c[1], 0x1c));
            let mut stub0 = "-".to_string();
            let (mut dx_s, mut dy_s) = ("-".to_string(), "-".to_string());
            let mut stub2: Vec<String> = Vec::new();
            let mut best = -1i64;
            'run: {
                if rd(&p[2 * wu], 0) & 1 == 0 {
                    break 'run;
                }
                if x < 0 || y < 0 || x >= w * 0xc0 || y >= h * 0xc0 {
                    break 'run;
                }
                if p5 == 0 {
                    stub0 = format!("{who},sp-0x24,sp-0x4,{who}");
                    dx_s = le(d3(x >> 6));
                    dy_s = le(d3(y >> 6));
                    if a[5] == 0 {
                        break 'run;
                    }
                } else {
                    let idx = (d3(y >> 6) * w + d3(x >> 6)) as usize;
                    let flags = u16::from_le_bytes([p[42][2 * idx], p[42][2 * idx + 1]]);
                    if flags & 0x100 == 0 {
                        break 'run;
                    }
                }
                let base = 27 + 4 * wu;
                let n = rd(&p[2 * wu + 1], 0);
                let (mut k_rad, mut k_cnt, mut k_slot) = (0, 0, 0);
                for b in 0..n.max(0) as usize {
                    let city = &p[base + b];
                    if rd(city, 4) & 1 == 0 {
                        continue;
                    }
                    let da = (d3(rd(city, 0x10) >> 6) - d3(y >> 6)).abs();
                    let db = (d3(rd(city, 0xc) >> 6) - d3(x >> 6)).abs();
                    let mut dist = crate::world::vector_dist(da, db);
                    let radius = a[9 + k_rad];
                    k_rad += 1;
                    if dist > radius {
                        continue;
                    }
                    if ty >= 0 {
                        let slot = a[15 + k_slot];
                        k_slot += 1;
                        if slot != 0 && rd(&p[18 + ty as usize], 0x2c0) & 0x200 != 0 {
                            stub2.push(format!("p{}+0x0,{ty},0,0", base + b));
                            let cnt = a[12 + k_cnt];
                            k_cnt += 1;
                            if cnt != 0 {
                                dist += 100;
                            }
                        }
                    }
                    if dist <= rd(m, 0x1fc) {
                        wr(m, 0x1fc, dist);
                        best = b as i64;
                    }
                }
            }
            log.push(stub0);
            log.push(if stub2.is_empty() { "-".into() } else { stub2.join("/") });
            log.push(dx_s);
            log.push(dy_s);
            best
        };
        assert_eq!(got, want_eax, "find_city_at — eax, row `{line}`");
        assert_eq!(log, want_log, "find_city_at — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_city_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_city_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "find_city_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("ObjectsData::find_city_at@0065b870: {n} rows agree");
}
