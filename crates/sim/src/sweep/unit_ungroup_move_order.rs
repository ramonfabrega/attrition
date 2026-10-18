//! `Unit::ungroup_move_order@005fd140` under the emulator against the port (item 3268; tools/sweep_gen.py, obj with windows).

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

/// The emulator's windows as the port's view (`crate::orders::UngroupView`).
///
/// Units: 0 is `this` (`m`), 1 and 2 are `c1`/`c2` at `UNIT1`/`UNIT2`, the
/// table `c0` holding their addresses. Node `k` is `c[3 + k]` at
/// `NODES + 0x10 k`; order `j` is `c[9 + j]` at `ORDERS + 0x400 j`. Each
/// order window lays what the vslots answer from: the record the order's
/// `+0x110` names at `+0x20` (its id at `+0x7c`, asked when `+0x1c` is set),
/// the group order's record at `+0x80` (`+0x98`, base `+0x84`, kind at
/// `+0x94`, the leader's id and player at `+0xd4`/`+0xd8`), and the fresh
/// order `get_obj` answers at `+0x100` (`+0x44` and `+0x40` answer inside
/// it).
struct Win<'a> {
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    /// Slot `+0xe8` was laid as the real `UnitData::is_captain`.
    real_captain: bool,
    /// The order whose group record is being converted.
    cur: usize,
    attack: bool,
    /// The unit whose list the last `remove_current` was asked of.
    listu: usize,
}

const UNIT1: i32 = 0xe9_1000;
const UNIT2: i32 = 0xe9_1200;
const NODES: i32 = 0xe9_0000;
const ORDERS: i32 = 0xe9_2000;

impl Win<'_> {
    fn unit(&mut self, u: usize) -> &mut [u8] {
        match u {
            0 => &mut *self.m,
            k => &mut self.c[k],
        }
    }
    fn uname(u: usize) -> String {
        if u == 0 { "m".into() } else { format!("c{u}") }
    }
    /// A pointer into a window, as the recorded calls print it.
    fn tok(a: i32) -> String {
        let spans = [(UNIT1, 0x100, 1), (UNIT2, 0x100, 2)];
        for (base, size, k) in spans {
            if (base..base + size).contains(&a) {
                return format!("c{k}+{:#x}", a - base);
            }
        }
        if (NODES..NODES + 0x60).contains(&a) {
            let k = 3 + (a - NODES) as usize / 0x10;
            return format!("c{k}+{:#x}", (a - NODES) % 0x10);
        }
        if (ORDERS..ORDERS + 0x1800).contains(&a) {
            let k = 9 + (a - ORDERS) as usize / 0x400;
            return format!("c{k}+{:#x}", (a - ORDERS) % 0x400);
        }
        a.to_string()
    }
    fn at(&mut self, a: i32) -> (&mut Vec<u8>, usize) {
        for (base, k) in [(UNIT1, 1), (UNIT2, 2)] {
            if (base..base + 0x100).contains(&a) {
                return (&mut self.c[k], (a - base) as usize);
            }
        }
        if (NODES..NODES + 0x60).contains(&a) {
            let k = 3 + (a - NODES) as usize / 0x10;
            return (&mut self.c[k], (a - NODES) as usize % 0x10);
        }
        assert!(
            (ORDERS..ORDERS + 0x1800).contains(&a),
            "address {a:#x} outside the windows"
        );
        let k = 9 + (a - ORDERS) as usize / 0x400;
        (&mut self.c[k], (a - ORDERS) as usize % 0x400)
    }
    fn rd(&mut self, a: i32) -> i32 {
        let (w, o) = self.at(a);
        rd(w, o)
    }
    fn wr(&mut self, a: i32, v: i32) {
        let (w, o) = self.at(a);
        wr(w, o, v);
    }
    fn order_base(&self, o: usize) -> i32 {
        ORDERS + 0x400 * (o as i32 - 9)
    }
    /// The unit window `u`'s `+0xcc` list, as the recorded calls print it.
    fn list(u: usize) -> String {
        format!("{}+0xcc", Self::uname(u))
    }
    /// The node `a` points at, as a window index.
    fn node(&self, a: i32) -> usize {
        3 + ((a - NODES) / 0x10) as usize
    }
    /// The base of a record reached through its virtual-base pointer.
    fn vbase(&mut self, p: i32) -> i32 {
        let vb = self.rd(p);
        p + self.rd(vb + 4)
    }
    /// `vslot +0x40`'s answer for `ecx`: `ecx + 0x30` when `ecx + 0x2c` is set.
    fn s40(&mut self, ecx: i32) -> i32 {
        self.log.push(format!("s5:{}", Self::tok(ecx)));
        if self.rd(ecx + 0x2c) != 0 {
            ecx + 0x30
        } else {
            0
        }
    }
}

