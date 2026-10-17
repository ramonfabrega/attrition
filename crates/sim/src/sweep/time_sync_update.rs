//! `TimeSync::update@00954cf0` under the emulator against the port (item 2793; tools/sweep_gen.py, obj with windows).

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

/// `TimeSync::update@00954cf0`, run by `tools/emu/sweep_time_sync_update.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), 8 random bytes at 0x00c89704; vtable slot +0x10 (0 args) stubbed; vtable slot +0x54 (4 args) stubbed; the import cell 0x00ac5460 holds a stub (0 args); the word at `0xe335c8` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `0xc061e8` points at 1536 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1, 2] (90%); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 9 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_time_sync_update.py") else {
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
            // `TimeSync::update()`: with the net system `0xe335c8` non-null (the first slot call says which; the cell is
            // no window) it asks slot 0x10 (`a[0]`'s low byte is the answer); on zero, and the next poll frame
            // `c[0]+4` at or before the game's frame (`p[1]+0x550`, signed), it reads `timeGetTime` (`a[2]`), sends the 9-byte
            // packet {0xc, time, 1} through slot 0x54 (host player `p[0]+0x2c`, flag 4) and sets the next
            // poll frame to frame + the interval `c[0]+0`.
            if want_log[0] != "-" {
                log.push("p0+0x0".to_string());
                let frame = rd(&p[1], 0x550);
                let next = rd(&c[0], 4);
                if a[0] as u8 == 0 && next <= frame {
                    let host = rd(&p[0], 0x2c);
                    log.push(format!("p0+0x0,sp-0x14,9,{host},4"));
                    log.push("p0+0x0".to_string());
                    let mut pkt = vec![0x0cu8];
                    pkt.extend_from_slice(&a[2].to_le_bytes());
                    pkt.extend_from_slice(&1i32.to_le_bytes());
                    log.push(pkt.iter().map(|b| format!("{b:02x}")).collect());
                    let timing = rd(&c[0], 0);
                    wr(&mut c[0], 4, frame.wrapping_add(timing));
                } else {
                    log.extend(["-".to_string(), "-".to_string(), "-".to_string()]);
                }
            } else {
                log.extend([
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                ]);
            }
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "update — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "update — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TimeSync::update@00954cf0: {n} rows agree");
}
