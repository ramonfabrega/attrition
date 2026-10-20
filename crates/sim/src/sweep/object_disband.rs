//! `Object::disband@006455c0` under the emulator against the port (item 3346; tools/sweep_gen.py, obj with windows).

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

use crate::disband::{DisbandView, disband_steps};

// **Held out of the draw, by name** (item 3346). The garrison recursion
// (`disband(inside, 0)` on `+0x28`'s chain): `Harness` hooks a callee by its
// address and this function's own entry is the sweep's entry, so the self-call
// cannot be stubbed; `this+0x28` is held negative and the loop never runs.
// The Lakota message (strings, a text bubble, a sound): `console->who` is
// held away from `who`, so the arm is never entered and a port that enters
// it logs `message`, which the original never prints. `Random::get` is
// stubbed to `0..=65535`, so `r % 3` is never negative.

/// The image's `game_random` cell, as `Random::get`'s `ecx` prints it.
const GAME_RANDOM: i32 = 14_908_044;

/// The windows: the leaders' rows `p0..p15` (two slices a row: flags, then the
/// goods pointer), the type records `p17..p20`, the units `p22..p24` (who 0)
/// and `p26..p28` (who 1), the goods arrays `p29`, `p30`, the data `p31`, the
/// original-type record `p32`, the gate record `p33`, the get_action orders
/// `p34..p36`, the update_action orders `p37..p39` and their targets
/// `p40..p42`; `c0` the objects, `c1` the constants, `c3` the console, `c4`
/// `last_razed`.
const TYPES: usize = 17;
const UNITS: [usize; 2] = [22, 26];
const GOODS: [usize; 2] = [29, 30];
const DATA: usize = 31;
const ORIG: usize = 32;
const ACTIONS: usize = 34;
const TARGETS: usize = 40;

/// Stub answers: `a[0]` the argument, then one a stub in the line's order.
const fn ans(stub: usize) -> usize {
    1 + stub
}

/// The `--seq` runs follow the 26 stubs: price (6), has_tribe_bonus (2),
/// type_avail (6); then the three null flags of `get_action`'s windows.
const PRICES: usize = 27;
const TRIBES: usize = 33;
const AVAILS: usize = 35;
const NULLS: usize = 41;

struct Emu<'a> {
    a: &'a [i32],
    m: &'a [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    actions: usize,
    updates: usize,
    prices: usize,
    tribes: usize,
    avails: usize,
}

impl Emu<'_> {
    fn unit_win(&self, i: i32) -> usize {
        UNITS[usize::from(self.m[9])] + i as usize
    }
    fn me_win(&self) -> usize {
        self.unit_win(i32::from(i16::from_le_bytes([self.m[10], self.m[11]])))
    }
}

