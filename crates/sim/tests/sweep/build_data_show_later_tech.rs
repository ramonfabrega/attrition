//! `BuildData::show_later_tech@0062d920` under the emulator against the port (item 3078; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `BuildData::show_later_tech@0062d920`, run by `tools/emu/sweep_obj.py` on its row `build_data_show_later_tech` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c061e8 points at 64 random bytes; the global cell 0x00c061e0 points at 16 random bytes; vtable slot +0x4 (0 args) stubbed; callee 0x00668700 (2 args) stubbed; callee 0x006e0c80 (1 args) stubbed; callee 0x00639d10 (2 args) stubbed; the word at `0xc0aabc` points at 48 random bytes less `-4` (placed); the word at `p0+0` points at 464 random bytes less `0` (placed); the word at `p0+4` points at 464 random bytes less `0` (placed); the word at `p0+8` points at 464 random bytes less `0` (placed); the word at `p0+12` points at 464 random bytes less `0` (placed); the word at `p0+16` points at 464 random bytes less `0` (placed); the word at `p0+20` points at 464 random bytes less `0` (placed); the word at `p0+24` points at 464 random bytes less `0` (placed); the word at `p0+28` points at 464 random bytes less `0` (placed); the word at `p0+32` points at 464 random bytes less `0` (placed); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 0, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 1 answers -3..7 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("build_data_show_later_tech") else {
        return;
    };
    let obj = super::Obj {
        name: "show_later_tech",
        full: "BuildData::show_later_tech@0062d920",
        cells: 2,
        calls: false,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            // `a`: 0 the tech, 1 the depth, 2..=5 the single answers (unused under `--seq`), then four runs of
            // four: the answers of slot +4 (the prerequisite count), `get_preq`, `has_tech` and `queue_here`,
            // each stub answering its run in call order and the last past the end. The table is `p[2..=9]`
            // (`p[1]` the record before it), `c[0]` the game (its byte 0x36 the ending technology).
            struct Go<'a> {
                a: &'a [i32],
                p: &'a [Vec<u8>],
                ending: i32,
                turn: [usize; 4],
            }
            impl Go<'_> {
                fn ask(&mut self, k: usize) -> i32 {
                    let v = self.a[6 + 4 * k + self.turn[k].min(3)];
                    self.turn[k] += 1;
                    v
                }
                fn rd4(&self, t: i32, o: usize) -> i32 {
                    let w = &self.p[(2 + t) as usize];
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn go(&mut self, tech: i32, depth: i32) -> i32 {
                    let count = self.ask(0);
                    if self.ending < self.rd4(tech, 0x1c8) {
                        return 0;
                    }
                    for _ in 0..count {
                        let preq = self.ask(1);
                        if preq < -1 {
                            return 0;
                        }
                        if self.ask(2) == 0 {
                            if self.ask(3) != 0 {
                                return 0;
                            }
                            if self.rd4(preq, 0x40) == 0x1b3 {
                                return 0;
                            }
                            if depth - 1 < 0 {
                                return 0;
                            }
                            if self.go(preq, depth - 1) == 0 {
                                return 0;
                            }
                        }
                    }
                    1
                }
            }
            let mut g = Go {
                a,
                p,
                ending: i32::from(c[0][0x36]),
                turn: [0; 4],
            };
            i64::from(g.go(a[0], a[1]))
        }
    });
}
