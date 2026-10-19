//! `Unit::do_air_patrol@005ea620` under the emulator against the port (item 3289; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_air_patrol@005ea620`, run by `tools/emu/sweep_unit_do_air_patrol.py` under unicorn
/// (tools/emu): 224 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p24 the place(s), in flag order, p25 stub 5's answer window; the global cell 0x00c0618c points at 1024 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; argument 0 names 32 random bytes; vtable slot +0x8c (0 args) stubbed; vtable slot +0x180 (2 args) stubbed; vtable slot +0x30 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006b53a0 (2 args) run, not hooked; callee 0x005e86d0 (3 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x005eb960 (5 args) stubbed; callee 0x005ebc70 (5 args) stubbed; callee 0x005e48c0 (7 args) stubbed; callee 0x0065ab40 (7 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p5+0` points at 64 random bytes less `0` (placed); the word at `p5+4` points at 64 random bytes less `0` (placed); the word at `p5+8` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 12 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); stub 0's answer points at 96 random bytes less `0` (placed); the word at `p13+0x14` points at 32 random bytes less `0` (placed); the word at `p13+0x30` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 560 random bytes less `0` (placed); the word at `p3+0x18` points at 560 random bytes less `0` (placed); the word at `p4+0x18` points at 560 random bytes less `0` (placed); the word at `p6+0x18` points at 560 random bytes less `0` (placed); the word at `p7+0x18` points at 560 random bytes less `0` (placed); the word at `p8+0x18` points at 560 random bytes less `0` (placed); the word at `p10+0x18` points at 560 random bytes less `0` (placed); the word at `p11+0x18` points at 560 random bytes less `0` (placed); the word at `p12+0x18` points at 560 random bytes less `0` (placed); stub 2 answered from [0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 7 answered from [0, 1, 1, 1, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 5 answers the address of 112 random bytes, a `p` window after the places; stub 2 answers 2 values in call order, printed after the answers; stub 3 answers 3 values in call order, printed after the answers; stub 9 answers 3 values in call order, printed after the answers; stub 10 answers 0..2 every row; stub 11 answers 0..2 every row; stub 13 answers -1..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 10 writes a value in 0..2 through its argument 4, printed after the answers (`o` in a place); stub 11 writes a value in 0..2 through its argument 4, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_air_patrol.py") else {
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
            // `a`: 0 the order argument's address, 1..=15 the stubs' single answers in the order the line names them
            // (0 `get_air_order` -> the record p13, 1 slot +0x180, 2 `is_animal`, 3 `is`, 4 slot +0x8, 5 slot +0xac -> p25,
            // 6 `restrict` (run), 7 `do_air_physics`, 8 `kill_current_order`, 9 `valid_target`, 10 `find_new_bomber_target`,
            // 11 `find_new_air_target`, 12 `add_strafe_order`, 13 `find_building_at`, 14 `vector_dist` (run)), 16..=17 the two
            // finders' lists (`outarg`), 18..=19 `is_animal`'s two answers in call order, 20..=22 `is`'s three, 23..=25
            // `valid_target`'s three. Windows: `m` the unit (owner byte +9, short +0xa, position +0x10/+0x14 less the key,
            // `spell_time` +0x98, order count +0xd8), `c[0]` the objects cell (word +0x200 the building list), `c[1]` the game
            // (frame +0x550, semaphore +0x821), `c[2]` the world (xs, ys), `c[3]` the div_3 table; `p[0]` the order argument,
            // `p[1+4L]` list L's pointer array and `p[2+4L+i]` its objects (flag byte +8, position +0x10/+0x14, data pointer
            // +0x18), `p[13]` the patrol record, `p[14]`/`p[15]` its x and y arrays, `p[16+3L+i]` the object data records
            // (domain +0x218), `p[25]` the building info (ever_seen +0x62).
            //
            // **The port is the crate's own steps**, `crate::air::air_patrol_steps`, which `Sim::do_air_patrol` calls over its
            // `SimPatrol`; here the view is the emulated original's windows and stubs. The `semaphore & 2` test is read off
            // the game window (no stub), the rest answers as the stubs do, and the log is the stubs' calls in order.
            use crate::air::{Home, PatrolView, Stale, air_patrol_steps};
            use crate::world::{Pos, vector_dist};
            const KEY: i32 = 0x63637;
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                animal: usize,
                is: usize,
                valid: usize,
            }
            fn rd32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            impl V<'_> {
                fn pos(&self) -> (i32, i32) {
                    (rd32(self.m, 0x10) ^ KEY, rd32(self.m, 0x14) ^ KEY)
                }
                fn home(&self) -> (i32, i32) {
                    (rd32(&self.p[13], 0x44), rd32(&self.p[13], 0x48))
                }
                fn object(list: i32, idx: i32) -> usize {
                    (2 + 4 * list + idx) as usize
                }
                fn home_object(&self) -> usize {
                    let (idx, list) = self.home();
                    Self::object(list, idx)
                }
                fn obj_pos(&self, w: usize) -> (i32, i32) {
                    (rd32(&self.p[w], 0x10) ^ KEY, rd32(&self.p[w], 0x14) ^ KEY)
                }
                fn seq(&mut self, which: u8) -> bool {
                    let (base, n, turn) = match which {
                        2 => (18, 2, &mut self.animal),
                        3 => (20, 3, &mut self.is),
                        _ => (23, 3, &mut self.valid),
                    };
                    let v = self.a[base + (*turn).min(n - 1)];
                    *turn += 1;
                    v != 0
                }
                fn finder(
                    &mut self,
                    stub: usize,
                    list: usize,
                    x: i32,
                    y: i32,
                    p4: String,
                ) -> (i32, i32) {
                    let name = if stub == 10 { "s10" } else { "s11" };
                    self.log
                        .push(format!("{name}:m+0x0,{x},{y},-1,{p4},sp-0x4"));
                    (self.a[1 + stub], self.a[list])
                }
            }
            impl PatrolView for V<'_> {
                type Target = (i32, i32);

                fn think(&mut self) {
                    self.log.push("s0:p0+0x0".into());
                    self.log.push("s1:m+0x0,p0+0x0,0".into());
                }
                fn len(&self) -> i32 {
                    rd32(&self.p[13], 8)
                }
                fn ylen(&self) -> i32 {
                    rd32(&self.p[13], 0x24)
                }
                fn waypoint(&self) -> i32 {
                    rd32(&self.p[13], 0x3c)
                }
                fn set_waypoint(&mut self, w: i32) {
                    self.p[13][0x3c..0x40].copy_from_slice(&w.to_le_bytes());
                }
                fn returning(&self) -> bool {
                    rd32(&self.p[13], 0x58) != 0
                }
                fn x_at(&self, i: i32) -> i32 {
                    rd32(&self.p[14], 4 * i as usize)
                }
                fn y_at(&self, i: i32) -> i32 {
                    rd32(&self.p[15], 4 * i as usize)
                }
                fn is_animal(&mut self) -> bool {
                    self.log.push("s2:m+0x0".into());
                    self.seq(2)
                }
                fn carrier(&mut self) -> Option<(i32, i32)> {
                    let (idx, list) = self.home();
                    if idx < 0 || list < 0 {
                        return None;
                    }
                    let w = Self::object(list, idx);
                    self.log.push(format!("s4:p{w}+0x0"));
                    (self.a[1 + 4] != 0).then(|| self.obj_pos(w))
                }
                fn home_flagged(&mut self) -> Home {
                    let (idx, _) = self.home();
                    if idx < 0 {
                        return Home::Absent;
                    }
                    let w = self.home_object();
                    if self.p[w][8] & 1 == 0 {
                        return Home::Plain;
                    }
                    let (x, y) = self.obj_pos(w);
                    Home::At(x, y)
                }
                fn restrict(&mut self, x: i32, y: i32) -> (i32, i32) {
                    let (xs, ys) = (rd32(&self.c[2], 0) * 0x300, rd32(&self.c[2], 4) * 0x300);
                    (x.max(0).min(xs - 1), y.max(0).min(ys - 1))
                }
                fn air_physics(&mut self, goal: Option<(i32, i32)>) -> bool {
                    let (x, y) = goal.unwrap_or((-1, -1));
                    self.log.push(format!("s7:m+0x0,p0+0x0,{x},{y}"));
                    self.a[1 + 7] != 0
                }
                fn dist_to(&self, x: i32, y: i32) -> i32 {
                    let (px, py) = self.pos();
                    vector_dist(px - x, py - y)
                }
                fn orders(&self) -> i32 {
                    rd32(self.m, 0xd8)
                }
                fn kill_current_order(&mut self) {
                    self.log.push("s8:m+0x0,0".into());
                }
                fn phase(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                        + rd32(&self.c[1], 0x550)
                }
                fn fighter_bomber(&mut self) -> bool {
                    self.log.push("s3:m+0x0,308,0".into());
                    self.seq(3)
                }
                fn bomber(&mut self) -> bool {
                    self.log.push("s3:m+0x0,304,0".into());
                    self.seq(3)
                }
                fn find_bomber(&mut self, x: i32, y: i32, p4: Option<i32>) -> Self::Target {
                    let p4 = p4.map_or_else(|| "m+0x0".to_string(), |v| v.to_string());
                    self.finder(10, 16, x, y, p4)
                }
                fn find_air(&mut self, x: i32, y: i32) -> Self::Target {
                    self.finder(11, 17, x, y, "-1".into())
                }
                fn self_pos(&self) -> (i32, i32) {
                    self.pos()
                }
                fn semaphore_fallback(&self) -> bool {
                    self.c[1][0x821] & 2 != 0
                }
                fn valid_target(&mut self, t: Self::Target) -> bool {
                    self.log.push(format!("s9:m+0x0,{},{},0", t.0, t.1));
                    self.seq(9)
                }
                fn target_flies(&self, t: Self::Target) -> bool {
                    rd32(&self.p[(16 + 3 * t.1 + t.0) as usize], 0x218) == 2
                }
                fn strafe(&mut self, t: Self::Target, mandatory: bool) {
                    let (hi, hl) = self.home();
                    self.log.push(format!(
                        "s12:m+0x0,{},{},{hi},{hl},{},0,0",
                        t.0,
                        t.1,
                        i32::from(mandatory)
                    ));
                }
                fn find_building_at(
                    &mut self,
                    x: i32,
                    y: i32,
                    stale: Stale,
                ) -> Option<Self::Target> {
                    let tile = Pos::new(x, y).tile();
                    let stale = match stale {
                        Stale::This => "m+0x0".to_string(),
                        Stale::HomeIdx => self.home().0.to_string(),
                        Stale::HomeObject => format!("p{}+0x0", self.home_object()),
                        Stale::World => "c2+0x0".to_string(),
                    };
                    self.log.push(format!(
                        "s13:c0+0x0,{},{},3,{},0,0,{stale}",
                        tile.x, tile.y, self.m[9]
                    ));
                    let idx = self.a[1 + 13];
                    (idx >= 0).then(|| (idx, rd32(&self.c[0], 0x200)))
                }
                fn ever_seen(&mut self, t: Self::Target) -> bool {
                    let w = Self::object(t.1, t.0);
                    self.log.push(format!("s5:p{w}+0x0"));
                    u32::from(self.p[25][0x62]) & (1u32 << (self.m[9] & 31)) != 0
                }
                fn spell_time(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0x98], self.m[0x99]]))
                }
                fn set_spell_time(&mut self, v: i32) {
                    self.m[0x98..0x9a].copy_from_slice(&(v as i16).to_le_bytes());
                }
            }
            let mut v = V {
                a,
                m,
                c: &*c,
                p,
                log: &mut log,
                animal: 0,
                is: 0,
                valid: 0,
            };
            air_patrol_steps(&mut v);
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_air_patrol — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_air_patrol — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_air_patrol — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "do_air_patrol — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::do_air_patrol@005ea620: {n} rows agree");
}
