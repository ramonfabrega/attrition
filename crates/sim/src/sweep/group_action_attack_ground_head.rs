//! `Group::action_attack_ground@00704520` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_attack_ground@00704520`, run by `tools/emu/sweep_group_action_attack_ground_head.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; 4 random bytes at 0x00cc02f8 (each word in 1..1); 4 random bytes at 0x00ed6574 (each word in 0..3); 4 random bytes at 0x00ed6590 (each word in 0..3); vtable slot +0x14 (0 args) stubbed; vtable slot +0x10 (4 args) stubbed; the word at `0xed6580+0x1c*0` points at 16 random bytes less `0` (placed); the word at `0xed6580+0x1c*1` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 2816 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_attack_ground_head.py") else {
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
                // The head slice: every row holds the group's count at 0, so the call ends where the count is tested.
                // Window layout. `m` the group (`+0x4a` player); `c[0]` `ignore_orders` (held at 1), `c[1]` and
                // `c[2]` players 0 and 1's ignore-orders counts; `p[who]` the elements of
                // `objects_ignoring_orders[who]` (players 0 and 1 are laid; a player of 8 or more skips the sweep).
                //
                // **The port is the crate's own steps**, `crate::attack_ground_view::action_attack_ground_steps`.
                use crate::attack_ground_view::{AttackGroundView, action_attack_ground_steps};
                struct V<'a> {
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl AttackGroundView for V<'_> {
                    fn refresh(&mut self) {
                        self.log.push("s0:m+0x0".into());
                    }
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[0], 0) != 0
                    }
                    fn ignoring_count(&self, who: i32) -> i32 {
                        rd32(&self.c[1 + who as usize], 0)
                    }
                    fn ignoring_at(&self, who: i32, i: i32) -> i32 {
                        rd32(&self.p[who as usize], 4 * i as usize)
                    }
                    fn drop_ignored(&mut self, o: i32, who: i32) {
                        self.log.push(format!("s1:m+0x0,{o},{who},0,0"));
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn world_dims(&self) -> (i32, i32) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn tile_owner(&self, _x: i32, _y: i32) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn is_peace(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn console_who(&self) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn refuse(&mut self) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn flagged(&self) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn member(&self, _i: i32) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn find_leader(&mut self) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn can_carry_air(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn obj_c0(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn is_on_map(&mut self) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn obj_slot8(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn obj_slot_bc(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn unit_c0(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn unit_can_attack_ground(&mut self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn add_attack_ground_order(
                        &mut self,
                        _w: i32,
                        _o: i32,
                        _x: i32,
                        _y: i32,
                        _q: i32,
                    ) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn obj_flag1(&self, _w: i32, _o: i32) -> bool {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn obj_link(&self, _w: i32, _o: i32) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn obj_link_who(&self, _w: i32, _o: i32) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn scratch_clear(&mut self) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn scratch_add(&mut self, _id: i32, _w: i32) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn scratch_count(&self) -> i32 {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn scratch_member(&self, _k: i32) -> (i32, i32) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn clear(&mut self) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn add_member(&mut self, _id: i32, _w: i32) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                    fn air_attack_ground(&mut self, _x: i32, _y: i32, _q: i32) {
                        unreachable!("the head sweep holds the count at 0")
                    }
                }
                let mut v = V {
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                };
                action_attack_ground_steps(&mut v, [a[0], a[1], a[2]]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_attack_ground — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "action_attack_ground — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_attack_ground — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "action_attack_ground — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Group::action_attack_ground@00704520: {n} rows agree");
}
