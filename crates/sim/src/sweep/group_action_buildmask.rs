//! `Group::action_buildmask@006fc9a0` under the emulator against the port (item 3200; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_buildmask@006fc9a0`, run by `tools/emu/sweep_group_action_buildmask.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 2 argument(s), `p` windows in order: p0..p3 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00c06210 points at 768 random bytes; vtable slot +0x10 (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; callee 0x0063e2a0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `c0+0x14` points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); stub 1's answer points at 128 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 0] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_buildmask.py") else {
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
            // m the group (`+0x49` armed, `+0x4a` player, `+0xc` count, `+0x8cc` the shorts of its
            // buildings); c1 the console record (`+0x298` who); p3 the record slot +0xb0 answers
            // (`+0x60` the mask's short). a[0] the mask; a[7..10] slot +0x10's three answers,
            // a[10..13] `valid_buildmask`'s, in call order, the last repeated. Recorded: the message
            // (stub 3) and the sound (stub 4).
            let mask = a[0];
            let count = i32::from_le_bytes(m[0xc..0x10].try_into().expect("four bytes"));
            let (mut msg, mut snd) = (Vec::new(), Vec::new());
            if m[0x49] != 0 && count > 0 {
                let (mut c0, mut c2) = (0usize, 0usize);
                let (mut any, mut set) = (false, true);
                for _ in 0..count {
                    let v = a[7 + c0.min(2)];
                    c0 += 1;
                    if v == 0 {
                        continue;
                    }
                    let ok = a[10 + c2.min(2)];
                    c2 += 1;
                    if ok == 0 {
                        continue;
                    }
                    let r60 = i16::from_le_bytes([p[3][0x60], p[3][0x61]]);
                    let b2 = mask & i32::from(r60) == 0 && set;
                    let new = if b2 {
                        r60 | mask as i16
                    } else {
                        r60 & !(mask as i16)
                    };
                    p[3][0x60..0x62].copy_from_slice(&new.to_le_bytes());
                    set = b2;
                    any = true;
                }
                let who = i32::from_le_bytes(c[1][0x298..0x29c].try_into().expect("four bytes"));
                if any && i32::from(m[0x4a]) == who && mask == 0x40 {
                    let (text, sound) = if set { (34340, 60) } else { (34360, 61) };
                    msg.push(format!(
                        "15267872,{text},-1,-1,0,13161032,-1,{who},0,0,{who}"
                    ));
                    snd.push(format!("15267872,{sound}"));
                }
            }
            for v in [msg, snd] {
                log.push(if v.is_empty() {
                    "-".to_string()
                } else {
                    v.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_buildmask — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_buildmask — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_buildmask — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "action_buildmask — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Group::action_buildmask@006fc9a0: {n} rows agree");
}
