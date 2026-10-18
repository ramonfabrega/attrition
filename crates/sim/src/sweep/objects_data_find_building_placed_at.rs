//! `ObjectsData::find_building_placed_at@00658c80` under the emulator against the port (item 3229; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_building_placed_at@00658c80`, run by `tools/emu/sweep_objects_data_find_building_placed_at.py` under unicorn
/// (tools/emu): 528 random bytes of object, 5 argument(s), `p` windows in order: p0..p38 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 36 random bytes at 0x00adc400 (each word in -1..1); 36 random bytes at 0x00adcaf0 (each word in -1..1); vtable slot +0xc (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; the word at `0xc0ab84+0` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+28` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 48 random bytes less `0` (placed); the word at `p4+4` points at 48 random bytes less `0` (placed); the word at `p4+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+56` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 48 random bytes less `0` (placed); the word at `p8+4` points at 48 random bytes less `0` (placed); the word at `p8+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+84` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 48 random bytes less `0` (placed); the word at `p12+4` points at 48 random bytes less `0` (placed); the word at `p12+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+112` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 48 random bytes less `0` (placed); the word at `p16+4` points at 48 random bytes less `0` (placed); the word at `p16+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+140` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 48 random bytes less `0` (placed); the word at `p20+4` points at 48 random bytes less `0` (placed); the word at `p20+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+168` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 48 random bytes less `0` (placed); the word at `p24+4` points at 48 random bytes less `0` (placed); the word at `p24+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+196` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 48 random bytes less `0` (placed); the word at `p28+4` points at 48 random bytes less `0` (placed); the word at `p28+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+224` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 48 random bytes less `0` (placed); the word at `p32+4` points at 48 random bytes less `0` (placed); the word at `p32+8` points at 48 random bytes less `0` (placed); the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 1280 random bytes less `0` (placed); stub 1's answer points at 576 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the word at `p38+0x18` laid as the address of `p38` every row; stub 0 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_building_placed_at.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `this+0x200` (find_who) is cleared; the flag short of the cell (stride `c0+0x18`) must read
                // `& 3 == 3` or have bit 7, then the nine neighbours (`move_x`/`move_y`, c1 / c2) of the tile
                // (`param >> 2`) in the world (`c0+0` x `c0+4`) are walked: a tile record (stride 0x1c, array
                // `p36`) holds the head (short +8 index, +10 type), an object (`p[4 type + 1 + index]`, the
                // objects table behind `0xc0ab84 + 0x1c type`) the next in shorts +0x2c / +0x2e. An object of
                // type < 8, not the (`a[3]`, `a[4]`) one, of type `a[2]` unless that is negative, whose slot
                // +0xc (stub 0, `a[7..]` in call order) answers, is asked its slot +0x40 (the wall, `p38`) and
                // `tile_corner`: a hit is the point inside the wall's box.
                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                wr(m, 0x200, -1);
                let (p1, p2, p3, p4, p5) = (a[0], a[1], a[2], a[3], a[4]);
                let (w, h, stride) = (rd(&c[0], 0), rd(&c[0], 4), rd(&c[0], 0x18));
                let fo = usize::try_from((stride * p2 + p1) * 2).expect("flag offset");
                let fl = u16::from_le_bytes([p[37][fo], p[37][fo + 1]]);
                let mut res: i64 = -1;
                let mut calls = 0usize;
                let (mut l0, mut l1): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
                if (fl & 3) == 3 || (fl >> 7) & 1 != 0 {
                    'tiles: for i in 0..9 {
                        let x = rd(&c[3], 4 * i) + (p1 >> 2);
                        let y = rd(&c[2], 4 * i) + (p2 >> 2);
                        if x < 0 || y < 0 || x >= w || y >= h {
                            continue;
                        }
                        let to = usize::try_from(w * y + x).expect("tile") * 0x1c;
                        let mut cur = sh(&p[36], to + 8);
                        let mut ty = sh(&p[36], to + 10);
                        while cur >= 0 {
                            let o = &p[usize::try_from(4 * ty + 1 + cur).expect("object")];
                            let (nxt, nty) = (sh(o, 0x2c), sh(o, 0x2e));
                            if ty < 8 && (cur != p4 || ty != p5) && (p3 < 0 || p3 == ty) {
                                let ans = a[7 + calls.min(5)];
                                calls += 1;
                                l0.push(format!("p{}+0x0", 4 * ty + 1 + cur));
                                if ans != 0 {
                                    l1.push(format!("p{}+0x0", 4 * ty + 1 + cur));
                                    let wall = &p[38];
                                    let (xs, ys) = (rd(wall, 0x234), rd(wall, 0x238));
                                    // `WallData::tile_corner`: the table behind the cell 0xcae5fc (`c1`).
                                    let dv = |k: i32| {
                                        rd(&c[1], 4 * usize::try_from(k).expect("table index"))
                                    };
                                    let corner = |v: i32, size: i32| {
                                        let q = dv((v ^ 0x63637) >> 6) * 0xc0
                                            + if size & 1 != 0 { 0x60 } else { 0 };
                                        dv(q >> 6) - (size >> 1)
                                    };
                                    let c0 =
                                        (corner(rd(wall, 0x10), xs), corner(rd(wall, 0x14), ys));
                                    if c0.0 <= p1 && p1 < xs + c0.0 && c0.1 <= p2 && p2 < ys + c0.1
                                    {
                                        wr(m, 0x200, ty);
                                        res = i64::from(cur);
                                        break 'tiles;
                                    }
                                }
                            }
                            cur = nxt;
                            ty = nty;
                        }
                    }
                }
                log.push(if l0.is_empty() {
                    "-".to_string()
                } else {
                    l0.join("/")
                });
                log.push(if l1.is_empty() {
                    "-".to_string()
                } else {
                    l1.join("/")
                });
                res
            }
        };
        assert_eq!(got, want_eax, "find_building_placed_at — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_building_placed_at — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_building_placed_at — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_building_placed_at — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "find_building_placed_at — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("ObjectsData::find_building_placed_at@00658c80: {n} rows agree");
}
