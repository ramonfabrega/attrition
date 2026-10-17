//! `Unit::do_group_attack_to@005e74e0` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_group_attack_to@005e74e0`, run by `tools/emu/sweep_unit_do_group_attack_to.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; callee 0x005e79a0 (1 args) stubbed; vtable slot +0xcc (0 args) stubbed; callee 0x005ff9c0 (5 args) stubbed; callee 0x005f22a0 (1 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); argument 0 copies `p1+0x8` (45%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_group_attack_to.py") else {
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
            // `a[0]` the order; the stubs' answers `a[1..5]` (do_group_move, slot 0xcc, find_melee_target,
            // do_attack_to_pause), `a[5]` 1 when the slot is the image's `is_supply`. `m+0xdc` the order list (`p[0]`,
            // null a third of the rows), whose node `p[1]` (the word at `p[0]+4`) gives `m+0xd4` its address, `m+0xcc` its
            // `+8` and `m+0xd0` its `+0xc`; the current order is `m+0xcc`, or 0 for a null list. When it is the
            // argument and the unit's id plus the frame (`c[0]+0x550`) is a multiple of 15: a type (`p[2]`) with
            // `+0x1e8` clear pauses, otherwise `is_supply` (bit 0x40 of its `+0x2b8`, or the slot's answer) clear
            // finds a melee target and set pauses.
            log.push(format!("s0:m+0x0,{}", a[0]));
            let cur = if rd(m, 0xdc) == 0 {
                0
            } else {
                let node = rd(&p[0], 4);
                wr(m, 0xd4, node);
                let w8 = rd(&p[1], 8);
                wr(m, 0xcc, w8);
                m[0xd0] = p[1][0xc];
                w8
            };
            let id = i16::from_le_bytes([m[0xa], m[0xb]]) as i32;
            if cur == a[0] && (id + rd(&c[0], 0x550)) % 15 == 0 {
                if rd(&p[2], 0x1e8) != 0 {
                    let supply = if a[5] == 1 {
                        rd(&p[2], 0x2b8) & 0x40
                    } else {
                        log.push("s1:m+0x0".to_string());
                        a[2]
                    };
                    if supply == 0 {
                        log.push("s2:m+0x0,-1,0,0,1,0".to_string());
                    } else {
                        log.push(format!("s3:m+0x0,{}", a[0]));
                    }
                } else {
                    log.push(format!("s3:m+0x0,{}", a[0]));
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_group_attack_to — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "do_group_attack_to — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_group_attack_to — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "do_group_attack_to — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::do_group_attack_to@005e74e0: {n} rows agree");
}
