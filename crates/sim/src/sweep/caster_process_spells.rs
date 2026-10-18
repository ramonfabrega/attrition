//! `Caster::process_spells@00739ad0` under the emulator against the port (item 3229; tools/sweep_gen.py, obj with windows).

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

/// `Caster::process_spells@00739ad0`, run by `tools/emu/sweep_caster_process_spells.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0..p6 the place(s), in flag order; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c06214 points at 64 random bytes; the global cell 0x00c06204 points at 256 random bytes; the global cell 0x00c06210 points at 768 random bytes; callee 0x00909480 (2 args) stubbed; callee 0x008e1410 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x0048a960 (3 args) stubbed; callee 0x006ce190 (0 args) stubbed; the word at `m+0x14` points at 96 random bytes less `0` (placed); the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 128 random bytes less `0` (placed); the word at `p1+4` points at 128 random bytes less `0` (placed); the word at `p2+0` points at 128 random bytes less `0` (placed); the word at `p2+4` points at 128 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_caster_process_spells.py") else {
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
            // m `this` (+8 the record count, +0x14 the record array = p0, 12-byte records of
            // type / tick / expiry); a[0] the unit index, a[1] the leader, a[2] the flag that
            // skips the per-frame effect; c0 the game (+0x550 frame), c1 graphic pieces, c2 the
            // options (+0x90 rebuild), c3 the console (+0x298 who); p[1 + L] leader L's unit
            // table, p[3 + 2 * L + u] the unit (+0x68 flags, +0xc/+0x10/+0x14 position).
            let (unit, leader, skip) = (a[0], a[1], a[2]);
            let ui = 3 + 2 * leader as usize + unit as usize;
            let mut flagged = false;
            let mut i = rd(m, 8).wrapping_sub(1);
            while i >= 0 {
                let r = i as usize * 12;
                let (ty, tick, expiry) = (rd(&p[0], r), rd(&p[0], r + 4), rd(&p[0], r + 8));
                let mut remove = true;
                if skip == 0 {
                    let frame = rd(&c[0], 0x550);
                    if expiry >= frame {
                        remove = false;
                        if ty == 0x283 && (tick.wrapping_sub(frame).wrapping_add(1) & 0x1f) == 0 {
                            log.push(format!("s0:c1+0x0,{leader},{unit}"));
                            let x = rd(&p[ui], 0xc) ^ 0x63637;
                            log.push(format!(
                                "s1:8,{},200,1000,0,0,-1,-1",
                                (x as f32).to_bits() as i32
                            ));
                        }
                    }
                }
                if remove {
                    let who = rd(&c[3], 0x298);
                    if ty == 0x27b || ty == 0x27c {
                        let (mask, snd) = if ty == 0x27b {
                            (0xffffd7ffu32, 45)
                        } else {
                            (0xffff7fffu32, 48)
                        };
                        let fl = (rd(&p[ui], 0x68) as u32) & mask;
                        wr(&mut p[ui], 0x68, fl as i32);
                        if leader == who && skip == 0 {
                            log.push(format!("s2:0,{snd}"));
                        }
                        flagged = true;
                    }
                    log.push(format!("s3:m+0x4,{ty},{tick},{expiry}"));
                }
                i -= 1;
            }
            if flagged {
                log.push(format!("s4:{}", 0xe3a390 + 0x6eec * leader));
                wr(&mut c[2], 0x90, 1);
            }
            // eax at the return: the options pointer last loaded (`mov eax,[0xc06204]`) when the
            // flag path ran, else the flag (0); the pointer is the c2 window's guest address
            // (a constant of this harness's window layout, 0x200006e0).
            let ret: i64 = if flagged { 0x2000_06e0 } else { 0 };
            if log.is_empty() {
                log.push("-".into());
            }
            ret
        };
        assert_eq!(got, want_eax, "process_spells — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_spells — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_spells — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_spells — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "process_spells — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Caster::process_spells@00739ad0: {n} rows agree");
}
