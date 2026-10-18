//! `ObjectsData::find_any_building_at@00659180` under the emulator against the port (item 3239; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_any_building_at@00659180`, run by `tools/emu/sweep_objects_data_find_any_building_at.py` under unicorn
/// (tools/emu): 528 random bytes of object, 7 argument(s), `p` windows in order: p0..p42 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; the global cell 0x00cae5fc points at 2048 random bytes; 36 random bytes at 0x00adc400 (each word in -1..1); 36 random bytes at 0x00adcaf0 (each word in -1..1); vtable slot +0xc (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; the word at `0xc0ab84+0` points at 12 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+28` points at 12 random bytes less `0` (placed); the word at `p4+0` points at 48 random bytes less `0` (placed); the word at `p4+4` points at 48 random bytes less `0` (placed); the word at `p4+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+56` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 48 random bytes less `0` (placed); the word at `p8+4` points at 48 random bytes less `0` (placed); the word at `p8+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+84` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 48 random bytes less `0` (placed); the word at `p12+4` points at 48 random bytes less `0` (placed); the word at `p12+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+112` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 48 random bytes less `0` (placed); the word at `p16+4` points at 48 random bytes less `0` (placed); the word at `p16+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+140` points at 12 random bytes less `0` (placed); the word at `p20+0` points at 48 random bytes less `0` (placed); the word at `p20+4` points at 48 random bytes less `0` (placed); the word at `p20+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+168` points at 12 random bytes less `0` (placed); the word at `p24+0` points at 48 random bytes less `0` (placed); the word at `p24+4` points at 48 random bytes less `0` (placed); the word at `p24+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+196` points at 12 random bytes less `0` (placed); the word at `p28+0` points at 48 random bytes less `0` (placed); the word at `p28+4` points at 48 random bytes less `0` (placed); the word at `p28+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+224` points at 12 random bytes less `0` (placed); the word at `p32+0` points at 48 random bytes less `0` (placed); the word at `p32+4` points at 48 random bytes less `0` (placed); the word at `p32+8` points at 48 random bytes less `0` (placed); the word at `0xc0ab84-28` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 48 random bytes less `0` (placed); the word at `p36+4` points at 48 random bytes less `0` (placed); the word at `p36+8` points at 48 random bytes less `0` (placed); the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 1280 random bytes less `0` (placed); stub 2's answer points at 576 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [0, 1] (90%); the word at `p42+0x18` laid as the address of `p42` every row; stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 4 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_any_building_at.py") else {
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
                // The crate's `find_any_building_at` (orders.rs) reads the sim's building list, not a byte window,
                // so this is a byte-window port of the original (the sibling sweep's way). `this+0x200` (find_who)
                // is cleared; the flag short of the cell (stride `c0+0x18`) must read `& 3 == 3` or have bit 7,
                // then the nine neighbours (`move_x` is the table at 0xadcaf0 = c3, `move_y` 0xadc400 = c2) of the
                // tile (`param >> 2`) in the world (`c0+0` x `c0+4`) are walked: a tile record (stride 0x1c, array
                // `p40`) holds the head (short +8 index, +10 type), an object (`p[4 type + 1 + index]`) the next in
                // shorts +0x2c / +0x2e. An object of type < 8 passes `Search::valid_search(type, 0, who)` (stub 3,
                // a type < 0 skips it), slot +0xc (stub 0), and slot +0x50 (stub 1) or `who == type`; its wall
                // (slot +0x40, `p42`) box is then tested against the tile (as `tile_corner`), and a filter `a[4]`
                // in 1..=23 asks `Search::valid_filter(type, index, a[5], 0, filter - 1)` (stub 4).
                let sh = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
                wr(m, 0x200, -1);
                let (p1, p2, who, filt, a5) = (a[0], a[1], a[3], a[4], a[5]);
                let (w, h, stride) = (rd(&c[0], 0), rd(&c[0], 4), rd(&c[0], 0x18));
                let fo = usize::try_from((stride * p2 + p1) * 2).expect("flag offset");
                let fl = u16::from_le_bytes([p[41][fo], p[41][fo + 1]]);
                let mut res: i64 = -1;
                let mut n = [0usize; 5];
                let mut l: [Vec<String>; 5] = Default::default();
                let mut ask = |k: usize, base: usize| {
                    let v = a[base + n[k].min(5)];
                    n[k] += 1;
                    v
                };
                if (fl & 3) == 3 || (fl >> 7) & 1 != 0 {
                    'tiles: for i in 0..9 {
                        let x = rd(&c[3], 4 * i) + (p1 >> 2);
                        let y = rd(&c[2], 4 * i) + (p2 >> 2);
                        if x < 0 || y < 0 || x >= w || y >= h {
                            continue;
                        }
                        let to = usize::try_from(w * y + x).expect("tile") * 0x1c;
                        let mut cur = sh(&p[40], to + 8);
                        let mut ty = sh(&p[40], to + 10);
                        while cur >= 0 {
                            // type -1 reads the table before type 0's (0xc0ab84 - 0x1c): the last `--ptrtab`, `p37..`.
                            let oi =
                                usize::try_from(if ty < 0 { 37 + cur } else { 4 * ty + 1 + cur })
                                    .expect("object");
                            let (nxt, nty) = (sh(&p[oi], 0x2c), sh(&p[oi], 0x2e));
                            let name = format!("p{oi}+0x0");
                            let mut ok = ty < 8;
                            if ok && ty >= 0 {
                                l[3].push(format!("{ty},0,{who}"));
                                ok = ask(3, 24) != 0;
                            }
                            if ok {
                                l[0].push(name.clone());
                                ok = ask(0, 12) != 0;
                            }
                            if ok {
                                l[1].push(name.clone());
                                ok = ask(1, 18) != 0 || who == ty;
                            }
                            if ok {
                                l[2].push(name.clone());
                                let wall = &p[42];
                                let (xs, ys) = (rd(wall, 0x234), rd(wall, 0x238));
                                // `WallData::tile_corner`'s arithmetic inline: the table behind the cell 0xcae5fc (`c1`).
                                let dv = |k: i32| {
                                    rd(&c[1], 4 * usize::try_from(k).expect("table index"))
                                };
                                let corner = |v: i32, size: i32| {
                                    let q = dv((v ^ 0x63637) >> 6) * 0xc0
                                        + if size & 1 != 0 { 0x60 } else { 0 };
                                    dv(q >> 6) - (size >> 1)
                                };
                                let c0 = (corner(rd(wall, 0x10), xs), corner(rd(wall, 0x14), ys));
                                ok = c0.0 <= p1 && p1 < xs + c0.0 && c0.1 <= p2 && p2 < ys + c0.1;
                            }
                            if ok && filt != 0 && filt.wrapping_sub(1) as u32 <= 0x16 {
                                l[4].push(format!("{ty},{cur},{a5},0,{}", filt.wrapping_sub(1)));
                                ok = ask(4, 30) != 0;
                            }
                            if ok {
                                wr(m, 0x200, ty);
                                res = i64::from(cur);
                                break 'tiles;
                            }
                            cur = nxt;
                            ty = nty;
                        }
                    }
                }
                for v in l {
                    log.push(if v.is_empty() {
                        "-".to_string()
                    } else {
                        v.join("/")
                    });
                }
                res
            }
        };
        assert_eq!(got, want_eax, "find_any_building_at — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "find_any_building_at — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "find_any_building_at — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_any_building_at — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "find_any_building_at — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("ObjectsData::find_any_building_at@00659180: {n} rows agree");
}
