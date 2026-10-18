//! `Army::do_forming@006f43c0` under the emulator against the port (item 3250; tools/sweep_gen.py, obj with windows).

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

/// `Army::do_forming@006f43c0`, run by `tools/emu/sweep_army_do_forming.py` under unicorn
/// (tools/emu): 152 random bytes of object, 1 argument(s), `p` windows in order: p0..p5 sparse table 0 (2 rows × 3 slice(s), row-major), p6..p17 the place(s), in flag order, p18 stub 5's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00e85f20 points at 10064 random bytes; the global cell 0x00c061e0 points at 57344 random bytes; callee 0x006f56d0 (0 args) stubbed; callee 0x006f9120 (2 args) stubbed; callee 0x006f8750 (1 args) stubbed; callee 0x0065d260 (9 args) stubbed; callee 0x006edb50 (1 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x0070fba0 (10 args) stubbed; callee 0x0070d830 (5 args) stubbed; callee 0x0092cf40 (4 args) stubbed; callee 0x00a46a00 (0 args) stubbed; the word at `c0+0x14+0x1c*(m[0x34]&3)` points at 20 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed); the word at `p6+12` points at 32 random bytes less `0` (placed); the word at `p6+16` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*(m[0x34]&3)` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p12+4` points at 16 random bytes less `0` (placed); the word at `p12+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 240 random bytes less `0` (placed); the word at `p5+0` points at 240 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 1] (90%); stub 1 answered from [0, 0, 0, 1, 2] (90%); stub 3 answered from [-1, -1, 0, 1, 5] (90%); stub 4 answered from [0, 1] (90%); stub 5 answers the address of 128 random bytes, a `p` window after the places; stub 9 answers 8 values in call order, printed after the answers; stub 9 answers -3000..3000 every row; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 writes a value in -300000..300000 through its argument 2, printed after the answers (`o` in a place); stub 8 writes a value in -300000..300000 through its argument 3, printed after the answers (`o` in a place); stub 8 is `__cdecl`: the caller pops its arguments; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0xc, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_do_forming.py") else {
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
            {
                // `a`: the function's own (unused) stack argument, then the eight stubs' single answers in the order the
                // line names them: 1 `is_engaged`, 2 `count` (COUNT_SIEGE), 3 `set_stance`, 4 `find_building`, 5 `is_ally`,
                // 6 slot +0xac's address (the data record, p18), 7 `action_move_to`, 8 `action_siege_attack_to`, 9 `project`, 10
                // `sin_table`; 11 and 12 the point `project` writes (x', y'), 13..20 `sin_table`'s answers in call order
                // (the last repeated). Windows:
                // `c[0]` the objects singleton, `c[1]` the groups array (stride 0x9d4; `+8` the army's id, `+0xc` its
                // units), `c[2]` the leaders' table of war flags (`+0x74 + 4 * (target * 0x1bbb + who)`), `p0..p5` the leaders
                // table's slices a row (stride 0x6eec; per leader: the player `+8`, the war flags `+0x74 + 4 * target`, the
                // pointer `+0x6eb8` to the Military record), `p12` the cities' array and
                // `p13..p15` their records (flags the byte at +4), `p16`, `p17` the leaders' Military records (age at +0xe8), `p18` the
                // target's data record (the city number the short at +0x72). Logs, in order: set_stance, find_building,
                // is_ally, action_move_to, action_siege_attack_to, project, sin_table, each call `ecx,edx,args` (the `edx` of a
                // callee that is not handed one is whatever was left, `*`; `project`'s two out-pointers are caller locals, `*`); `Group::action_siege_attack_to`'s two stack words
                // between the point and the angle are never written (`sub esp, 8`), so they are masked on both sides.
                //
                // **The port is the crate's own steps**, `crate::army::do_forming_steps`, which `Sim::do_forming` calls
                // over its `FormSim`; here the view is the emulated original's windows.
                use crate::army::{FormView, do_forming_steps};
                use crate::combat::Stance;
                use crate::movement::Angle;
                use crate::orders::{MoveKind, QueuePos};
                use crate::world::{Cell, Pos};
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
                    fn gtok(g: i32) -> String {
                        format!("c1+{:#x}", g as usize * 0x9d4)
                    }
                }
                impl FormView for V<'_> {
                    fn project(&mut self, p: Pos, angle: Angle, d: i32) -> Pos {
                        self.push(
                            5,
                            format!("{},{d},{},{},sp-0x10,sp-0x14", angle.0, p.x, p.y),
                        );
                        Pos::new(self.a[11], self.a[12])
                    }
                    fn sin_table(&mut self, folded: i32, d: i32) -> i32 {
                        self.push(6, format!("{folded},{d}"));
                        let v = self.a[13 + self.sins.min(7)];
                        self.sins += 1;
                        v
                    }
                    fn engaged(&mut self) -> bool {
                        self.a[1] != 0
                    }
                    fn group_count(&self) -> i32 {
                        rd16(self.m, 0x96)
                    }
                    fn muster(&self) -> Cell {
                        Cell {
                            x: rd32(self.m, 0x48),
                            y: rd32(self.m, 0x4c),
                        }
                    }
                    fn muster_angle(&self) -> Angle {
                        Angle(rd32(self.m, 0x50))
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
                    fn hurry(&self) -> bool {
                        rd32(self.m, 0x2c) != 0
                    }
                    fn has_target_who(&self) -> bool {
                        rd32(self.m, 0x34) >= 0
                    }
                    fn target_who_differs(&self) -> bool {
                        rd32(self.m, 0x34) != rd32(&self.p[3 * self.leader()], 0)
                    }
                    fn target_enemy(&mut self) -> bool {
                        let (l, t) = (self.leader(), rd32(self.m, 0x34) as usize);
                        let who = rd32(&self.p[3 * l], 0) as usize;
                        rd32(&self.p[3 * l + 1], 4 * t) == 0
                            || rd32(&self.c[2], (t * 0x1bbb + who) * 4 + 0x74) == 0
                    }
                    fn young(&mut self) -> bool {
                        (rd32(&self.p[16 + self.leader()], 0xe8) ^ 0x63187) < 4
                    }
                    fn siege_count(&mut self) -> i32 {
                        self.a[2]
                    }
                    fn set_stance(&mut self, s: Stance) {
                        let v = match s {
                            Stance::Raid => 3,
                            Stance::Aggressive => 0,
                            _ => panic!("a stance the original does not set here"),
                        };
                        self.push(0, format!("m+0x0,*,{v}"));
                    }
                    fn enemy_combat_building_near(&mut self, p: Pos) -> bool {
                        let l = self.leader();
                        self.push(1, format!("c0+0x0,*,{},{},3,{l},768,512,9,0,0", p.x, p.y));
                        self.a[4] >= 0
                    }
                    fn target_is_me(&self) -> bool {
                        rd32(self.m, 0x34) == self.leader() as i32
                    }
                    fn target_ally(&mut self) -> bool {
                        let (l, t) = (self.leader(), rd32(self.m, 0x34));
                        self.push(2, format!("{},*,{t}", 14918544 + l * 0x6eec));
                        self.a[5] != 0
                    }
                    fn has_target_obj(&self) -> bool {
                        rd32(self.m, 0x30) >= 0
                    }
                    fn target_city_flags(&mut self) -> Option<i32> {
                        let n = rd16(&self.p[18], 0x72);
                        (n >= 0).then(|| i32::from(self.p[13 + n as usize][4]))
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
                        self.push(3, s);
                    }
                    fn siege_attack_to(&mut self, g: i32, to: Pos, angle: Angle) {
                        let s = format!("{},*,{},{},*,*,{}", Self::gtok(g), to.x, to.y, angle.0);
                        self.push(4, s);
                    }
                }
                log.extend(["-", "-", "-", "-", "-", "-", "-"].map(String::from));
                let mut v = V {
                    a,
                    m,
                    c,
                    p: &*p,
                    log: &mut log,
                    sins: 0,
                };
                i64::from(do_forming_steps(&mut v))
            }
        };
        assert_eq!(got, want_eax, "do_forming — eax, row `{line}`");
        // Two words of the original's calls are the stack's and not the function's: the `edx` of a callee that is
        // not handed one (the first five logs; `--record-edx` prints it for every stub), and the two words
        // `action_siege_attack_to` is given between the point and the angle (`sub esp, 8`, never written). Both sides
        // are masked.
        let mask = |logs: &[&str]| -> Vec<String> {
            logs.iter()
                .enumerate()
                .map(|(k, col)| {
                    col.split('/')
                        .map(|call| {
                            let mut f: Vec<&str> = call.split(',').collect();
                            if k < 5 && f.len() > 1 {
                                f[1] = "*";
                            }
                            if k == 4 && f.len() > 5 {
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
            "do_forming — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_forming — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_forming — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "do_forming — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Army::do_forming@006f43c0: {n} rows agree");
}
