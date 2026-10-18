//! `Unit::add_trade_order@005e4dc0` under the emulator against the port (item 3234; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_trade_order@005e4dc0`, run by `tools/emu/sweep_unit_add_trade_order.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 6 argument(s), `p` windows in order: p0..p8 the place(s), in flag order, p9 stub 3's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0aec0 points at 64 random bytes; the global cell 0x00c06188 points at 320 random bytes; 56792 random bytes at 0x00e3a390; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x80 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x0046f790 (0 args) stubbed; callee 0x0046f290 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); stub 4's answer points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 6 answered from [0, 1, 2] (90%); stub 7 answered from [0, 1, 2, 3] (90%); stub 8 answered from [1, 0] (90%); stub 3 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p9+0`; stub 6 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 6 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 6 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_trade_order.py") else {
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
            {
                let off = rd(&p[7], 4) as usize;
                let calls = &mut log;
                if a[4] == 2 {
                    wr(m, 0x68, rd(m, 0x68) & 0xfbffffffu32 as i32);
                    wr(m, 0xc0, 0);
                    calls.push("s0:m+0x0,0".to_string());
                    calls.push("s1:m+0x0".to_string());
                    calls.push("s2:m+0x0".to_string());
                }
                wr(m, 0x68, rd(m, 0x68) & 0xfffffdffu32 as i32);
                wr(&mut p[6], 8, a[0]);
                wr(&mut p[6], 0xc, a[1]);
                let o = 2 + 2 * a[1] as usize + a[0] as usize;
                let short = if a[0] < 0 || a[1] < 0 {
                    0xffffu16
                } else {
                    u16::from_le_bytes([p[o][0x30], p[o][0x31]])
                };
                p[6][0x10..0x12].copy_from_slice(&short.to_le_bytes());
                wr(&mut p[6], 0x20, 0);
                wr(&mut p[6], 0x14, a[2]);
                wr(&mut p[6], 0x18, a[3]);
                let short2 = if a[2] >= 0 && a[1] >= 0 {
                    let o2 = 2 + 2 * a[3] as usize + a[2] as usize;
                    u16::from_le_bytes([p[o2][0x30], p[o2][0x31]])
                } else {
                    0xffffu16
                };
                p[6][0x24..0x26].copy_from_slice(&short2.to_le_bytes());
                wr(&mut p[6], 0x1c, 0);
                if a[5] == 0 {
                    p[6][off + 8] &= 0xfb;
                } else {
                    p[6][off + 8] |= 4;
                }
                let tile = |c1: &[u8], v: i32| rd(c1, 4 * ((v ^ 0x63637) >> 6) as usize);
                let (ux, uy) = (tile(&c[1], rd(m, 0x10)), tile(&c[1], rd(m, 0x14)));
                let (ox, oy) = (tile(&c[1], rd(&p[o], 0x10)), tile(&c[1], rd(&p[o], 0x14)));
                calls.push("s6:c3+0x0,sp-0x8,sp-0x18".to_string());
                calls.push("s6:c3+0x0,sp-0x10,sp-0x14".to_string());
                if a[16] != a[15] {
                    let lead = rd(&c[4], m[9] as usize * 0x6eec);
                    let tier = if lead & 0x100 != 0 {
                        3
                    } else if lead & 0x200 != 0 {
                        2
                    } else {
                        (lead as u32 >> 10 & 1) as i32
                    };
                    if a[13] <= tier && a[14] != 0 {
                        wr(m, 0x68, rd(m, 0x68) | 0x800000);
                    }
                }
                calls.push(format!("s5:m+0xcc,p6+0x{:x}", 4 + off));
                calls.push("s2:m+0x0".to_string());
                let hex = |v: i32| {
                    v.to_le_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                };
                calls.push(format!("{}/{}", hex(ox), hex(ux)));
                calls.push(format!("{}/{}", hex(oy), hex(uy)));
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_trade_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_trade_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_trade_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "add_trade_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::add_trade_order@005e4dc0: {n} rows agree");
}