impl DisbandView for Emu<'_> {
    type Order = usize;
    fn who(&mut self) -> i32 {
        i32::from(self.m[9])
    }
    fn index(&mut self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[10], self.m[11]]))
    }
    fn in_use(&mut self) -> bool {
        self.a[ans(0)] != 0
    }
    fn active(&mut self) -> bool {
        self.a[ans(1)] != 0
    }
    fn started(&mut self) -> bool {
        self.a[ans(5)] != 0
    }
    fn is_structure(&mut self) -> bool {
        self.a[ans(2)] != 0
    }
    fn is_unit(&mut self) -> bool {
        self.a[ans(3)] != 0
    }
    fn is_captain(&mut self) -> bool {
        self.a[ans(4)] != 0
    }
    fn is_plain_building(&mut self) -> bool {
        self.a[ans(6)] != 0
    }
    fn on_map(&mut self) -> bool {
        self.a[ans(7)] != 0
    }
    fn builders_gate(&mut self) -> bool {
        self.a[ans(14)] != 0
    }
    fn unit_count(&mut self) -> i32 {
        rd(&self.c[0], 0x15c + 4 * usize::from(self.m[9]))
    }
    fn unit_in_use(&mut self, i: i32) -> bool {
        self.p[self.unit_win(i)][8] & 1 != 0
    }
    fn unit_type(&mut self, i: i32) -> i32 {
        rd(&self.p[self.unit_win(i)], 0x94)
    }
    fn unit_action(&mut self, i: i32) -> Option<usize> {
        let w = self.unit_win(i);
        self.log.push(format!("s18:p{w}+0x0"));
        let k = self.actions.min(2);
        self.actions += 1;
        (self.a[NULLS + k] == 0).then_some(ACTIONS + k)
    }
    fn order_kind(&mut self, _o: usize) -> i32 {
        self.a[ans(15)]
    }
    fn updated_order_target(&mut self, i: i32) -> (i32, i32) {
        let w = self.unit_win(i);
        self.log.push(format!("s19:p{w}+0x0"));
        let e = TARGETS + self.updates.min(2);
        self.updates += 1;
        (rd(&self.p[e], 8), rd(&self.p[e], 0xc))
    }
    fn unit_clear_orders(&mut self, i: i32) {
        let w = self.unit_win(i);
        self.log.push(format!("s20:p{w}+0x0"));
    }
    fn has_tribe_bonus(&mut self, who: i32, n: i32) -> bool {
        self.log.push(format!("s21:p{}+0x0,{n}", 2 * who));
        let v = self.a[TRIBES + self.tribes.min(1)];
        self.tribes += 1;
        v != 0
    }
    fn lakota_raze_price(&mut self) -> i32 {
        rd(&self.c[1], 0x854)
    }
    fn console_who(&mut self) -> i32 {
        rd(&self.c[3], 0x298)
    }
    fn time(&mut self) -> u32 {
        let who = self.who();
        self.log.push(format!("s12:p{DATA}+0x0,{who}"));
        self.a[ans(12)] as u32
    }
    fn job_counter(&mut self) -> u32 {
        rd(&self.p[DATA], 0x48) as u32
    }
    fn construct_time(&mut self) -> i32 {
        self.log.push(format!("s13:p{DATA}+0x0,0"));
        self.a[ans(13)]
    }
    fn job_counter_2(&mut self) -> i32 {
        rd(&self.p[DATA], 0x4c)
    }
    fn unit_flags68(&mut self) -> i32 {
        rd(&self.p[self.me_win()], 0x68)
    }
    fn unit_killer(&mut self) -> i32 {
        i32::from(self.p[self.me_win()][0xa9] as i8)
    }
    fn unit_killed_obj(&mut self) -> i32 {
        let u = &self.p[self.me_win()];
        i32::from(i16::from_le_bytes([u[0xa4], u[0xa5]]))
    }
    fn leader_flags(&mut self, who: i32) -> i32 {
        rd(&self.p[2 * who as usize], 0)
    }
    fn plunder(&mut self, a: i32, killer: i32, c: i32, d: i32) {
        let w = self.me_win();
        self.log.push(format!("s22:p{w}+0x0,{a},{killer},{c},{d}"));
    }
    fn set_last_razed(&mut self, who: i32, index: i32) {
        wr(&mut self.c[4], 4 * who as usize, index);
    }
    fn num_inside(&mut self) -> i32 {
        self.log.push("s23:m+0x0,0".to_string());
        self.a[ans(23)]
    }
    fn inside_head(&mut self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0x28], self.m[0x29]]))
    }
    fn disband_inside(&mut self, _s: i32) {
        panic!("the garrison chain is held negative: the recursion is not swept");
    }
    fn random(&mut self, lo: i32, hi: i32) -> i32 {
        self.log.push(format!("s24:{GAME_RANDOM},{lo},{hi}"));
        self.a[ans(24)]
    }
    fn die(&mut self, a: i32, b: i32, c: i32) {
        self.log.push(format!("s9:m+0x0,{a},{b},{c}"));
    }
    fn close(&mut self, a: i32, b: i32, c: i32) {
        self.log.push(format!("s8:m+0x0,{a},{b},{c}"));
    }
    fn type_avail(&mut self, who: i32, good: i32, strict: i32) -> bool {
        self.log
            .push(format!("s25:p{}+0x0,{good},{strict}", 2 * who));
        let v = self.a[AVAILS + self.avails.min(5)];
        self.avails += 1;
        v != 0
    }
    fn orig_type(&mut self) -> i32 {
        rd(&self.p[ORIG], 0x6c)
    }
    fn price(&mut self, args: [i32; 7]) -> i32 {
        let ty = rd(self.m, 0x44) as usize;
        let list: Vec<String> = args.iter().map(i32::to_string).collect();
        self.log
            .push(format!("s17:p{}+0x0,{}", TYPES + ty, list.join(",")));
        let v = self.a[PRICES + self.prices.min(5)];
        self.prices += 1;
        v
    }
    fn add_good(&mut self, who: i32, good: i32, amount: i32) {
        let w = &mut self.p[GOODS[who as usize]];
        let at = 4 * good as usize;
        let word = rd(w, at);
        wr(w, at, ((word ^ 0x8221).wrapping_add(amount)) ^ 0x8221);
    }
    fn message_good(&mut self, _good: i32, _amount: i32, _first: bool) {
        self.log.push("message".to_string());
    }
    fn message_end(&mut self) {
        self.log.push("message".to_string());
    }
}

