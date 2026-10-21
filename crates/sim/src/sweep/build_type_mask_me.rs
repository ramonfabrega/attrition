//! `BuildType::mask_me@006312a0` under the emulator against the port (item 3400; tools/sweep_gen.py, obj with windows).

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

/// `BuildType::mask_me@006312a0`, run by `tools/emu/sweep_build_type_mask_me.py` under unicorn
/// (tools/emu): 800 random bytes of object, 8 argument(s), `p` windows in order: p0..p8 the place(s), in flag order, p9 stub 7's answer window; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00cae5fc points at 256 random bytes; 4 random bytes at 0x00ee1098 (each word in 1..1); vtable slot +0x60 (2 args) stubbed; vtable slot +0x64 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x108 (0 args) stubbed; callee 0x006b43b0 (5 args) stubbed; callee 0x006b4900 (3 args) stubbed; callee 0x006b4610 (3 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00740c90 (2 args) stubbed; vtable slot +0xb0 (0 args) stubbed; callee 0x0063e310 (4 args) stubbed; callee 0x0063c580 (6 args) stubbed; the word at `c1+0x14+0x1c*0` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `c1+0x14+0x1c*1` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `m+0x30c` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `c0+0x134` points at 320 random bytes less `0` (placed); the word at `c0+0x138` points at 192 random bytes less `0` (placed); slot +0x64 laid as 0x00472bc0 or its stub, a flag after the answers; slot +0x90 laid as 0x00472bb0 or its stub, a flag after the answers; stub 7 answers the address of 128 random bytes, a `p` window after the places; the word at `m+0` laid as the image vtable at 0xb42b94 (the stub slots copied there) half the rows; stub 0 answers 64 values in call order, printed after the answers; stub 1 answers 40 values in call order, printed after the answers; stub 2 answers 40 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 0 answers 0..1 every row; stub 1 answers 0..1 every row; stub 2 answers 0..1 every row; stub 3 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 4 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 4 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 5 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 5 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_mask_me.py") else {
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
                // INLINE PORT, and a NAMESAKE situation: `Sim::mask_building` (city.rs) is the Sim-level fold of
                // `Wall::mask_me` -> `BuildType::mask_me` and shares no argument list with this function (no
                // Coord pair, no object indices, no RegenRoads), so it cannot be called here. This block is the
                // reading of `BuildType::mask_me@006312a0` itself: c0 the world (+0 cell width, +0x18/+0x1c tile
                // size, +0x134 cells at p7, +0x138 tiles at p8), c1 the objects cell (records p1/p2 and p4/p5),
                // c2 `div_3_table`; a[0..8] the arguments, a[8+k] stub k's answer, then the four `--seq`
                // streams, then the two slot-compare flags and the mask-null flag.
                const IMG: u32 = 0x00b4_2b94;
                struct St<'a> {
                    log: Vec<String>,
                    seq: [&'a [i32]; 4],
                    pos: [usize; 4],
                    keep64: bool,
                    keep90: bool,
                    img: bool,
                    flags: i32,
                    derefs: [Vec<String>; 6],
                }
                fn hex(v: i32) -> String {
                    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                }
                impl St<'_> {
                    fn next(&mut self, k: usize) -> i32 {
                        let s = self.seq[k];
                        let v = s[self.pos[k].min(s.len() - 1)];
                        self.pos[k] += 1;
                        v
                    }
                    // `Type::is(ty, 0)`, vtable slot 0x60.
                    fn is(&mut self, ty: i32) -> bool {
                        self.log.push(format!("s0:m+0x0,{ty},0"));
                        self.next(0) != 0
                    }
                    // The slot-0x64 call: inlined to `is(0x19e)` when the slot holds `BuildTypeData::is_city`.
                    fn is_city(&mut self) -> bool {
                        if self.keep64 {
                            self.is(0x19e)
                        } else {
                            self.log.push("s1:m+0x0".into());
                            self.next(1) != 0
                        }
                    }
                    // The slot-0x90 call: inlined to `build_flags & 0x40` when it holds `is_gather_type`.
                    fn is_gather(&mut self) -> bool {
                        if self.keep90 {
                            self.flags & 0x40 != 0
                        } else {
                            self.log.push("s2:m+0x0".into());
                            self.next(2) != 0
                        }
                    }
                    // `BuildTypeData::connects_to_roads@006398c0`, called for real by the original.
                    fn connects_to_roads(&mut self) -> bool {
                        if (self.is_gather() || self.flags & 0x10 != 0) && !self.is(0x1a4) {
                            return false;
                        }
                        true
                    }
                    fn world(&mut self, s: usize, x: i32, y: i32, tail: &str) {
                        self.log.push(format!("s{s}:c0+0x0,sp-0x44,sp-0x40{tail}"));
                        if s >= 4 {
                            self.derefs[2 * (s - 4)].push(hex(x));
                            self.derefs[2 * (s - 4) + 1].push(hex(y));
                        }
                    }
                }
                let (p1, p2, p3, p4, p6, p7, p8) = (a[0], a[1], a[2], a[3], a[5], a[6], a[7]);
                let vt = rd(m, 0) as u32;
                let mut st = St {
                    log: Vec::new(),
                    seq: [&a[20..84], &a[84..124], &a[124..164], &a[164..166]],
                    pos: [0; 4],
                    keep64: a[166] != 0 && vt != IMG,
                    keep90: a[167] != 0 && vt != IMG,
                    img: vt == IMG,
                    flags: rd(m, 0x2c0),
                    derefs: Default::default(),
                };
                let (xs, ys) = (rd(m, 0x234), rd(m, 0x238));
                let obj = format!("p{}+0x0", 1 + 3 * p7 + p6);
                let d3 = |c: &[Vec<u8>], i: i32| rd(&c[2], 4 * i as usize);
                if rd(m, 0x30c) != 0 {
                    let mut ax = d3(c, p1 >> 6) * 0xc0 + 0x60;
                    if xs & 1 == 0 {
                        ax -= 0x60;
                    }
                    let mut ay = d3(c, p2 >> 6) * 0xc0 + 0x60;
                    if ys & 1 == 0 {
                        ay -= 0x60;
                    }
                    let ox = d3(c, ax >> 6) - (xs >> 1);
                    let oy = d3(c, ay >> 6) - (ys >> 1);
                    let cell = (d3(c, p2 >> 8) * rd(&c[0], 0) + d3(c, p1 >> 8)) as usize;
                    let at = cell * 0x1c;
                    let mut f = u16::from_le_bytes([p[7][at], p[7][at + 1]]);
                    if p4 == 0 {
                        f &= 0xbfff;
                    } else {
                        f |= 0x4000;
                    }
                    p[7][at..at + 2].copy_from_slice(&f.to_le_bytes());
                    let (tw, th) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
                    for i in 0..xs.max(0) {
                        for j in 0..ys.max(0) {
                            let (x, y) = (i + ox, j + oy);
                            if x < 0 || y < 0 || x >= tw || y >= th {
                                continue;
                            }
                            let t = 2 * (tw * y + x) as usize;
                            let mut w = u16::from_le_bytes([p[8][t], p[8][t + 1]]);
                            w &= 0xffbf;
                            w &= 0xff7f;
                            if p4 == 0 {
                                if w & 3 == 3 {
                                    w &= 0xfffc;
                                }
                            } else {
                                w |= 3;
                                if !st.is_city()
                                    && (st.is_gather() || st.flags & 0x10 != 0)
                                    && !st.is(0x1a4)
                                {
                                    st.world(4, x, y, ",0,0,0");
                                }
                            }
                            p[8][t..t + 2].copy_from_slice(&w.to_le_bytes());
                            if p[6][(xs * j + i) as usize] == 1 {
                                st.world(5, x, y, &format!(",{p4}"));
                            } else if p4 != 0 {
                                st.world(5, x, y, ",0");
                                if st.is_city() || st.connects_to_roads() {
                                    st.world(4, x, y, ",1,0,0");
                                }
                            }
                        }
                    }
                    let dock = if st.img {
                        st.is(0x1b0)
                    } else {
                        st.log.push("s3:m+0x0".into());
                        st.next(3) != 0
                    };
                    if dock {
                        let mut i = -3;
                        while i < xs + 3 {
                            let mut j = -3;
                            while j < ys + 3 {
                                let (x, y) = (i + ox, j + oy);
                                if x >= 0 && y >= 0 && x < tw && y < th {
                                    let t = 2 * (tw * y + x) as usize;
                                    if p[8][t] & 0x30 == 0x20 {
                                        st.world(6, x, y, &format!(",{p4}"));
                                    }
                                }
                                j += 1;
                            }
                            i += 1;
                        }
                        if p4 == 0 {
                            st.log.push(format!("s7:{obj}"));
                            let short = i16::from_le_bytes([p[9][0x78], p[9][0x79]]);
                            st.log.push(format!("s8:{obj},{p7},{short}"));
                        }
                    }
                }
                // The tail's vtable test is against `Type_`'s vtable only; the else arm calls slot 0x64 for
                // real, which (kept) is `BuildTypeData::is_city` itself and so asks `is(0x19e)`.
                let city = if st.img { st.is(0x19e) } else { st.is_city() };
                if city {
                    st.log.push(format!("s9:{obj}"));
                    st.log.push(format!(
                        "s10:{},{},{},{},{}",
                        a[8 + 9],
                        d3(c, p1 >> 6),
                        d3(c, p2 >> 6),
                        p4,
                        p3
                    ));
                }
                let (flag, regen) = if p4 == 0 {
                    (0, 1)
                } else if p8 == 0 {
                    (-1, 0)
                } else {
                    (1, p8)
                };
                if flag >= 0 {
                    st.log
                        .push(format!("s11:m+0x0,{p1},{p2},{p6},{p7},{flag},{regen}"));
                }
                log.extend(std::mem::take(&mut st.log));
                for d in &st.derefs {
                    log.push(if d.is_empty() {
                        "-".into()
                    } else {
                        d.join("/")
                    });
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "mask_me — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "mask_me — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "mask_me — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "mask_me — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2400, "only {n} rows");
    eprintln!("BuildType::mask_me@006312a0: {n} rows agree");
}
