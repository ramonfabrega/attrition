//! `Unit::find_goody_box@005f2540` under the emulator against the port (item 3239; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_goody_box@005f2540`, run by `tools/emu/sweep_unit_find_goody_box.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 196 random bytes at 0x00adc400 (each word in -3..3); 196 random bytes at 0x00adcaf0 (each word in -3..3); callee 0x006b53f0 (3 args) stubbed; callee 0x0065c040 (1 args) stubbed; callee 0x005f7690 (2 args) stubbed; vtable slot +0x48 (2 args) stubbed; the word at `0xc0a110` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 2048 random bytes less `0` (placed); the word at `c1+0x134` points at 2048 random bytes less `0` (placed); the word at `m+0x18` points at 560 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 1 answers -1..3 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_goody_box.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `Unit::find_goody_box`: a unit whose type (`this+0x18`, `p7`) has `+0x218` clear sweeps the 49
                // cells (`move_x` at 0xadcaf0 = c4, `move_y` at 0xadc400 = c3) round the cell its position
                // (`this+0x10/+0x14`, through `div_3_table`, c2) names. A cell must be in the world (c0, cells
                // `p5`), in the unit's own region (+4), carry GOODY (the sign of the flag short) and be seen
                // at any of its four half-cells (stub 0, `was_seen`, called in the listing's order); then
                // the second world (`worldc`, c1, cells `p6`) answers at once for a flag-0x100-clear cell of
                // type 1 or 2, else `find_goody_at` (stub 1) names an item whose slot 0x48 (stub 3) must
                // answer. The cell the unit already walks to ends the sweep; any other `get_goody_box`es.
                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                let hexw = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
                let col = |v: &Vec<String>| if v.is_empty() { "-".to_string() } else { v.join("/") };
                let (mut l_gb, mut l_it, mut l_wx, mut l_wy, mut l_fy): (Vec<String>, Vec<String>, Vec<String>, Vec<String>, Vec<String>) =
                    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                let mut res: i64 = 0;
                if rd(&p[7], 0x218) == 0 {
                    let div = |v: i32| rd(&c[2], 4 * usize::try_from(v).expect("table index"));
                    let bx = div((rd(m, 0x10) ^ 0x63637) >> 8);
                    let by = div((rd(m, 0x14) ^ 0x63637) >> 8);
                    let (w0, h0, w1) = (rd(&c[0], 0), rd(&c[0], 4), rd(&c[1], 0));
                    let cell = |x: i32, y: i32| usize::try_from(w0 * y + x).expect("cell") * 0x1c;
                    let region = sh(&p[5], cell(bx, by) + 4) as u16;
                    let who = i32::from(m[9]);
                    let (mut k0, mut k1, mut k3) = (0usize, 0usize, 0usize);
                    for i in 0..49 {
                        let x = rd(&c[4], 4 * i) + bx;
                        let y = rd(&c[3], 4 * i) + by;
                        if x < 0 || y < 0 || x >= w0 || y >= h0 {
                            continue;
                        }
                        let o = cell(x, y);
                        if sh(&p[5], o + 4) as u16 != region || sh(&p[5], o) >= 0 {
                            continue;
                        }
                        let mut seen = false;
                        for (px, py) in [(2 * x + 1, 2 * y + 1), (2 * x, 2 * y + 1), (2 * x + 1, 2 * y), (2 * x, 2 * y)] {
                            l_wx.push(hexw(px));
                            l_wy.push(hexw(py));
                            let ans = a[4 + k0.min(7)];
                            k0 += 1;
                            if ans != 0 {
                                seen = true;
                                break;
                            }
                        }
                        if !seen {
                            continue;
                        }
                        let o2 = usize::try_from(w1 * y + x).expect("worldc cell") * 0x1c;
                        let fl = u16::from_le_bytes([p[6][o2], p[6][o2 + 1]]);
                        let mut accept = false;
                        if fl & 0x100 == 0 {
                            let t = p[6][o2 + 2] as i8;
                            accept = t == 1 || t == 2;
                        }
                        if !accept {
                            l_fy.push(hexw(y));
                            let found = a[12 + k1.min(5)];
                            k1 += 1;
                            if found < 0 {
                                accept = true;
                            } else {
                                let it = 1 + usize::try_from(found).expect("item");
                                l_it.push(format!("p{it}+0x0,{who},0"));
                                let ans = a[18 + k3.min(5)];
                                k3 += 1;
                                accept = ans != 0;
                            }
                        }
                        if !accept {
                            continue;
                        }
                        let (ox, oy) = (div(rd(m, 0x70) >> 8), div(rd(m, 0x74) >> 8));
                        if !(x == ox && y == oy) {
                            l_gb.push(format!("m+0x0,{x},{y}"));
                            res = 1;
                        }
                        break;
                    }
                }
                log.push(col(&l_gb));
                log.push(col(&l_it));
                log.push(col(&l_wx));
                log.push(col(&l_wy));
                log.push(col(&l_fy));
                res
            }
        };
        assert_eq!(got, want_eax, "find_goody_box — eax, row `{line}`");
        assert_eq!(log, want_log, "find_goody_box — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_goody_box — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_goody_box — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "find_goody_box — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Unit::find_goody_box@005f2540: {n} rows agree");
}
