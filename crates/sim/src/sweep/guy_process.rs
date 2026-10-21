//! `Guy::process@005e0230` under the emulator against the port (item 3396; tools/sweep_gen.py, obj with windows).

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

/// `Guy::process@005e0230`, run by `tools/emu/sweep_guy_process.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p3 the place(s), in flag order, p4..p9 the stubs' answer window(s); the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 832 random bytes; 128 random bytes at 0x00c0aec0; 1536 random bytes at 0x00d10000; callee 0x005d9240 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046d250 (2 args) stubbed; the word at `c1+0x134` points at 448 random bytes less `0` (placed); the word at `c3+i8(m,0xa1)*0x1c` points at 4 random bytes less `i16(m,0x8c)*4` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 784 random bytes less `0` (placed); stub 1 answered from [-1, 0, 1, 1, 2, 3, 32767, 32768, 65535] (90%); stub 2 answers a window of its own to each call, in call order (112 bytes, ...); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 5000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_process.py") else {
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
            // hand: port
            // `Guy::process` is no one crate method: its pivot turn is `anim::Turret::step`, its
            // sixty-fourth-frame test `collide::repaint_due` (also the flag clear's), its disc
            // `collide::paint_walk` over a `PaintView` (`Sim::coll_paint`'s too), and the three
            // gates between them (`+0x218` not air, `guy_num < squad_size`, `coll_size != 0`) have
            // no counterpart: `Sim::coll_repaint` reads them off a `Sim` (`coll_paint`'s size and air;
            // `squad_size` is its SEAM). The gates are therefore inline here, and so is the block
            // structure the crate flattens: a block's `+8` word zeroed on every mark, a null block
            // allocated by `World::new_coll_block` (stubbed, one answer window a call, the last
            // repeated), the bit `(x & 15) * 16 + (y & 15)` at `+0xc`. `Guy::move` is stubbed.
            // A cell whose block word is `-1` is not drawn: the crate has no such cell (parked).
            // Windows: m the guy; c0 the game (`+0x550` the frame), c1 the world (`+0`/`+4` its size),
            // c2 `div_3_table`, c3 the units table, c4 sixteen blocks of 0x60 at `BLOCKS`; p0 the
            // world's cell table (`0x1c` apiece: `+4` region short, `+0x18` block), p3 the type record,
            // p4.. the windows `new_coll_block` answers.
            use sim::anim::Turret;
            use sim::collide::{CollGrid, PaintView, paint_walk, repaint_due};
            use sim::world::Pos;
            const BLOCKS: i32 = 0xd1_0000;
            let u16at = |w: &[u8], o: usize| u16::from_le_bytes([w[o], w[o + 1]]);
            let le =
                |v: i32| -> String { v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect() };
            let mut flags = u16at(m, 0x9a) & !2;
            if flags & 0x100 != 0 {
                let mut t = Turret::ZERO;
                for k in 0..4 {
                    t.angles[k] = rd(m, 0x20 + 4 * k);
                    t.des[k] = rd(m, 0x30 + 4 * k);
                }
                t.node_flags = u16at(m, 0x96);
                t.step();
                for k in 0..4 {
                    wr(m, 0x20 + 4 * k, t.angles[k]);
                }
                m[0x96..0x98].copy_from_slice(&t.node_flags.to_le_bytes());
            }
            let (mut rec_t, mut rec_b) = (Vec::new(), Vec::new());
            let (mut dtx, mut dty, mut dbx, mut dby) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let frame = rd(&c[0], 0x550);
            let idx = i16::from_le_bytes([m[0x8c], m[0x8d]]);
            if repaint_due(i64::from(frame), idx, rd(m, 0x84)) {
                let ty = &p[3];
                let (domain, squad, size) = (rd(ty, 0x218), rd(ty, 0x304), rd(ty, 0x248));
                if domain != 2 && i32::from(m[0xa2] as i8) < squad && size != 0 {
                    let (x, y) = (rd(m, 0xc), rd(m, 0x10));
                    let (w, h) = (rd(&c[1], 0), rd(&c[1], 4));
                    let ans = a[1];
                    let tile = Pos::new(x, y).tile();
                    rec_t.push("c1+0x0,sp-0x10,sp-0x14".to_string());
                    dtx.push(le(tile.x));
                    dty.push(le(tile.y));
                    // The answer is an int and a cell's region a short: an answer past `0x7fff` equals
                    // none, which the view says by giving every cell a region it is not.
                    struct View<'a> {
                        ans: i32,
                        w: i32,
                        h: i32,
                        cells: &'a [u8],
                        grid: CollGrid,
                        visited: Vec<Pos>,
                    }
                    impl PaintView for View<'_> {
                        fn region(&mut self) -> Option<u16> {
                            (self.ans >= 0).then_some(if self.ans < 0x8000 {
                                self.ans as u16
                            } else {
                                0
                            })
                        }
                        fn cell_region(&mut self, cell: Pos) -> Option<u16> {
                            if cell.x < 0 || cell.y < 0 || cell.x >= self.w || cell.y >= self.h {
                                return None;
                            }
                            let o = ((cell.y * self.w + cell.x) * 0x1c + 4) as usize;
                            let s = i16::from_le_bytes([self.cells[o], self.cells[o + 1]]);
                            Some(if self.ans >= 0x8000 {
                                1
                            } else if s >= 0 {
                                s as u16
                            } else {
                                0x8000
                            })
                        }
                        fn mark(&mut self, p: Pos, on: bool) {
                            assert!(on);
                            self.grid.set(p.x, p.y, true);
                            if self.grid.get(p.x, p.y) {
                                self.visited.push(p);
                            }
                        }
                    }
                    let mut v = View {
                        ans,
                        w,
                        h,
                        cells: &p[0],
                        grid: CollGrid::new(w, h),
                        visited: Vec::new(),
                    };
                    paint_walk(&mut v, size, sim::collide::ucell(Pos::new(x, y)), true);
                    let visited = std::mem::take(&mut v.visited);
                    let mut allocs = 0usize;
                    for q in visited {
                        let (cx, cy) = (q.x >> 4, q.y >> 4);
                        let k = (cy * w + cx) as usize;
                        let blk = rd(&p[0], k * 0x1c + 0x18);
                        let bit = ((q.x & 15) * 16 + (q.y & 15)) as usize;
                        assert!(
                            blk == 0 || (BLOCKS..BLOCKS + 0x600).contains(&blk),
                            "block {blk:#x}"
                        );
                        if blk == 0 {
                            let ecx = if ans < 0 {
                                "p0+0x0".to_string()
                            } else {
                                i32::from(i16::from_le_bytes([
                                    p[0][k * 0x1c + 4],
                                    p[0][k * 0x1c + 5],
                                ]))
                                .to_string()
                            };
                            rec_b.push(format!("{ecx},sp-0x24,sp-0x20"));
                            dbx.push(le(cx));
                            dby.push(le(cy));
                            let win = &mut p[4 + allocs.min(5)];
                            allocs += 1;
                            win[0xc + (bit >> 3)] |= 1 << (bit & 7);
                            wr(win, 8, 0);
                        } else {
                            let o = (blk - BLOCKS) as usize;
                            let b = &mut c[4][o..o + 0x60];
                            b[0xc + (bit >> 3)] |= 1 << (bit & 7);
                            wr(b, 8, 0);
                        }
                    }
                }
                flags &= !0x20;
            }
            m[0x9a..0x9c].copy_from_slice(&flags.to_le_bytes());
            let join = |v: &[String]| {
                if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                }
            };
            for v in [&rec_t, &rec_b, &dtx, &dty, &dbx, &dby] {
                log.push(join(v));
            }
            0i64
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 5000, "only {n} rows");
    eprintln!("Guy::process@005e0230: {n} rows agree");
}