impl crate::orders::UngroupView for Win<'_> {
    type U = usize;
    type O = usize;
    type G = usize;
    type N = usize;
    fn is_captain(&mut self, u: usize) -> bool {
        let real = self.real_captain;
        let w = self.unit(u);
        if real {
            ((rd(w, 0x8c) >> 16) as i16) < 0
        } else {
            rd(w, 0x8c) != 0
        }
    }
    fn up(&mut self, u: usize) -> Option<usize> {
        let idx = (rd(self.unit(u), 0x8c) >> 16) as i16 as i32;
        let a = rd(&self.c[0], 4 * idx as usize);
        Some(if a == UNIT1 { 1 } else { 2 })
    }
    fn down(&mut self, u: usize) -> Option<usize> {
        let w = self.unit(u);
        let idx = rd(w, 0x90) as i16 as i32;
        if idx < 0 {
            return None;
        }
        let a = rd(&self.c[0], 4 * idx as usize);
        let d = if a == UNIT1 { 1 } else { 2 };
        (self.unit(d)[8] & 1 != 0).then_some(d)
    }
    fn start_cursor(&mut self, u: usize) {
        let head = rd(self.unit(u), 0xdc);
        if head != 0 {
            let n = self.node(head);
            let (data, metric) = (rd(&self.c[n], 8), self.c[n][0xc]);
            let w = self.unit(u);
            wr(w, 0xd4, head);
            wr(w, 0xcc, data);
            w[0xd0] = metric;
        }
    }
    fn has_orders(&mut self, u: usize) -> bool {
        rd(self.unit(u), 0xd8) != 0
    }
    fn cursor_order(&mut self, u: usize) -> Option<usize> {
        let a = rd(self.unit(u), 0xcc);
        (a != 0).then(|| 9 + ((a - ORDERS) / 0x400) as usize)
    }
    fn carries(&mut self, o: usize, id: i64) -> bool {
        self.log.push(format!("s1:c{o}+0x0"));
        if rd(&self.c[o], 0x1c) == 0 {
            return false;
        }
        self.log.push(format!("s1:c{o}+0x0"));
        i64::from(rd(&self.c[o], 0x7c)) == id
    }
    fn has_head(&mut self, u: usize) -> bool {
        rd(self.unit(u), 0xdc) != 0
    }
    fn advance(&mut self, u: usize) {
        let d4 = rd(self.unit(u), 0xd4);
        let next = rd(&self.c[self.node(d4)], 0);
        let n = self.node(next);
        let (data, metric) = (rd(&self.c[n], 8), self.c[n][0xc]);
        let w = self.unit(u);
        wr(w, 0xd4, next);
        wr(w, 0xcc, data);
        w[0xd0] = metric;
    }
    fn at_head(&mut self, u: usize) -> bool {
        let w = self.unit(u);
        rd(w, 0xd4) == rd(w, 0xdc)
    }
    fn group_record(&mut self, o: usize) -> usize {
        self.log.push(format!("s2:c{o}+0x0"));
        self.cur = o;
        o
    }
    fn is_leader(&mut self, u: usize, g: usize) -> bool {
        let w = self.unit(u);
        let (id, player) = ((rd(w, 8) >> 16) as i16 as i32, w[9]);
        rd(&self.c[g], 0xd4) == id && rd(&self.c[g], 0xd8) == i32::from(player)
    }
    fn kind(&mut self, g: usize) -> i32 {
        self.log.push(format!("s3:c{g}+0x84"));
        rd(&self.c[g], 0x94)
    }
    fn get_obj(&mut self, kind: i32) -> usize {
        let o = self.cur;
        self.log.push(format!("s6:c{o}+0x84,{kind}"));
        o
    }
    fn assign_attack(&mut self, n: usize, g: usize) {
        let base = self.order_base(n);
        let _ = g;
        self.attack = true;
        let new_obj = base + 0x100;
        self.log.push(format!("s4:{}", Self::tok(new_obj)));
        let p6 = if self.rd(new_obj + 0x7c) != 0 {
            new_obj + 0x80
        } else {
            0
        };
        let nb = self.vbase(p6);
        let ob = base + 0x84;
        let old = self.s40(ob);
        let new = self.s40(nb);
        self.log
            .push(format!("s7:{},{}", Self::tok(new), Self::tok(old)));
    }
    fn assign_move(&mut self, n: usize, g: usize) {
        let base = self.order_base(n);
        let _ = g;
        self.attack = false;
        let q = self.s40(base + 0x100);
        let old = self.s40(base + 0x84);
        self.log
            .push(format!("s7:{},{}", Self::tok(q), Self::tok(old)));
    }
    fn finish(&mut self, n: usize, leader: bool) {
        let base = self.order_base(n);
        let obj = if self.attack {
            base + 0x180
        } else {
            base + 0x130
        };
        let nb = self.vbase(obj);
        if !leader {
            let (w, o) = self.at(nb + 4);
            w[o] &= 0xfe;
        }
        self.wr(obj + 0x10, 0);
        let x = self.rd(obj + 4);
        self.wr(obj + 0x44, x);
        let y = self.rd(obj + 8);
        self.wr(obj + 0x48, y);
    }
    fn remove_current(&mut self, u: usize) {
        self.listu = u;
        self.log.push(format!("s8:{}", Self::list(u)));
    }
    fn give_obj(&mut self, o: usize) {
        // `ecx` is the list the last call left it on.
        let u = self.listu;
        self.log.push(format!("s9:{},c{o}+0x0", Self::list(u)));
    }
    fn add(&mut self, u: usize, n: usize) {
        let base = self.order_base(n);
        let obj = if self.attack {
            base + 0x180
        } else {
            base + 0x130
        };
        let nb = self.vbase(obj);
        self.log
            .push(format!("s10:{},{}", Self::list(u), Self::tok(nb)));
    }
    fn head_next(&mut self, u: usize) {
        let head = rd(self.unit(u), 0xdc);
        let next = rd(&self.c[self.node(head)], 0);
        wr(self.unit(u), 0xdc, next);
    }
    fn kill_current_path(&mut self, u: usize) {
        self.log.push(format!("s11:{}+0x0", Self::uname(u)));
    }
}

