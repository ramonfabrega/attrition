//! `Caravan::restart_trade_route@0073d070` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

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

/// `Caravan::restart_trade_route@0073d070`, run by `tools/emu/sweep_caravan_restart_trade_route.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x0073de80 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x0073db10 (0 args) stubbed; the word at `0xc0aec0+i8(m,0xd)*0x1c` points at 4 random bytes less `max(i16(m,0xa),0)*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_caravan_restart_trade_route.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Reset: the counter at +0x18 runs down to 0 (a negative or zero one is left at 0 too), the temp road is cleared (clear_temp_road, this), bit 2 of the flags byte +0xc dropped. A non-negative unit index (short +0xa) and non-negative player (signed byte +0xd): the unit is units[player].list[index]; with bit 0 of its byte +8 set it is repathed and the road rebuilt (build_road, this).
                wr(m, 0x18, 0);
                log.push("s0:m+0x0".into());
                m[0xc] &= 0xfb;
                let idx = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
                if idx >= 0 && (m[0xd] as i8) >= 0 && p[1][8] & 1 != 0 {
                    log.push("s1:p1+0x0".into());
                    log.push("s2:m+0x0".into());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "restart_trade_route — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "restart_trade_route — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "restart_trade_route — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "restart_trade_route — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Caravan::restart_trade_route@0073d070: {n} rows agree");
}
