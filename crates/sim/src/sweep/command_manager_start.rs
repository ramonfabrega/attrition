//! `CommandManager::start@00942e10` under the emulator against the port (item 3396; tools/sweep_gen.py, obj with windows).

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

// hand: port
/// No crate method is this function's counterpart (`CommandManager` is the cut multiplayer
/// session's start, with no sim state); the port is inline, with the file's constants, over the
/// windows the sweep lays: c0 the game record, c1 the string table's record, c2/c3 the loading
/// and scenario splash records, c4/c5 the console and interface records, c6 the scene record,
/// c7 `use_mp_playback`, c8 `time_sync` (time, base time), c9 the game log's (type, detail).
/// A call is logged as the sweep's recorder prints it: `s<stub>:<ecx>,<args>`.
fn start_port(a: &[i32], c: &mut [Vec<u8>], log: &mut Vec<String>) {
    const GAME: usize = 0;
    const MP: usize = 7;
    const TS: usize = 8;
    const GL: usize = 9;
    const CM: i32 = 0xe8ff60;
    const EMPTY: i32 = 0x00eb437c;
    let strs = rd(&c[1], 0x10);
    let sync = |log: &mut Vec<String>, ecx: &str, off: i32| {
        log.push(format!("s0:{ecx},{}", strs.wrapping_add(off)));
    };
    if c[MP][0] != 0 {
        c[GAME][0x820] &= !0x10;
        if rd(&c[GAME], 0x81c) == 0 {
            wr(&mut c[GAME], 0x81c, 2);
        }
    }
    let sem = c[GAME][0x820];
    let mut ecx = "m+0x0".to_string();
    if sem & 4 != 0 && sem & 0x10 == 0 {
        sync(log, &ecx, 0x2ea4);
        log.push("s1:m+0x0".into());
        log.push("s2:m+0x0".into());
        log.push("s1:m+0x0".into());
        let (ty, detail) = (rd(&c[GL], 0), rd(&c[GL], 4));
        let when = rd(&c[TS], 0).wrapping_sub(rd(&c[TS], 4)).wrapping_add(a[1]);
        log.push(format!(
            "s3:{},{},{EMPTY},{when},0,0,0,0,0,0,1",
            0xeb1360,
            strs.wrapping_add(0x2eb8)
        ));
        wr(&mut c[GL], 0, ty);
        wr(&mut c[GL], 4, if detail >= 0 { detail } else { 0 });
        log.push(format!("s4:{CM}"));
        log.push(format!("s5:{CM}"));
        ecx = CM.to_string();
    }
    sync(log, &ecx, 0x2ecc);
    let (win, vft) = if c[GAME][0x822] & 8 == 0 {
        if c[GAME][0x820] & 4 != 0 {
            sync(log, &ecx, 0x2ee0);
        }
        (2, 0xb3e4ac)
    } else {
        (3, 0xb3d0c4)
    };
    if rd(&c[win], 0) == vft {
        if win == 3 {
            log.push("s8:c3+0x474".into());
        }
        log.push(format!("s6:c{win}+0x0"));
        log.push(format!("s7:c{win}+0x0"));
    } else {
        log.push(format!("s14:c{win}+0x0"));
    }
    c[GAME][0x821] |= 1;
    wr(&mut c[GAME], 0x81c, 0);
    let say_time = |log: &mut Vec<String>, tag: &str, line: i32| {
        log.push(format!("s9:sp-0x28,{tag}"));
        log.push(format!("s10:sp-0x28,sp-0x28,sp-0x28,{line}"));
        log.push("s11:sp-0x28".into());
    };
    say_time(log, "11499056", 0x8e8);
    log.push("s15:c4+0x0,0".into());
    say_time(log, "11498960", 0x8ea);
    log.push("s15:c5+0x0,0".into());
    say_time(log, "11498980", 0x8ec);
    log.push(format!("s12:c6+0x1b0,{EMPTY}"));
    say_time(log, "11499152", 0x8ee);
    log.push("s13:sp-0x28,143".into());
    say_time(log, "11499172", 0x8f0);
}
// end hand

/// `CommandManager::start@00942e10`, run by `tools/emu/sweep_command_manager_start.py` under unicorn
/// (tools/emu): an empty body: 64 random bytes of object and 0 argument(s), every byte held after the return and `esp` checked (parked 1889).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_manager_start.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(10);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            start_port(a, c, &mut log);
            let _ = m;
            want_eax
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "start — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "start — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "start — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[11 + i], "start — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandManager::start@00942e10: {n} rows agree");
}
