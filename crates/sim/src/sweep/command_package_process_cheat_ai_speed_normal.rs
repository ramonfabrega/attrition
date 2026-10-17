//! `CommandPackage::process_cheat_ai_speed_normal@00944df0` under the emulator against the port (item 2941; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_cheat_ai_speed_normal@00944df0`, run by `tools/emu/sweep_command_package_process_cheat_ai_speed_normal.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c061c0 points at 16 random bytes; 8 random bytes at 0x00eb13a8; the import cell 0x00ac543c holds a stub (2 args); callee 0x004634d0 (4 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; stub 0 answered from [0, 4194304] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_cheat_ai_speed_normal.py")
    else {
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
            // strrchr's answer picks the file name (the constant when 0, else the hit + 1); the sync log's line 0x8f9,
            // `Log::say` round `game_log`'s type and detail (put back unless the detail was negative, then 1), then
            // the effect unless `game->semaphore` bit 4.
            let file = if a[1] == 0 {
                0xafb344
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!("0,{file},2297,4,11503176"));
            log.push(format!(
                "15405920,{},15418236,0,0,0,0,0,0,0,0",
                rd(&c[0], 0x10).wrapping_add(0x35fc)
            ));
            if rd(&c[3], 4) < 0 {
                wr(&mut c[3], 4, 1);
            }
            if c[1][0x820] & 4 == 0 {
                wr(&mut c[2], 0, 1);
            }
            1
        };
        assert_eq!(
            got, want_eax,
            "process_cheat_ai_speed_normal — eax, row `{line}`"
        );
        assert_eq!(
            log, want_log,
            "process_cheat_ai_speed_normal — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "process_cheat_ai_speed_normal — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_cheat_ai_speed_normal — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "process_cheat_ai_speed_normal — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_cheat_ai_speed_normal@00944df0: {n} rows agree");
}