/// `Unit::ungroup_move_order@005fd140`, run by `tools/emu/sweep_unit_ungroup_move_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: none; the global cell 0x00c0aec0 points at 64 random bytes; 256 random bytes at 0x00e91000; 256 random bytes at 0x00e91200; 16 random bytes at 0x00e90000; 16 random bytes at 0x00e90010; 16 random bytes at 0x00e90020; 16 random bytes at 0x00e90030; 16 random bytes at 0x00e90040; 16 random bytes at 0x00e90050; 768 random bytes at 0x00e92000; 768 random bytes at 0x00e92400; 768 random bytes at 0x00e92800; 768 random bytes at 0x00e92c00; 768 random bytes at 0x00e93000; 768 random bytes at 0x00e93400; vtable slot +0xe8 (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; vtable slot +0x98 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x44 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; callee 0x00730420 (1 args) stubbed; callee 0x0046d620 (0 args) stubbed; callee 0x00730bb0 (1 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e31d0 (0 args) stubbed; slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x8c`; stub 1 answers from its own `ecx`: the word at `+0x20`, or `ecx+0x20` when the word at `+0x1c` is set; stub 2 answers from its own `ecx`: the word at `+0x80`, or `ecx+0x80` when the word at `+0x78` is set; stub 3 answers from its own `ecx`: the word at `+0x10`; stub 4 answers from its own `ecx`: the word at `+0x80`, or `ecx+0x80` when the word at `+0x7c` is set; stub 5 answers from its own `ecx`: the word at `+0x30`, or `ecx+0x30` when the word at `+0x2c` is set; stub 6 answers from its own `ecx`: the word at `+0x7c`, or `ecx+0x7c` when the word at `+0x78` is set; a circular list in `c3..c4` at 0xe90000+0x10k, next `+0x0`, prev `+0x4`, head the word at `m+0xdc`; a circular list in `c5..c6` at 0xe90020+0x10k, next `+0x0`, prev `+0x4`, head the word at `c1+0xdc`; a circular list in `c7..c8` at 0xe90040+0x10k, next `+0x0`, prev `+0x4`, head the word at `c2+0xdc`; the stubbed vtable laid at `c1+0` as well as at `this`; the stubbed vtable laid at `c2+0` as well as at `this`; the stubbed vtable laid at `c9+0` as well as at `this`; the stubbed vtable laid at `c9+0x84` as well as at `this`; the stubbed vtable laid at `c9+0x100` as well as at `this`; the stubbed vtable laid at `c9+0x184` as well as at `this`; the stubbed vtable laid at `c10+0` as well as at `this`; the stubbed vtable laid at `c10+0x84` as well as at `this`; the stubbed vtable laid at `c10+0x100` as well as at `this`; the stubbed vtable laid at `c10+0x184` as well as at `this`; the stubbed vtable laid at `c11+0` as well as at `this`; the stubbed vtable laid at `c11+0x84` as well as at `this`; the stubbed vtable laid at `c11+0x100` as well as at `this`; the stubbed vtable laid at `c11+0x184` as well as at `this`; the stubbed vtable laid at `c12+0` as well as at `this`; the stubbed vtable laid at `c12+0x84` as well as at `this`; the stubbed vtable laid at `c12+0x100` as well as at `this`; the stubbed vtable laid at `c12+0x184` as well as at `this`; the stubbed vtable laid at `c13+0` as well as at `this`; the stubbed vtable laid at `c13+0x84` as well as at `this`; the stubbed vtable laid at `c13+0x100` as well as at `this`; the stubbed vtable laid at `c13+0x184` as well as at `this`; the stubbed vtable laid at `c14+0` as well as at `this`; the stubbed vtable laid at `c14+0x84` as well as at `this`; the stubbed vtable laid at `c14+0x100` as well as at `this`; the stubbed vtable laid at `c14+0x184` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_ungroup_move_order.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(15);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let mut w = Win {
                    m,
                    c,
                    log: &mut log,
                    real_captain: a[a.len() - 1] == 1,
                    cur: 0,
                    attack: false,
                    listu: 0,
                };
                crate::orders::ungroup_move_order_steps(&mut w, 0usize, i64::from(a[0]), a[1] != 0);
            }
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "ungroup_move_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "ungroup_move_order — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "ungroup_move_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[16 + i],
                "ungroup_move_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::ungroup_move_order@005fd140: {n} rows agree");
}