/// `Object::disband@006455c0`, run by `tools/emu/sweep_object_disband.py` under unicorn
/// (tools/emu): 96 random bytes of object, 1 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p33 the place(s), in flag order, p34..p42 the stubs' answer window(s); the global cell 0x00c0618c points at 384 random bytes; the global cell 0x00c061f0 points at 2144 random bytes; the global cell 0x00e85ddc points at 32 random bytes; the global cell 0x00c06210 points at 672 random bytes; 8 random bytes at 0x00cc2190; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x150 (3 args) stubbed; vtable slot +0x158 (3 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xa4 (1 args) stubbed; vtable slot +0x18c (1 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x6c (0 args) stubbed; vtable slot +0x78 (7 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x00605b00 (4 args) stubbed; callee 0x00646d50 (1 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x006e33a0 (2 args) stubbed; the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c2+0` points at 16 random bytes less `0` (placed); the word at `c2+4` points at 16 random bytes less `0` (placed); the word at `c2+8` points at 16 random bytes less `0` (placed); the word at `c2+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0` points at 12 random bytes less `0` (placed); the word at `p21+0` points at 176 random bytes less `0` (placed); the word at `p21+4` points at 176 random bytes less `0` (placed); the word at `p21+8` points at 176 random bytes less `0` (placed); the word at `0xc0aedc` points at 12 random bytes less `0` (placed); the word at `p25+0` points at 176 random bytes less `0` (placed); the word at `p25+4` points at 176 random bytes less `0` (placed); the word at `p25+8` points at 176 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 24 random bytes less `0` (placed); stub 10's answer points at 128 random bytes less `0` (placed); stub 11's answer points at 128 random bytes less `0` (placed); the word at `p31+0x18` points at 64 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 0, 0] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 12 answered from [0, 1, 7, 1000, 12345, 100000, 1000000, 2000000] (90%); stub 13 answered from [0, 1, 7, 60, 1000, 12345, 777777, 2000000] (90%); stub 14 answered from [1, 1, 1, 0] (90%); stub 15 answered from [7, 7, 7, 0, 6] (90%); stub 21 answered from [0, 0, 1] (90%); stub 23 answered from [0, 1] (90%); stub 25 answered from [1, 1, 1, 0] (90%); stub 18 answers a window of its own to each call, in call order (-64 bytes, ...); stub 19 answers a window of its own to each call, in call order (64 bytes, ...); stub 16 answers a window of its own to each call, in call order (64 bytes, ...); the word at `m+0x18` laid as the address of `m+0x40` every row; the word at `c0+0x30` laid as the address of `p16` every row; the word at `p16+0` laid as the address of `m` every row; the word at `p16+4` laid as the address of `m` every row; the word at `p16+8` laid as the address of `m` every row; the word at `p22+0x18` laid as the address of `p22+0x90` every row; the word at `p23+0x18` laid as the address of `p23+0x90` every row; the word at `p24+0x18` laid as the address of `p24+0x90` every row; the word at `p26+0x18` laid as the address of `p26+0x90` every row; the word at `p27+0x18` laid as the address of `p27+0x90` every row; the word at `p28+0x18` laid as the address of `p28+0x90` every row; stub 17 answers 6 values in call order, printed after the answers; stub 21 answers 2 values in call order, printed after the answers; stub 25 answers 6 values in call order, printed after the answers; stub 24 answers 0..65535 every row; stub 17 answers 0..20000 every row; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; the stubbed vtable laid at `p39+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_disband.py") else {
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
            let mut emu = Emu {
                a,
                m,
                c,
                p,
                log: &mut log,
                actions: 0,
                updates: 0,
                prices: 0,
                tribes: 0,
                avails: 0,
            };
            disband_steps(&mut emu, a[0]);
            want_eax
        };
        assert_eq!(got, want_eax);
        assert_eq!(log, want_log, "disband — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "disband — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "disband — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "disband — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Object::disband@006455c0: {n} rows agree");
}
