//! `SpellType::cast_transport@00670db0` under the emulator against the port (item 3330; tools/sweep_gen.py, obj with windows).

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

/// `SpellType::cast_transport@00670db0`, run by `tools/emu/sweep_spell_type_cast_transport.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), `p` windows in order: p0..p27 the place(s), in flag order, p28 stub 22's answer window; the global cell 0x00c0618c points at 384 random bytes; the global cell 0x00c06210 points at 704 random bytes; the global cell 0x00c061f0 points at 3072 random bytes; the global cell 0x00c061fc points at 64 random bytes; the global cell 0x00e8d444 points at 5120 random bytes; the global cell 0x00cae5fc points at 16384 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; vtable slot +0x70 (0 args) stubbed; vtable slot +0x48 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x15c (1 args) stubbed; callee 0x006055c0 (0 args) stubbed; callee 0x005f9400 (2 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x0046d620 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x0046d9a0 (0 args) stubbed; callee 0x0046e8c0 (1 args) stubbed; callee 0x0046d870 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; vtable slot +0x10 (4 args) stubbed; vtable slot +0x18 (3 args) stubbed; callee 0x00643b70 (2 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; the word at `0xc0aec0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `p0+12` points at 256 random bytes less `0` (placed); the word at `0xe3a2a0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 16 random bytes less `0` (placed); the word at `p5+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed); the word at `p5+12` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `c3+0x10` points at 1400 random bytes less `0` (placed); the word at `p1+0x18` points at 768 random bytes less `0` (placed); the word at `p2+0x18` points at 768 random bytes less `0` (placed); the word at `p3+0x18` points at 768 random bytes less `0` (placed); the word at `p4+0x18` points at 768 random bytes less `0` (placed); the word at `p1+0xb8` points at 256 random bytes less `0` (placed); the word at `p2+0xb8` points at 256 random bytes less `0` (placed); the word at `p3+0xb8` points at 256 random bytes less `0` (placed); the word at `p4+0xb8` points at 256 random bytes less `0` (placed); the word at `p1+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p2+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p3+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p4+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p20+4` points at 16 random bytes less `0` (placed); the word at `p21+4` points at 16 random bytes less `0` (placed); the word at `p22+4` points at 16 random bytes less `0` (placed); the word at `p23+4` points at 16 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; slot +0x70 laid as 0x00662670 or its stub, a flag after the answers; stub 0 answered from [1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 1, 1, 1, 1, 1, 0] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 0, 0, 1] (90%); stub 5 answered from [0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 9 answered from [0, 1, 1] (90%); stub 10 answered from [0, 0, 0, 1] (90%); stub 11 answered from [0, 0, 1] (90%); stub 23 answered from [0, 1, 1, 2] (90%); stub 26 answered from [0, 0, 1] (90%); stub 22 answers the address of 16 random bytes, a `p` window after the places; the word at `p10+0` laid as the address of `p1` every row; the word at `p10+4` laid as the address of `p2` every row; the word at `p10+8` laid as the address of `p3` every row; the word at `p10+12` laid as the address of `p4` every row; stub 2 answers 2 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 23 answers 2 values in call order, printed after the answers; stub 4 answers -1..318 step 0x13f every row; stub 7 answers -1..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 23 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 23 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 24 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); stub 5 writes a value in 0..250000 through its argument 2, printed after the answers (`o` in a place); stub 5 writes a value in 0..250000 through its argument 3, printed after the answers (`o` in a place); stub 17 writes 0 (4 bytes) at `p4+0xd8` each call; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `c4+0` as well as at `this`; the stubbed vtable laid at `c4+0x9f0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_transport.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**, `crate::cast_transport::cast_transport_steps`, which
                // `Sim::cast_transport` runs over its `SimTransport`; here the view is the emulated original's windows.
                //
                // `a`: `g` (always the fourth unit) and `who`, then one answer a stub (`a[2 + k]`), the two words
                // `find_nearby_spot` writes through its arguments 2 and 3 (the spot, `a[33]`, `a[34]`), the call-order
                // answers of stubs 2, 3 and 23 (two each, from `a[35]`), and the three vtable flags (`a[41]`: slot
                // +0xb8 laid as `ObjectData::is`, `a[42]`: slot +0xd0 as `UnitData::is_caravan`, `a[43]`: slot +0x70 as
                // `SubObjectOut::is_on_screen`).
                // Stubs: 0 and 1 the object's vslots +0x8 and +0xbc, 2 the type's +0x130, 3 the unit's +0xd0, 4
                // `current_upgrade`, 5 `find_nearby_spot`, 6 the sound, 7 `init_unit`, 8 the unit's +0x70, 9 its +0x48,
                // 10 the unit's `is`, 11 the type's `is` (+0x60), 12 the boat's +0x15c, 13 `update_speed`, 14
                // `same_damage`, 15 `set_new_location`, 16 `set_angle`, 17 `remove_current`, 18 `add`, 19
                // `kill_current_order`, 20 `invert`, 21 `increase_size`, 22 `pop`, 23 `get_tregion`, 24 `push`, 25
                // `clear_orders`, 26 `GroupData::member`, 27 and 28 the group's +0x10 and +0x18, 29 `replace_hotunit`,
                // 30 `go_inside`.
                // Windows: `c0` the objects singleton, `c1` the console (`+0x298` who, `+0x2a0` play), `c2` the
                // constants (`+0x98` the board distance), `c3` the unit types' cell (its `+0x10` points at `p11`),
                // `c4` the selection groups, `c5` `div_3_table`; `p0` the units' array, `p1..p4` the four units, `p5`
                // the caravans' list and `p6..p9` its records, `p10` the objects' list (the units again), `p12..p15`
                // the units' types, `p16..p19` their path stacks' buffers, `p20..p23` their order lists' tails and
                // `p24..p27` the nodes, `p28` what `pop` answers.
                use crate::Pos;
                use crate::cast_transport::{TransportView, cast_transport_steps};
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 3],
                    tz: Vec<(i32, i32)>,
                    pushed: Vec<String>,
                    find_ecx: i32,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn hx(v: i32) -> String {
                    v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
                }
                impl V<'_> {
                    fn who(&self) -> i32 {
                        self.a[1]
                    }
                    fn unit(&self, o: i32) -> &[u8] {
                        &self.p[1 + o as usize]
                    }
                    fn unit_mut(&mut self, o: i32) -> &mut Vec<u8> {
                        &mut self.p[1 + o as usize]
                    }
                    /// Stub `slot`'s next of two answers from `a[base]`, the last repeated.
                    fn seq(&mut self, slot: usize, base: usize) -> i32 {
                        let i = self.turn[slot].min(1);
                        self.turn[slot] += 1;
                        self.a[base + i]
                    }
                    fn leader(&self) -> i32 {
                        14918544 + 0x6eec * self.a[1]
                    }
                    fn group(&self) -> String {
                        format!("c4+{:#x}", rd32(&self.c[1], 0x2a0) * 0x9f0)
                    }
                    fn xy(&self, o: i32) -> (i32, i32) {
                        (
                            rd32(self.unit(o), 0x10) ^ 0x63637,
                            rd32(self.unit(o), 0x14) ^ 0x63637,
                        )
                    }
                }
                impl TransportView for V<'_> {
                    fn alive(&mut self, g: i32) -> bool {
                        self.log.push(format!("s0:p{}+0x0", 1 + g));
                        self.a[2] != 0
                    }
                    fn on_map(&mut self, g: i32) -> bool {
                        self.log.push(format!("s1:p{}+0x0", 1 + g));
                        self.a[3] != 0
                    }
                    fn domain(&self, g: i32) -> i32 {
                        rd32(&self.p[12 + g as usize], 0x218)
                    }
                    fn is_caravan(&mut self, g: i32) -> bool {
                        if self.a[42] == 0 {
                            self.log.push(format!("s3:p{}+0x0", 1 + g));
                            self.seq(1, 37) != 0
                        } else {
                            self.log.push(format!("s2:p{}+0x0", 12 + g));
                            self.seq(0, 35) != 0
                        }
                    }
                    fn current_upgrade(&mut self, base: i32) -> i32 {
                        self.log.push(format!("s4:{},{base}", self.leader()));
                        self.a[6]
                    }
                    fn find_spot(&mut self, g: i32, ty: i32) -> Option<Pos> {
                        let (x, y) = self.xy(g);
                        self.find_ecx = rd32(&self.p[11], 4 * ty as usize);
                        self.log.push(format!(
                            "s5:{},{x},{y},sp-0x38,sp-0x3c,0,{},0,1431655765,3,-1,-1,0,0,-1,0,-1",
                            self.find_ecx,
                            rd32(&self.c[2], 0x98)
                        ));
                        (self.a[7] == 0).then(|| Pos::new(self.a[33], self.a[34]))
                    }
                    fn init_unit(&mut self, who: i32, ty: i32, g: i32) -> i32 {
                        let (x, y) = self.xy(g);
                        self.log
                            .push(format!("s7:c0+0x0,{who},{ty},{x},{y},-1,-1,-1"));
                        self.a[9]
                    }
                    fn console_who(&self) -> i32 {
                        rd32(&self.c[1], 0x298)
                    }
                    fn on_screen(&mut self, g: i32) -> bool {
                        if self.a[43] == 0 {
                            self.log.push(format!("s8:p{}+0x0", 1 + g));
                            self.a[10] != 0
                        } else {
                            self.unit(g)[0x1c] != 0
                        }
                    }
                    fn console_sees(&mut self, g: i32) -> bool {
                        let who = self.console_who();
                        self.log.push(format!("s9:p{}+0x0,{who},0", 1 + g));
                        self.a[11] != 0
                    }
                    fn sound(&mut self, id: i32) {
                        let ecx = if id == 0x4e {
                            self.find_ecx.to_string()
                        } else {
                            format!("p{}+0x0", 1 + self.a[0])
                        };
                        self.log.push(format!("s6:{ecx},{id}"));
                    }
                    fn is(&mut self, g: i32, ty: i32) -> bool {
                        if self.a[41] == 0 {
                            self.log.push(format!("s10:p{}+0x0,{ty},0", 1 + g));
                            self.a[12] != 0
                        } else {
                            self.log.push(format!("s11:p{}+0x0,{ty},0", 12 + g));
                            self.a[13] != 0
                        }
                    }
                    fn marines(&mut self, b: i32) {
                        let w = self.unit_mut(b);
                        let f = rd32(w, 0x6c);
                        wr32(w, 0x6c, f | 0x200);
                        self.log.push(format!("s12:p{}+0x0,0", 1 + b));
                        self.log.push(format!("s13:p{}+0x0", 1 + b));
                    }
                    fn same_damage(&mut self, b: i32, g: i32, who: i32) {
                        self.log.push(format!("s14:p{}+0x0,{g},{who}", 1 + b));
                    }
                    fn set_new_location(&mut self, _g: i32, b: i32, to: Pos) {
                        self.log
                            .push(format!("s15:p{}+0x0,{},{},1,1", 1 + b, to.x, to.y));
                    }
                    fn heading(&self, g: i32) -> i32 {
                        rd32(self.unit(g), 0x50)
                    }
                    fn set_angle(&mut self, b: i32, angle: i32) {
                        self.log
                            .push(format!("s16:p{}+0x0,{angle},p0+0x0,1", 1 + b));
                    }
                    fn caravan_slot(&self, o: i32) -> i32 {
                        i32::from(i16::from_le_bytes([self.unit(o)[0x86], self.unit(o)[0x87]]))
                    }
                    fn set_caravan_slot(&mut self, o: i32, s: i32) {
                        self.unit_mut(o)[0x86..0x88].copy_from_slice(&(s as i16).to_le_bytes());
                    }
                    fn set_caravan_unit(&mut self, _who: i32, s: i32, b: i32) {
                        self.p[6 + s as usize][0xa..0xc].copy_from_slice(&(b as i16).to_le_bytes());
                    }
                    fn has_200(&self, o: i32) -> bool {
                        rd32(self.unit(o), 0x68) & 0x200 != 0
                    }
                    fn set_200(&mut self, o: i32, on: bool) {
                        let w = self.unit_mut(o);
                        let f = rd32(w, 0x68);
                        wr32(w, 0x68, if on { f | 0x200 } else { f & !0x200 });
                    }
                    fn orders_left(&self, g: i32) -> i32 {
                        rd32(self.unit(g), 0xd8)
                    }
                    fn orders_tail(&self, g: i32) -> bool {
                        rd32(self.unit(g), 0xdc) != 0
                    }
                    fn orders_sync(&mut self, g: i32) {
                        let node = rd32(&self.p[20 + g as usize], 4);
                        let (word, byte) = (
                            rd32(&self.p[24 + g as usize], 8),
                            self.p[24 + g as usize][0xc],
                        );
                        let w = self.unit_mut(g);
                        wr32(w, 0xd4, node);
                        wr32(w, 0xcc, word);
                        w[0xd0] = byte;
                    }
                    fn order_head(&mut self, g: i32) -> i32 {
                        rd32(self.unit(g), 0xcc)
                    }
                    fn order_remove(&mut self, g: i32) {
                        self.log.push(format!("s17:p{}+0xcc", 1 + g));
                        // The stub unlinks the head its caller loops over.
                        wr32(self.unit_mut(g), 0xd8, 0);
                    }
                    fn order_add(&mut self, b: i32, order: i32) {
                        self.log.push(format!("s18:p{}+0xcc,{order}", 1 + b));
                    }
                    fn kill_current_order(&mut self, b: i32) {
                        self.log.push(format!("s19:p{}+0x0,0", 1 + b));
                    }
                    fn path_invert(&mut self, g: i32) {
                        self.log.push(format!("s20:p{}+0xb8", 1 + g));
                    }
                    fn path_len(&self, o: i32) -> i32 {
                        rd32(self.unit(o), 0xc0)
                    }
                    fn path_take(&mut self, g: i32) -> [i32; 4] {
                        let w = self.unit_mut(g);
                        let mut n = rd32(w, 0xc0);
                        if n < 1 {
                            n = 1;
                        }
                        n -= 1;
                        wr32(w, 0xc0, n);
                        let buf = &self.p[16 + g as usize];
                        let at = 16 * n as usize;
                        [
                            rd32(buf, at),
                            rd32(buf, at + 4),
                            rd32(buf, at + 8),
                            rd32(buf, at + 12),
                        ]
                    }
                    fn path_put(&mut self, b: i32, pd: [i32; 4]) {
                        let (n, cap, grow) = {
                            let w = self.unit(b);
                            (rd32(w, 0xc0), rd32(w, 0xbc), w[0xc4] as i8)
                        };
                        if n >= cap {
                            self.log.push(format!("s21:p{}+0xb8,{grow}", 1 + b));
                        }
                        let at = 16 * n as usize;
                        let buf = &mut self.p[16 + b as usize];
                        for (k, v) in pd.iter().enumerate() {
                            wr32(buf, at + 4 * k, *v);
                        }
                        wr32(self.unit_mut(b), 0xc0, n + 1);
                    }
                    fn mark_auto_transport(&mut self, b: i32) {
                        let w = self.unit_mut(b);
                        let f = rd32(w, 0x68);
                        wr32(w, 0x68, f | 0x800000);
                    }
                    fn path_pop(&mut self, b: i32) -> [i32; 4] {
                        self.log.push(format!("s22:p{}+0xb8", 1 + b));
                        let w = &self.p[28];
                        [rd32(w, 0), rd32(w, 4), rd32(w, 8), rd32(w, 12)]
                    }
                    fn div3(&self, k: i32) -> i32 {
                        rd32(&self.c[5], 4 * k as usize)
                    }
                    fn tregion(&mut self, x: i32, y: i32) -> i32 {
                        let (first, second) = ("sp-0x24,sp-0x28", "sp-0x30,sp-0x2c");
                        let args = if self.tz.is_empty() { first } else { second };
                        self.log.push(format!("s23:12621800,{args}"));
                        self.tz.push((x, y));
                        self.seq(2, 39)
                    }
                    fn path_push(&mut self, b: i32, pd: [i32; 4]) {
                        self.log.push(format!("s24:p{}+0xb8,sp-0x20", 1 + b));
                        self.pushed.push(pd.iter().map(|v| hx(*v)).collect());
                    }
                    fn clear_orders(&mut self, g: i32) {
                        self.log.push(format!("s25:p{}+0x0", 1 + g));
                    }
                    fn member(&mut self, g: i32, who: i32) -> bool {
                        self.log.push(format!("s26:{},{g},{who},1", self.group()));
                        self.a[28] != 0
                    }
                    fn group_swap(&mut self, g: i32, b: i32, who: i32) {
                        self.log.push(format!("s27:{},{g},{who},0,0", self.group()));
                        self.log.push(format!("s28:{},{b},{who},0", self.group()));
                    }
                    fn replace_hotunit(&mut self, g: i32, b: i32, who: i32) {
                        self.log.push(format!("s29:p{}+0x0,{b},{who}", 1 + g));
                    }
                    fn go_inside(&mut self, g: i32, b: i32, who: i32) {
                        self.log.push(format!("s30:p{}+0x0,{b},{who},0", 1 + g));
                    }
                }
                let g = a[0];
                let who = a[1];
                let mut v = V {
                    a,
                    c,
                    p,
                    log: &mut log,
                    turn: [0; 3],
                    tz: Vec::new(),
                    pushed: Vec::new(),
                    find_ecx: 0,
                };
                let _ = v.who();
                cast_transport_steps(&mut v, g, who);
                let join = |v: Vec<String>| {
                    if v.is_empty() {
                        "-".to_string()
                    } else {
                        v.join("/")
                    }
                };
                let d0 = join(v.tz.iter().map(|t| hx(t.0)).collect());
                let d1 = join(v.tz.iter().map(|t| hx(t.1)).collect());
                let dp = join(std::mem::take(&mut v.pushed));
                v.log.extend([d0, d1, dp]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cast_transport — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_transport — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cast_transport — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "cast_transport — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("SpellType::cast_transport@00670db0: {n} rows agree");
}
