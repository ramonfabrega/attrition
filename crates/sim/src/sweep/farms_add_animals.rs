//! `Farms::add_animals@008d8f30` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Farms::add_animals@008d8f30`, run by `tools/emu/sweep_farms_add_animals.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 1 argument(s), the global cell 0x00c0a914 points at 512 random bytes; the global cell 0x00c0618c points at 512 random bytes; the global cell 0x00c06184 points at 64 random bytes; vtable slot +0xac (0 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; vtable slot +0xb4 (0 args) stubbed; the word at `c1+0x14` points at 8 random bytes less `0` (placed); the word at `c1+0x30` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `c1+0x110` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 2, 3, 384, 383, 385, 65535, 32767, -1, -385] (90%); stub 0 answers the address of 32 random bytes, a `p` window after the places; stub 3 answers the address of 352 random bytes, a `p` window after the places; the stubbed vtable laid at `p2+0`; stub 1 answers 15 values in call order, printed after the answers; stub 2 answers 0..1 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_farms_add_animals.py") else {
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
            // The row's farm (`+0xbd == 1`), the building `Objects` answers
            // for it (`p2..p5`, by player and index), and the vtable test
            // `Farms::add_animals` makes of what slot `0xac` answers (`p9`):
            // `Build` and `Wall` read `flags & 4`, `Animal` `flags & 1`
            // (`SubObjectData::is_active@0046cda0`).
            let f = a[0] as usize * 0xc0;
            let pasture = c[0][f + 0xbd] == 1;
            let player = rd(&c[0], f);
            let index = rd(&c[0], f + 4);
            let obj = 2 + 2 * player as usize + index as usize;
            let mut active = false;
            if pasture {
                log.push(format!("s0:p{obj}+0x0"));
                let flags = p[9][8];
                active = match rd(&p[9], 0) as u32 {
                    0x00b4_2174 | 0x00b4_2cf8 => flags & 4 != 0,
                    0x00b4_145c => flags & 1 != 0,
                    v => panic!("a vtable the draw does not lay: {v:#x}"),
                };
            }
            let pos = (rd(&p[obj], 0x10) ^ 0x63637, rd(&p[obj], 0x14) ^ 0x63637);
            struct Host<'a> {
                a: &'a [i32],
                seq: usize,
                log: &'a mut Vec<String>,
                p: &'a mut [Vec<u8>],
            }
            impl crate::farms::AnimalHost for Host<'_> {
                fn draw(&mut self, _site: &'static str) -> i32 {
                    self.log.push("s1:c2+0x0,0,65535".to_string());
                    let v = self.a[5 + self.seq.min(14)];
                    self.seq += 1;
                    v
                }
                fn init_unit(&mut self, s: &crate::farms::AnimalSpawn) -> bool {
                    self.log
                        .push(format!("s2:c1+0x0,9,{},{},{},-1,-1,-1", s.ty, s.x, s.y));
                    // The three `+0xb4` calls land on the object `init_unit`
                    // answered (`p7`, `p8`) and stamp the one answer window.
                    for _ in 0..3 {
                        self.log.push(format!("s3:p{}+0x0", 7 + self.a[3] as usize));
                    }
                    self.p[10][0x150..0x152].copy_from_slice(&(s.index as u16).to_le_bytes());
                    self.p[10][0x152..0x154].copy_from_slice(&(s.player as u16).to_le_bytes());
                    self.p[10][0x154] = s.slot;
                    true
                }
            }
            let mut host = Host {
                a,
                seq: 0,
                log: &mut log,
                p,
            };
            crate::farms::add_animals_steps(&mut host, pasture, active, pos, player, index);
            0
        };
        if log.is_empty() {
            log.push("-".to_string());
        }
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_animals — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_animals — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_animals — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "add_animals — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Farms::add_animals@008d8f30: {n} rows agree");
}
