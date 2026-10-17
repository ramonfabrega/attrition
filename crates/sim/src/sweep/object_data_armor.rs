//! `ObjectData::armor@00647db0` under the emulator against the port (item 2837; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::armor@00647db0`, run by `tools/emu/sweep_object_data_armor.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 2240 random bytes; callee 0x006e1370 (1 args) stubbed; callee 0x00595260 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xc8 (0 args) stubbed; the word at `m+0x18` points at 544 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p1+0` points at 224 random bytes less `0` (placed); the word at `p2+0` points at 224 random bytes less `0` (placed); the word at `p3+0` points at 224 random bytes less `0` (placed); the word at `p4+0` points at 224 random bytes less `0` (placed); the word at `p5+0` points at 224 random bytes less `0` (placed); the word at `p6+0` points at 224 random bytes less `0` (placed); the word at `p7+0` points at 224 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [7, 8, 9, 10] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `0xc061e0`, slices +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_armor.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The type's armor (`+0x214` of the record at `m+0x18`, window p8); a Dutch attack bonus
            // (`constantsc +0x8b8`, `c[0]`) added times the owner's (`m[9]`, bucket buffer `p[9 + who]`
            // word `+0xdc` XOR 0x62766) when the owner has the tribe bonus (`a[0]`), the object's slot
            // +0x18 (`a[2]`) holds, the type is `0x3d`, `0x3e` or 400 or slot +0xd0 (`a[3]`) or +0xcc
            // (`a[4]`) does, and the patch version (`a[1]`) is 8 or less or slot +0xc8 (`a[5]`) does
            // not (item 2837).
            let armor = rd(&p[8], 0x214);
            let bonus = rd(&c[0], 0x8b8);
            let id = rd(&p[8], 4);
            let r = 'arm: {
                if a[0] == 0 || bonus == 0 {
                    break 'arm i64::from(armor);
                }
                log.push("s2:m+0x0".into());
                if a[2] == 0 {
                    break 'arm i64::from(armor);
                }
                if id != 0x3d && id != 0x3e && id != 400 {
                    log.push("s3:m+0x0".into());
                    if a[3] == 0 {
                        log.push("s4:m+0x0".into());
                        if a[4] == 0 {
                            break 'arm i64::from(armor);
                        }
                    }
                }
                if a[1] > 8 {
                    log.push("s5:m+0x0".into());
                    if a[5] != 0 {
                        break 'arm i64::from(armor);
                    }
                }
                let who = usize::from(m[9]);
                i64::from(
                    (rd(&p[9 + who], 0xdc) ^ 0x62766)
                        .wrapping_mul(bonus)
                        .wrapping_add(armor),
                )
            };
            if log.is_empty() {
                log.push("-".into());
            }
            r
        };
        assert_eq!(got, want_eax, "armor — eax, row `{line}`");
        assert_eq!(log, want_log, "armor — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "armor — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "armor — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "armor — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::armor@00647db0: {n} rows agree");
}
