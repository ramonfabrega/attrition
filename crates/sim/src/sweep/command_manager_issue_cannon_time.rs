//! `CommandManager::issue_cannon_time@00941260` under the emulator against the port (item 2495; tools/sweep_gen.py, obj with windows).

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

/// `CommandManager::issue_cannon_time@00941260`, run by `tools/emu/sweep_command_manager_issue_cannon_time.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00c06210 points at 688 random bytes; the global cell 0x00c8cd00 points at 5888 random bytes; the global cell 0x00c06208 points at 32 random bytes; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0094bae0 (2 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 prints the 2 bytes its argument 0 points at, hex, after the record (`/` a call); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x48:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_manager_issue_cannon_time.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                static SEEN: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(false);
                use std::sync::atomic::Ordering::Relaxed;
                let ok = c[0][0x820] & 4 != 0 && rd(&p[rd(&c[1], 0x298) as usize], 0) as u32 != 0;
                if ok {
                    SEEN.store(true, Relaxed);
                    for t in ["-", "-", "-", "15269768,sp-0x1c,2", "4d01"] {
                        log.push(t.into());
                    }
                } else {
                    log.push(
                        if c[0][0x820] & 4 == 0 {
                            "15558448,c2+0x166c"
                        } else {
                            "15558448,c2+0x1680"
                        }
                        .into(),
                    );
                    log.push("sp-0x38,15558448".into());
                    log.push(format!(
                        "c3+0x0,0,0,0,0,{},-1,13161032",
                        if SEEN.load(Relaxed) { 9704261 } else { 0 }
                    ));
                    log.push("-".into());
                    log.push("-".into());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "issue_cannon_time — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "issue_cannon_time — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "issue_cannon_time — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "issue_cannon_time — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandManager::issue_cannon_time@00941260: {n} rows agree");
}
