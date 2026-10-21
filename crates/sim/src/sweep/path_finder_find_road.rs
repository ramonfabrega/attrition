//! `PathFinder::find_road@00688a40` under the emulator against the port (item 3397; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::find_road@00688a40`, run by `tools/emu/sweep_path_finder_find_road.py` under unicorn
/// (tools/emu): 64 random bytes of object, 7 argument(s), `p` windows in order: p0 argument 0, p1..p19 the place(s), in flag order; the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; 24 random bytes at 0x00e85ec4 (each word in 0..1); argument 0 names 16 random bytes; vtable slot +0xac (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x64 (0 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x00685990 (7 args) stubbed; callee 0x00687ae0 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `p1+12` points at 64 random bytes less `0` (placed); the word at `m+0x20` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `p6+8` points at 64 random bytes less `0` (placed); the word at `p6+12` points at 64 random bytes less `0` (placed); the word at `m+0x30` points at 16 random bytes less `0` (placed); the word at `p11+0` points at 576 random bytes less `0` (placed); the word at `p11+4` points at 576 random bytes less `0` (placed); the word at `p11+8` points at 576 random bytes less `0` (placed); the word at `p11+12` points at 576 random bytes less `0` (placed); the word at `0xc0aec0` points at 4 random bytes less `0` (placed); the word at `p16+0` points at 256 random bytes less `0` (placed); the word at `c0+0x134` points at 4096 random bytes less `0` (placed); the word at `p17+0x18` points at 704 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 4 answered from [-1, 0, 2, 5] (90%); the word at `p7+0x18` laid as the address of `p12` every row; the word at `p8+0x18` laid as the address of `p13` every row; the word at `p9+0x18` laid as the address of `p14` every row; the word at `p10+0x18` laid as the address of `p15` every row; stub 0 answers `p7+j`, the data of the object its `ecx` is in `p2+j` (j < 4), linked at `+0x3c`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_find_road.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            'g: {
                // `find_road` with `astar_caravan_road` and `kill_lists` stubbed. The crate's `Sim::find_road` is this
                // function's `param_6 < 0` arm end to end (`start_road` is the front matter below, `step_road` the
                // callee); the sweep holds the front matter and the three flags the callee reads, which is all the
                // function itself computes. Windows: c0 the world, c1 `div_3_table`, c2 the
                // pathfinder's `+0x84..+0x9c`; p0 the stack, p2..p5 the objects, p7..p10 their `+0xac` data, p12..p15
                // their types, p17 the unit, p18 the regions, p19 the unit's type. a[7..=12] the stubs' answers, a[13] the
                // `+0x64` slot flag (1 = `is_city` itself, so `+0x60` is entered directly).
                const K: i32 = 0x63637;
                let mut toks: Vec<String> = vec!["-".to_string(); 6];
                fn note(toks: &mut [String], i: usize, s: String) {
                    if toks[i] == "-" {
                        toks[i] = s;
                    } else {
                        toks[i].push('/');
                        toks[i].push_str(&s);
                    }
                }
                let table = |v: i32| rd(&c[1], 4 * usize::try_from(v).expect("table index"));
                let pos = |w: &[u8], o: usize| (rd(w, o) ^ K) >> 6;
                let end = |toks: &mut [String], k: usize| -> (i32, i32) {
                    // The object's `+0xac`, then `BuildTypeData::is_city` on its type: entered through slot +0x64, or
                    // inlined as slot +0x60(0x19e, 0).
                    note(toks, 0, format!("p{}+0x0", 2 + k));
                    let city = if a[13] != 0 {
                        note(toks, 1, format!("p{}+0x0,414,0", 12 + k));
                        a[8] != 0
                    } else {
                        note(toks, 2, format!("p{}+0x0", 12 + k));
                        a[9] != 0
                    };
                    if city {
                        let o = &p[2 + k];
                        return (table(pos(o, 0x10)), table(pos(o, 0x14)));
                    }
                    for _ in 0..3 {
                        note(toks, 0, format!("p{}+0x0", 2 + k));
                    }
                    let (x, ty) = (&p[7 + k], &p[12 + k]);
                    let (sx, sy) = (rd(ty, 0x234), rd(ty, 0x238));
                    // `WallData::tile_corner`: the half-tile shift is added when the footprint is odd.
                    let corner = |raw: i32, s: i32| {
                        let t = table((raw ^ K) >> 6).wrapping_mul(0xc0);
                        let t = if s & 1 == 0 { t } else { t.wrapping_add(0x60) };
                        table(t >> 6) - (s >> 1)
                    };
                    let (cx, cy) = (corner(rd(x, 0x10), sx), corner(rd(x, 0x14), sy));
                    (cx - 1 + sx, sy + cy - 1)
                };
                let (fx, fy) = end(&mut toks, a[1] as usize);
                let (tx, ty) = end(&mut toks, a[3] as usize);
                let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
                let mut deref: Vec<String> = Vec::new();
                // `pathfinder+0x84` set: both endpoints taken as they stand, none pushed.
                if rd(&c[2], 0) == 0 {
                    let inside = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h;
                    if !(inside(tx, ty) && inside(fx, fy) && (fx != tx || fy != ty)) {
                        wr(&mut p[0], 8, 0);
                        log.extend(toks);
                        log.push("-".to_string());
                        break 'g -1;
                    }
                    let pd = |x: i32, y: i32| {
                        [x * 0xc0 + 0x60, y * 0xc0 + 0x60, 0x60, 0]
                            .iter()
                            .flat_map(|v| v.to_le_bytes())
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    };
                    deref.push(pd(tx, ty));
                    deref.push(pd(fx, fy));
                    note(&mut toks, 3, "p0+0x0,sp-0x48".to_string());
                    note(&mut toks, 3, "p0+0x0,sp-0x58".to_string());
                }
                // `pathfinder+0x88 = 0`, `+0x8c` the regions' shorts equal, `+0x98` the unit's flag.
                wr(&mut c[2], 4, 0);
                let region = |x: i32, y: i32| {
                    let i = ((y >> 2) * rd(&c[0], 0) + (x >> 2)) as isize * 0x1c + 4;
                    let i = usize::try_from(i).expect("region offset");
                    i16::from_le_bytes([p[18][i], p[18][i + 1]])
                };
                let same = i32::from(region(fx, fy) == region(tx, ty));
                wr(&mut c[2], 8, same);
                let unit = &p[17];
                let short = if a[5] < 0 {
                    wr(&mut c[2], 0x14, 0);
                    -1
                } else {
                    let walking = (rd(unit, 0x68) as u32 & 0x80_0000 == 0)
                        || (rd(unit, 0x6c) as u32 & 0x2000 != 0);
                    let flag = !(walking && p[19][0x2b4] & 0x10 == 0);
                    wr(&mut c[2], 0x14, i32::from(flag));
                    i32::from(i16::from_le_bytes([unit[0x86], unit[0x87]]))
                };
                toks[4] = format!(
                    "{},p0+0x0,{},{},{},{},{},{}",
                    a[6], a[1], a[2], a[3], a[4], short, a[6]
                );
                toks[5] = "15228480".to_string();
                log.extend(toks);
                log.push(if deref.is_empty() {
                    "-".to_string()
                } else {
                    deref.join("/")
                });
                if a[11] < 0 {
                    -1
                } else {
                    i64::from(rd(&p[0], 8))
                }
            }
            // end hand
        };
        assert_eq!(got, want_eax, "find_road — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_road — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_road — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_road — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "find_road — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("PathFinder::find_road@00688a40: {n} rows agree");
}
