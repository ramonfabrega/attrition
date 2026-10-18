//! `Unit::update_local_seen@0060e410` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_local_seen@0060e410`, run by `tools/emu/sweep_unit_update_local_seen.py` under unicorn
/// (tools/emu): 72 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00cae5fc points at 512 random bytes; the global cell 0x00c06188 points at 32 random bytes; 16 random bytes at 0x00cbe330 (each word in 0..24); 32 random bytes at 0x00cb7e90 (each byte in -4..4); 32 random bytes at 0x00cbb0e0 (each byte in -4..4); callee 0x006b4bb0 (4 args) stubbed; the word at `m+0x18` points at 576 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_local_seen.py") else {
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
            let le = |v: i32| {
                v.to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            };
            let d3 = |i: i32| rd(&c[0], 4 * i as usize);
            let seen = i32::from(m[0x40] as i8);
            let (mut t_call, mut t_x, mut t_y): (Vec<String>, Vec<String>, Vec<String>) =
                (Vec::new(), Vec::new(), Vec::new());
            if seen != 0 {
                let radius = rd(&c[2], 4 * rd(&p[0], 0x234) as usize);
                let gx = d3((rd(m, 0x10) ^ 0x63637) >> 7);
                let gy = d3((rd(m, 0x14) ^ 0x63637) >> 7);
                let (w, h) = (rd(&c[1], 0xc), rd(&c[1], 0x10));
                for (&bx, &by) in c[3].iter().zip(c[4].iter()).take(radius.max(0) as usize) {
                    let x = i32::from(bx as i8) + gx;
                    let y = i32::from(by as i8) + gy;
                    if x >= 0 && y >= 0 && x < w && y < h {
                        t_call.push(format!("{x},sp-0x10,sp-0x14,{seen},0"));
                        t_x.push(le(x));
                        t_y.push(le(y));
                    }
                }
            }
            for t in [t_call, t_x, t_y] {
                log.push(if t.is_empty() {
                    "-".to_string()
                } else {
                    t.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "update_local_seen — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_local_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_local_seen — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "update_local_seen — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Unit::update_local_seen@0060e410: {n} rows agree");
}
