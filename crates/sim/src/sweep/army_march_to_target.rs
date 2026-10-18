//! `Army::march_to_target@006f4d80` under the emulator against the port (item 3259; tools/sweep_gen.py, obj with windows).

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

/// `Army::march_to_target@006f4d80`, run by `tools/emu/sweep_army_march_to_target.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p13 the place(s), in flag order, p14 stub 8's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00e85f20 points at 10064 random bytes; callee 0x006f56d0 (0 args) stubbed; callee 0x00a46a00 (0 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x006ebaa0 (1 args) stubbed; callee 0x006f9120 (2 args) stubbed; callee 0x006f8750 (1 args) stubbed; callee 0x0065d260 (9 args) stubbed; callee 0x006edb50 (1 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x11c (1 args) stubbed; callee 0x0070fba0 (10 args) stubbed; callee 0x0070d830 (5 args) stubbed; the word at `c0+0x14+0x1c*(m[0x34]&3)` points at 20 random bytes less `0` (placed); the word at `p2+0` points at 48 random bytes less `0` (placed); the word at `p2+4` points at 48 random bytes less `0` (placed); the word at `p2+8` points at 48 random bytes less `0` (placed); the word at `p2+12` points at 48 random bytes less `0` (placed); the word at `p2+16` points at 48 random bytes less `0` (placed); the word at `0xc09970+0x1c*(m[0x34]&3)` points at 12 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p8+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 240 random bytes less `0` (placed); the word at `p1+0` points at 240 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 0, 0, 1, 2] (90%); stub 6 answered from [-1, -1, 0, 1, 5] (90%); stub 7 answered from [0, 1] (90%); stub 9 answered from [0, 10, 100, 1000, -5, 3] (90%); stub 8 answers the address of 128 random bytes, a `p` window after the places; stub 1 answers 10 values in call order, printed after the answers; stub 2 answers 1 values in call order, printed after the answers; stub 1 answers -3000..3000 every row; stub 2 answers -2147483648..2147483647 every row; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_march_to_target.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: the twelve stubs' single answers in the order the line names them (0 `is_engaged`, 1 and 2 unused
                // here, 3 `is_enemy`, 4 `count` (COUNT_SIEGE), 5 unused, 6 `find_building`, 7 `is_ally`, 8 slot +0xac's
                // address (p14), 9 slot +0x11c's hits, 10 and 11 unused), then `a[12..22]` `sin_table`'s answers in call
                // order (the last repeated), `a[22]` `find_angle`'s. Windows: `c[0]` the objects singleton, `c[1]` the
                // groups array (stride 0x9d4; `+8` the army's id, `+0xc` its units), `p[0..2]` the leaders' `+0x6eb8`
                // slices (stride 0x6eec), `p[2]` the target object table, `p[3..8]` its objects (`+8` flags, `+0x10`/`+0x14`
                // the point `^ 0x63637`, `+0x24` damage), `p[8]` the cities' array, `p[9..12]` their records (flags the
                // byte at +4), `p[12]`, `p[13]` the leaders' Military records (age at +0xe8), `p[14]` the target's data
                // record (the city number the short at +0x72). Logs, one per stub, each call `ecx,edx,args`; the `edx` of a
                // callee that is not handed one is whatever was left, `*`, masked below.
                //
                // **The port is the crate's own steps**, `crate::army::march_to_target_steps`, which
                // `Sim::march_to_target` calls over its `MarchToSim`; here the view is the emulated original's windows.
                use crate::army::{MarchToView, march_to_target_steps};
                use crate::combat::Stance;
                use crate::movement::Angle;
                use crate::orders::{MoveKind, QueuePos};
                use crate::world::Pos;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    sins: usize,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
                }
                impl V<'_> {
                    fn push(&mut self, k: usize, s: String) {
                        if self.log[k] == "-" {
                            self.log[k] = s;
                        } else {
                            self.log[k].push('/');
                            self.log[k].push_str(&s);
                        }
                    }
                    fn leader(&self) -> usize {
                        rd16(self.m, 0x94) as usize
                    }
                    fn tobj(&self) -> usize {
                        rd32(self.m, 0x30) as usize
                    }
                    fn obj(&self) -> &[u8] {
                        &self.p[3 + self.tobj()]
                    }
                    fn gtok(g: i32) -> String {
                        format!("c1+{:#x}", g as usize * 0x9d4)
                    }
                }
                impl MarchToView for V<'_> {
                    fn has_target_obj(&self) -> bool {
                        rd32(self.m, 0x30) >= 0
                    }
                    fn engaged(&mut self) -> bool {
                        self.push(0, "m+0x0,*".to_string());
                        self.a[0] != 0
                    }
                    fn status(&self) -> i32 {
                        rd32(self.m, 4)
                    }
                    fn set_status(&mut self, s: i32) {
                        self.m[4..8].copy_from_slice(&s.to_le_bytes());
                    }
                    fn muster_angle(&self) -> Angle {
                        Angle(rd32(self.m, 0x50))
                    }
                    fn pos(&self) -> Pos {
                        Pos::new(rd32(self.m, 0x38), rd32(self.m, 0x3c))
                    }
                    fn sin_table(&mut self, folded: i32, d: i32) -> i32 {
                        self.push(1, format!("{folded},{d}"));
                        let v = self.a[12 + self.sins.min(9)];
                        self.sins += 1;
                        v
                    }
                    fn target_pos(&self) -> Pos {
                        let o = self.obj();
                        Pos::new(rd32(o, 0x10) ^ 0x63637, rd32(o, 0x14) ^ 0x63637)
                    }
                    fn find_angle(&mut self, dx: i32, dy: i32) -> Angle {
                        self.push(2, format!("{dx},{dy}"));
                        Angle(self.a[22])
                    }
                    fn target_enemy(&mut self) -> bool {
                        let (l, t) = (self.leader(), rd32(self.m, 0x34));
                        self.push(3, format!("{},*,{t}", 14918544 + l * 0x6eec));
                        self.a[3] != 0
                    }
                    fn hurry(&self) -> bool {
                        rd32(self.m, 0x2c) != 0
                    }
                    fn young(&mut self) -> bool {
                        (rd32(&self.p[12 + self.leader()], 0xe8) ^ 0x63187) < 4
                    }
                    fn siege_count(&mut self) -> i32 {
                        self.push(4, "m+0x0,*,4,0".to_string());
                        self.a[4]
                    }
                    fn set_stance(&mut self, s: Stance) {
                        let v = match s {
                            Stance::Raid => 3,
                            _ => panic!("a stance the original does not set here"),
                        };
                        self.push(5, format!("m+0x0,*,{v}"));
                    }
                    fn enemy_combat_building_near(&mut self, p: Pos) -> bool {
                        let l = self.leader();
                        self.push(6, format!("c0+0x0,*,{},{},3,{l},768,512,9,0,0", p.x, p.y));
                        self.a[6] >= 0
                    }
                    fn target_is_me(&self) -> bool {
                        rd32(self.m, 0x34) == self.leader() as i32
                    }
                    fn target_ally(&mut self) -> bool {
                        let (l, t) = (self.leader(), rd32(self.m, 0x34));
                        self.push(7, format!("{},*,{t}", 14918544 + l * 0x6eec));
                        self.a[7] != 0
                    }
                    fn target_city_flags(&mut self) -> Option<i32> {
                        self.push(8, format!("p{}+0x0,*", 3 + self.tobj()));
                        let n = rd16(&self.p[14], 0x72);
                        (n >= 0).then(|| i32::from(self.p[9 + n as usize][4]))
                    }
                    fn group_count(&self) -> i32 {
                        rd16(self.m, 0x96)
                    }
                    fn group_id(&self, i: i32) -> i32 {
                        rd32(self.m, 0x54 + 4 * i as usize)
                    }
                    fn group_populated(&mut self, g: i32) -> bool {
                        rd32(&self.c[1], g as usize * 0x9d4 + 0xc) != 0
                    }
                    fn group_claim(&mut self, g: i32) {
                        let v = rd16(self.m, 2);
                        let o = g as usize * 0x9d4 + 8;
                        self.c[1][o..o + 4].copy_from_slice(&v.to_le_bytes());
                    }
                    fn target_city_centre(&mut self) -> bool {
                        rd32(self.obj(), 8) & 0x20 != 0
                    }
                    fn target_hits(&mut self) -> i32 {
                        let t = format!("p{}+0x0,p{}+0x0,0", 3 + self.tobj(), 3 + self.tobj());
                        self.push(9, t);
                        self.a[9]
                    }
                    fn target_damage(&mut self) -> i32 {
                        rd32(self.obj(), 0x24)
                    }
                    fn move_to(
                        &mut self,
                        g: i32,
                        to: Pos,
                        queue: QueuePos,
                        set_angle: bool,
                        angle: Angle,
                        kind: MoveKind,
                        action: bool,
                    ) {
                        let q = match queue {
                            QueuePos::New => 2,
                            _ => panic!("a queue the original does not pass here"),
                        };
                        let k = match kind {
                            MoveKind::AttackTo => 2,
                            _ => panic!("a kind the original does not pass here"),
                        };
                        let s = format!(
                            "{},*,{},{},{q},{},{},{k},{},-1,-1,0",
                            Self::gtok(g),
                            to.x,
                            to.y,
                            i32::from(set_angle),
                            angle.0,
                            i32::from(action),
                        );
                        self.push(10, s);
                    }
                    fn siege_attack_to(&mut self, g: i32, to: Pos, angle: Angle) {
                        let s = format!("{},*,{},{},*,*,{}", Self::gtok(g), to.x, to.y, angle.0);
                        self.push(11, s);
                    }
                }
                log.extend(std::iter::repeat_n("-".to_string(), 12));
                let mut v = V {
                    a,
                    m,
                    c,
                    p: &*p,
                    log: &mut log,
                    sins: 0,
                };
                march_to_target_steps(&mut v);
                0
            }
        };
        let _ = (got, want_eax);
        // The `edx` of a callee that is not handed one is the stack's and not the function's (every stub but
        // `sin_table`, `find_angle` and the hits slot), and so are the two words `action_siege_attack_to` is given
        // between the point and the angle (`sub esp, 8`, never written). Both sides are masked.
        let mask = |logs: &[&str]| -> Vec<String> {
            logs.iter()
                .enumerate()
                .map(|(k, col)| {
                    col.split('/')
                        .map(|call| {
                            let mut f: Vec<&str> = call.split(',').collect();
                            if ![1, 2, 9].contains(&k) && f.len() > 1 {
                                f[1] = "*";
                            }
                            if k == 11 && f.len() > 5 {
                                f[4] = "*";
                                f[5] = "*";
                            }
                            f.join(",")
                        })
                        .collect::<Vec<_>>()
                        .join("/")
                })
                .collect()
        };
        let mine: Vec<&str> = log.iter().map(String::as_str).collect();
        assert_eq!(
            mask(&mine),
            mask(&want_log),
            "march_to_target — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "march_to_target — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "march_to_target — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "march_to_target — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 5000, "only {n} rows");
    eprintln!("Army::march_to_target@006f4d80: {n} rows agree");
}
