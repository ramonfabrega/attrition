//! `CommandPackage::process_ping@009453f0` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_ping@009453f0`, run by `tools/emu/sweep_command_package_process_ping.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c061ec points at 2304 random bytes; 227168 random bytes at 0x00e3a390; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b8350 (7 args) stubbed; callee 0x006d18a0 (3 args) stubbed; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_ping.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The sync log (strrchr's answer picks the file name, which is also the tags; `who` is a stack local), then
            // each of the eight leaders `i` whose flag byte bit 0 is set is pinged at (x, y) from `who` — unless
            // `leaders[who]`'s word `i` is nonzero or `leaders[i]`'s word `who` is 2 — both lifted by `game->semaphore` bit 6.
            let file = if a[1] == 0 {
                11513980
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,{file},2190,{file},11513872,sp-0x10,p0+0x1,p0+0x5"
            ));
            let who = c[0][0x77 + rd(m, 4) as usize * 0x8c] as usize;
            let (x, y) = (rd(&p[0], 1), rd(&p[0], 5));
            let sem = c[0][0x820] & 0x40 != 0;
            let mut pings: Vec<String> = Vec::new();
            for i in 0..8 {
                let at = i * 0x6eec;
                if c[1][at] & 1 != 0
                    && (rd(&c[1], 0x54 + (who * 0x1bbb + i) * 4) == 0 || sem)
                    && (rd(&c[1], 0x54 + who * 4 + at) != 2 || sem)
                {
                    pings.push(format!("c1+{at:#x},{x},{y},{who}"));
                }
            }
            log.push(if pings.is_empty() {
                "-".into()
            } else {
                pings.join("/")
            });
            9
        };
        assert_eq!(got, want_eax, "process_ping — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_ping — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_ping — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_ping — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "process_ping — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_ping@009453f0: {n} rows agree");
}
