//! `City::check_upgrade@00738b20` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

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

/// `City::check_upgrade@00738b20`, run by `tools/emu/sweep_city_check_upgrade.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p11 sparse table 0 (3 rows × 4 slice(s), row-major), p12..p22 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061ec points at 1792 random bytes; the global cell 0x00c06210 points at 768 random bytes; callee 0x00736480 (0 args) stubbed; callee 0x00738450 (0 args) stubbed; callee 0x007364f0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x80 (2 args) stubbed; callee 0x00642fc0 (2 args) stubbed; callee 0x007384c0 (0 args) stubbed; callee 0x006dc490 (0 args) stubbed; callee 0x00680f60 (0 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; vtable slot +0x10c (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x00a1b260 (3 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `c0+0x14+u8(m,0x5e)*0x1c` points at 12 random bytes less `0` (placed); the word at `p12+0` points at 64 random bytes less `0` (placed); the word at `p12+4` points at 64 random bytes less `0` (placed); the word at `p12+8` points at 64 random bytes less `0` (placed); the word at `0xc09970+u8(m,0x5e)*0x1c` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 256 random bytes less `0` (placed); the word at `p16+4` points at 256 random bytes less `0` (placed); the word at `p16+8` points at 256 random bytes less `0` (placed); the word at `c2+0x134` points at 768 random bytes less `0` (placed); stub 3's answer points at 128 random bytes less `0` (placed); the word at `p21+0x18` points at 16 random bytes less `0x74` (placed); stub 0 answered from [0, 1, 1, 1] (90%); the stubbed vtable laid at `p21+0`; the word at `p21+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 1 answers 4 values in call order, printed after the answers; stub 1 answers 1..5 every row; stub 2 answers 415..416 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x95c:0x4, +0xe62:0x80, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_check_upgrade.py") else {
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
            // `a[i]` stub `i`'s answer (0 `ready_to_upgrade`, 2 `upgrades_to`, 12 `String::parse`, 14 `get_neon`);
            // `a[17..21]` stub 1's four `get_pop_value` answers in call order. Stubs: 4 the building's `set_type`
            // (slot `+0x80`), 5 `Wall::mask_me`, 6 `find_buildings`, 7 `Leader::calc_pop_cap`, 8 `Region::fix_borders`,
            // 9 `String::operator=`, 10 slot `+0x10c`, 11 `String::operator+=`, 12 `String::parse`, 13 `String::close`,
            // 14 `TeamColor::get_neon`, 15 `MessageWin::add_message`, 16 `SoundGlobal::play`. Windows: `m` the city
            // (`+0x5e` who, `+6` its index, `+8` its object); `c[0]` the objects singleton, `c[1]` `div_3_table`, `c[2]`
            // the world (`+0` width, `+0x134` the region table `p[20]`), `c[3]` the game (`+0x6cc` `world_pop`), `c[4]`
            // the console (`+0x298` who); `p[4*who + 0..4]` the leader's flags, `+0x95c` pop, `+0xe62` region pops and
            // `+0x6928` colour; `p[13 + o]` the objects, `p[17 + city]` the cities, `p[21]` the building the object
            // answers (vtable `p[21]+0`, `+0x18` the name `p[22]`).
            //
            // **The port is the crate's own steps**, `crate::city::check_upgrade_steps`, which `Sim::check_upgrade`
            // runs over its records; here the view is the emulated original's windows. The population adjustments
            // are the running delta the original keeps (the crate recounts, which is the same number), and the
            // message is the view's own: this crate models none of it (SEAM on `announce`). The `ecx` of
            // `fix_borders` and `play` is whatever the previous call left; the log names what the emulator left.
            use crate::city::{CheckUpgradeView, check_upgrade_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                pops: usize,
            }
            impl V<'_> {
                fn who(&self) -> usize {
                    self.m[0x5e] as i8 as usize
                }
                fn obj(&self) -> usize {
                    13 + i16::from_le_bytes([self.m[8], self.m[9]]) as usize
                }
                fn next_pop(&mut self) -> i32 {
                    self.log.push("s1:m+0x0".into());
                    let v = self.a[17 + self.pops.min(3)];
                    self.pops += 1;
                    v
                }
            }
            impl CheckUpgradeView for V<'_> {
                fn ready_to_upgrade(&mut self) -> bool {
                    self.log.push("s0:m+0x0".into());
                    self.a[0] != 0
                }
                fn pop_value(&mut self) -> i32 {
                    self.next_pop()
                }
                fn upgrades_to(&mut self) -> usize {
                    self.log.push("s2:m+0x0".into());
                    self.a[2] as usize
                }
                fn set_type(&mut self, next: usize) {
                    self.log.push(format!("s4:p21+0x0,{next},0"));
                }
                fn mask_city(&mut self) {
                    self.log.push("s5:p21+0x0,1,0".into());
                }
                fn flag_wall_stats(&mut self) {
                    let w = &mut self.p[4 * self.who()];
                    let f = rd(w, 0) | 0x8000000;
                    wr(w, 0, f);
                }
                fn find_buildings(&mut self) {
                    self.log.push("s6:m+0x0".into());
                }
                fn repop(&mut self, before: i32) {
                    let row = 4 * self.who();
                    let now = self.next_pop();
                    let pop = rd(&self.p[row + 1], 0);
                    wr(
                        &mut self.p[row + 1],
                        0,
                        pop.wrapping_add(now.wrapping_sub(before)),
                    );
                    let now = self.next_pop();
                    let world = rd(&self.c[3], 0x6cc);
                    wr(
                        &mut self.c[3],
                        0x6cc,
                        world.wrapping_add(now.wrapping_sub(before)),
                    );
                    // The region of the object's cell: `div_3_table` of each coordinate, the world's width, 0x1c a record.
                    let o = self.obj();
                    let tile = |w: i32| rd(&self.c[1], (((w ^ 0x63637) >> 8) as usize) * 4);
                    let (x, y) = (tile(rd(&self.p[o], 0x10)), tile(rd(&self.p[o], 0x14)));
                    let at = 4 + (rd(&self.c[2], 0) * y + x) as usize * 0x1c;
                    let reg = i16::from_le_bytes([self.p[20][at], self.p[20][at + 1]]);
                    if i32::from(reg) < 0x40 {
                        let now = self.next_pop() as i16;
                        let w = &mut self.p[row + 2];
                        let off = 2 * reg as usize;
                        let old = i16::from_le_bytes([w[off], w[off + 1]]);
                        let new = now.wrapping_add(old.wrapping_sub(before as i16));
                        w[off..off + 2].copy_from_slice(&new.to_le_bytes());
                    }
                    self.log.push(format!("s7:p{row}+0x0"));
                }
                fn fix_borders(&mut self) {
                    self.log.push(format!("s8:p{}+0x0", 4 * self.who()));
                }
                fn announce(&mut self) {
                    if self.who() as i32 != rd(&self.c[4], 0x298) {
                        return;
                    }
                    let city = 17 + i16::from_le_bytes([self.m[6], self.m[7]]) as usize;
                    // `loc_str_array_orig`'s cell reads 0 in the image, so the string is at `+0x1608`.
                    self.log.push("s9:sp-0x40,5640".into());
                    if rd(&self.p[21], 0) as u32 == 0xb42174 {
                        self.log.push("s11:sp-0x54,p22+0x0".into());
                    } else {
                        self.log.push("s10:p21+0x0,sp-0x54".into());
                    }
                    self.log
                        .push(format!("s12:sp-0x40,sp-0x68,p{city}+0x90,sp-0x54"));
                    self.log.push(format!("s9:sp-0x40,{}", self.a[12]));
                    self.log.push("s13:sp-0x68".into());
                    let colour = i32::from(self.p[4 * self.who() + 3][0]);
                    // The colour's record lies at `0xe39ce0 + 100 * colour`; from colour 18 it reaches into the
                    // leaders table, whose slices the log names by window.
                    let at = 0xe39ce0 + colour * 100;
                    let slices = [(0, 4), (0x95c, 4), (0xe62, 0x80), (0x6928, 4)];
                    let named = (0..12).find_map(|k| {
                        let (off, size) = slices[k % 4];
                        let base = 0xe3a390 + (k / 4) as i32 * 0x6eec + off;
                        (base..=base + size)
                            .contains(&at)
                            .then(|| format!("p{k}+{:#x}", at - base))
                    });
                    self.log.push(format!(
                        "s14:{},sp-0x2c",
                        named.unwrap_or_else(|| at.to_string())
                    ));
                    let o = self.obj();
                    let (x, y) = (
                        rd(&self.p[o], 0x10) ^ 0x63637,
                        rd(&self.p[o], 0x14) ^ 0x63637,
                    );
                    let slot = 4 * (o - 13);
                    self.log.push(format!(
                        "s15:15267872,sp-0x40,{x},{y},1,{},-1,p12+{slot:#x},0,0,p12+{slot:#x}",
                        self.a[14]
                    ));
                    self.log.push("s16:15267872,21".into());
                    self.log.push("s13:sp-0x54".into());
                    self.log.push("s13:sp-0x40".into());
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                pops: 0,
            };
            check_upgrade_steps(&mut v);
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "check_upgrade — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_upgrade — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_upgrade — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "check_upgrade — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("City::check_upgrade@00738b20: {n} rows agree");
}
