//! `BuildData::get_base_garrison_arrows@0062e340` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::get_base_garrison_arrows@0062e340`, run by `tools/emu/sweep_build_data_get_base_garrison_arrows.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), the global cell 0x00c061d4 points at 256 random bytes; the global cell 0x00c061e4 points at 1424 random bytes; 4 random bytes at 0x00c061e0; callee 0x006e1370 (1 args) stubbed; vtable slot +0x4c (0 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x2cc` (placed); the word at `c0+0x10+u8(m,9)*0x1c` points at 4 random bytes less `i16(m,0x72)*4` (placed); the word at `p1+0` points at 96 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_get_base_garrison_arrows.py") else {
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
                // The type's base arrows (+0x18 -> +0x2cc); with the owner's tribe bonus
                // (has_tribe_bonus(1) on leaders + owner*0x6eec) the Maya's extra
                // (constants +0x588) is added, unless the building is a wall piece (+8
                // bit 0x20) that is active (slot +0x4c, inlined as +8 bit 4 when it is
                // WallData::is_active) and its city (cities[owner][+0x72]) has +0x5f and
                // +0x5e apart.
                let owner = i32::from(m[9]);
                let mut v = rd(&p[0], 0);
                log.push(
                    format!("{}", rd(&c[2], 0).wrapping_add(owner.wrapping_mul(0x6eec))) + ",1",
                );
                let mut slot = Vec::new();
                if a[0] != 0 {
                    let mut wall_city_apart = false;
                    if m[8] & 0x20 != 0 {
                        let active = if a[2] != 0 {
                            i32::from(m[8] & 4)
                        } else {
                            slot.push("m+0x0".to_string());
                            a[1]
                        };
                        wall_city_apart = active != 0 && p[2][0x5f] != p[2][0x5e];
                    }
                    if !wall_city_apart {
                        v = v.wrapping_add(rd(&c[1], 0x588));
                    }
                }
                if slot.is_empty() {
                    log.push("-".into());
                } else {
                    log.push(slot.join("/"));
                }
                i64::from(v)
            }
        };
        assert_eq!(
            got, want_eax,
            "get_base_garrison_arrows — eax, row `{line}`"
        );
        assert_eq!(
            log, want_log,
            "get_base_garrison_arrows — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "get_base_garrison_arrows — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_base_garrison_arrows — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "get_base_garrison_arrows — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::get_base_garrison_arrows@0062e340: {n} rows agree");
}
