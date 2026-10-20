//! `Objects::find_builds@0065a120` under the emulator against the port (item 3379; tools/sweep_gen.py, obj with windows).

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

/// `Objects::find_builds@0065a120`, run by `tools/emu/sweep_objects_find_builds.py` under unicorn
/// (tools/emu): 64 random bytes of object, 10 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major), p8..p36 the place(s), in flag order; the global cell 0x00c0618c points at 560 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061ec points at 1728 random bytes; callee 0x0067daa0 (2 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x0 (1 args) stubbed; the word at `c0+0x14+0x1c*0` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*1` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*2` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*3` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*4` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*5` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*6` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*7` points at 8 random bytes less `8000` (placed); the word at `c0+0x14+0x1c*8` points at 8 random bytes less `8000` (placed); the word at `p8+0` points at 576 random bytes less `0` (placed); the word at `p8+4` points at 576 random bytes less `0` (placed); the word at `p9+0` points at 576 random bytes less `0` (placed); the word at `p9+4` points at 576 random bytes less `0` (placed); the word at `p10+0` points at 576 random bytes less `0` (placed); the word at `p10+4` points at 576 random bytes less `0` (placed); the word at `p11+0` points at 576 random bytes less `0` (placed); the word at `p11+4` points at 576 random bytes less `0` (placed); the word at `p12+0` points at 576 random bytes less `0` (placed); the word at `p12+4` points at 576 random bytes less `0` (placed); the word at `p13+0` points at 576 random bytes less `0` (placed); the word at `p13+4` points at 576 random bytes less `0` (placed); the word at `p14+0` points at 576 random bytes less `0` (placed); the word at `p14+4` points at 576 random bytes less `0` (placed); the word at `p15+0` points at 576 random bytes less `0` (placed); the word at `p15+4` points at 576 random bytes less `0` (placed); the word at `p16+0` points at 576 random bytes less `0` (placed); the word at `p16+4` points at 576 random bytes less `0` (placed); the word at `c2+0x134` points at 1376 random bytes less `0` (placed); the word at `c0+0x214` points at 1024 random bytes less `0` (placed); the stubbed vtable laid at `p17+0`; the word at `p17+0x118` laid as the address of `p17` every row; the word at `p18+0x118` laid as the address of `p18` every row; the word at `p19+0x118` laid as the address of `p19` every row; the word at `p20+0x118` laid as the address of `p20` every row; the word at `p21+0x118` laid as the address of `p21` every row; the word at `p22+0x118` laid as the address of `p22` every row; the word at `p23+0x118` laid as the address of `p23` every row; the word at `p24+0x118` laid as the address of `p24` every row; the word at `p25+0x118` laid as the address of `p25` every row; the word at `p26+0x118` laid as the address of `p26` every row; the word at `p27+0x118` laid as the address of `p27` every row; the word at `p28+0x118` laid as the address of `p28` every row; the word at `p29+0x118` laid as the address of `p29` every row; the word at `p30+0x118` laid as the address of `p30` every row; the word at `p31+0x118` laid as the address of `p31` every row; the word at `p32+0x118` laid as the address of `p32` every row; the word at `p33+0x118` laid as the address of `p33` every row; the word at `p34+0x118` laid as the address of `p34` every row; stub 0 answers 30 values in call order, printed after the answers; stub 1 answers 30 values in call order, printed after the answers; stub 2 answers 30 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 1 answers 0..3 every row; stub 2 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; `0x006817f0` run once before the rows (a table builder the startup would have run); stub 3 answers from its own `ecx`: the word at `+0x100`, or `ecx+0x100` when the word at `+0x104` is set; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `c0+0x204` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_find_builds.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let mut want_log: Vec<&str> = calls.split_whitespace().collect();
        if want_log == ["-"] {
            want_log.clear();
        }
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
            // `a`: 0 x, 1 y, 2 search, 3 who, 4 range, 5 flags, 6 filter, 7 p8, 8 (unread), 9 the continue flag; 10..15 the stubs'
            // plain answers; then three 30-answer sequences in call order (valid_search, exists, valid_filter), the last repeated.
            // Windows: c0 the `objects` record (`+0x14 + 0x1c k` the players' arrays, `+0x184 + 4k` their counts, `+0x1fc`/`+0x200`
            // the nearest pair, `+0x204` the result array's vtable, `+0x208` its length, `+0x20c` its capacity, `+0x210` the grow
            // argument, `+0x214` its data); c1 `div_3_table`; c2 the world (`+0` width, `+4` height, `+0x134` the cells, 0x1c apiece);
            // c3 the game (`+0x6b0` `num_def_builds`). p0..p7 the leaders' flag word; p8..p16 the players' arrays (entry `id - 2000`);
            // p17.. the objects, `2 * player + (id - 2000)`; p35 the cells; p36 the result data. The port is the crate's own
            // `find_builds_view::find_builds_steps` over a view of the windows.
            use crate::find_builds_view::{FindBuildsArgs, FindBuildsView, find_builds_steps};
            struct V<'a> {
                a: &'a [i32],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                calls: [usize; 3],
            }
            impl V<'_> {
                fn obj(player: i32, id: i32) -> usize {
                    assert!(
                        (0..=8).contains(&player) && (2000..2002).contains(&id),
                        "find_builds — object ({player}, {id}) names no window"
                    );
                    17 + (2 * player + (id - 2000)) as usize
                }
                fn ans(&mut self, stub: usize) -> i32 {
                    let k = self.calls[stub];
                    self.calls[stub] += 1;
                    self.a[15 + 30 * stub + k.min(29)]
                }
                fn short(w: &[u8], off: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
                }
            }
            impl FindBuildsView for V<'_> {
                fn clear_len(&mut self) {
                    wr(&mut self.c[0], 0x208, 0);
                }
                fn note_search(&mut self, who: i32) {
                    wr(&mut self.c[0], 0x200, who);
                    wr(&mut self.c[0], 0x1fc, 99_999_999);
                }
                fn div3(&mut self, i: i32) -> i32 {
                    rd(&self.c[1], 4 * i as usize)
                }
                fn width(&mut self) -> i32 {
                    rd(&self.c[2], 0)
                }
                fn height(&mut self) -> i32 {
                    rd(&self.c[2], 4)
                }
                fn cell_region(&mut self, index: i32) -> i32 {
                    Self::short(&self.p[35], index as usize * 0x1c + 4)
                }
                fn cell_head(&mut self, index: i32) -> (i32, i32) {
                    let o = index as usize * 0x1c;
                    (
                        Self::short(&self.p[35], o + 8),
                        Self::short(&self.p[35], o + 10),
                    )
                }
                fn num_def_builds(&mut self) -> i32 {
                    rd(&self.c[3], 0x6b0)
                }
                fn circle_radius(&mut self, ring: i32) -> i32 {
                    crate::ai_place::circle().radius[ring as usize] as i32
                }
                fn circle_point(&mut self, i: i32) -> (i32, i32) {
                    let c = crate::ai_place::circle();
                    (c.x[i as usize], c.y[i as usize])
                }
                fn link(&mut self, player: i32, id: i32) -> (i32, i32) {
                    let w = &self.p[Self::obj(player, id)];
                    (Self::short(w, 0x2c), Self::short(w, 0x2e))
                }
                fn valid_search(&mut self, player: i32, search: i32, who: i32) -> bool {
                    self.log.push(format!("s0:{player},{search},{who}"));
                    self.ans(0) != 0
                }
                fn exists(&mut self, player: i32, id: i32) -> bool {
                    self.log.push(format!("s1:p{}+0x0", Self::obj(player, id)));
                    self.ans(1) != 0
                }
                fn valid_filter(&mut self, player: i32, id: i32, p8: i32, filter: i32) -> bool {
                    self.log.push(format!("s2:{player},{id},{p8},0,{filter}"));
                    self.ans(2) != 0
                }
                fn push(&mut self, player: i32, id: i32) {
                    let len = rd(&self.c[0], 0x208);
                    if rd(&self.c[0], 0x20c) <= len {
                        let arg = Self::short(&self.c[0], 0x210);
                        self.log.push(format!("s4:c0+0x204,{arg}"));
                    }
                    let entry = rd(&self.p[8 + player as usize], 4 * (id - 2000) as usize);
                    wr(&mut self.p[36], 4 * len as usize, entry);
                    wr(&mut self.c[0], 0x208, len + 1);
                }
                fn result_len(&mut self) -> i32 {
                    rd(&self.c[0], 0x208)
                }
                fn leader_in_game(&mut self, k: i32) -> bool {
                    self.p[k as usize][0] & 1 != 0
                }
                fn count(&mut self, k: i32) -> i32 {
                    rd(&self.c[0], 0x184 + 4 * k as usize)
                }
                fn position(&mut self, player: i32, id: i32) -> (i32, i32) {
                    let w = &self.p[Self::obj(player, id)];
                    (rd(w, 0x10), rd(w, 0x14))
                }
                fn reach_x(&mut self, player: i32, id: i32) -> i32 {
                    let o = Self::obj(player, id);
                    self.log.push(format!("s3:p{o}+0x0"));
                    rd(&self.p[o], 0x234)
                }
                fn reach_y(&mut self, player: i32, id: i32) -> i32 {
                    let o = Self::obj(player, id);
                    self.log.push(format!("s3:p{o}+0x0"));
                    rd(&self.p[o], 0x238)
                }
            }
            let args = FindBuildsArgs {
                x: a[0],
                y: a[1],
                search: a[2],
                who: a[3],
                range: a[4],
                flags: a[5],
                filter: a[6],
                p8: a[7],
                cont: a[9],
            };
            let mut v = V {
                a,
                c,
                p,
                log: &mut log,
                calls: [0; 3],
            };
            i64::from(find_builds_steps(&mut v, &args))
        };
        assert_eq!(got, want_eax, "find_builds — eax, args {:?}", &a[..10]);
        if log != want_log {
            let k = log
                .iter()
                .zip(&want_log)
                .position(|(x, y)| x != y)
                .unwrap_or(log.len().min(want_log.len()));
            panic!(
                "find_builds — the recorded calls part at {k}: ours {:?} theirs {:?}; args {:?}",
                &log[k.saturating_sub(2)..(k + 3).min(log.len())],
                &want_log[k.saturating_sub(2)..(k + 3).min(want_log.len())],
                &a[..10]
            );
        }
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_builds — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_builds — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "find_builds — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Objects::find_builds@0065a120: {n} rows agree");
}
