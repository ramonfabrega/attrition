//! `Unit::cavarch_fight@005ff4b0` under the emulator against the port (item 3134; tools/sweep_gen.py, obj with windows).

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

/// `Unit::cavarch_fight@005ff4b0`, run by `tools/emu/sweep_unit_cavarch_fight_written.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), `p` windows in order: p0..p8 the place(s), in flag order; the global cell 0x00c0618c points at 96 random bytes; callee 0x005fd4d0 (5 args) stubbed; vtable slot +0xe4 (0 args) stubbed; the word at `m+0x18` points at 736 random bytes less `0` (placed); the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+4` points at 32 random bytes less `0` (placed); the word at `p2+8` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 2, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes 1 (2 bytes) at `m+0xa2` each call; stub 0 writes 0 (1 bytes) at `m+0xa8` each call; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_cavarch_fight_written.py") else {
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
            {
                let rd = |w: &[u8], o: usize| {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                };
                let rd16 = |w: &[u8], o: usize| {
                    i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32
                };
                // a[1] the slot +0xe4 stub's answer; windows: m the unit, p0 its object record (+0x18), p3 + 3 * player + index
                // the units a player's array names. `fight` is stubbed; FIGHT_WRITES says whether it leaves the target as (1, 0).
                const FIGHT_WRITES: bool = true;
                let (idx, pl) = (rd16(m, 0xa2), m[0xa8] as i8 as i32);
                if rd(&p[0], 0x2b4) & 0x400 != 0 {
                    let v = rd(&p[0], 0x2d0);
                    p[0][0x1fc..0x200].copy_from_slice(&v.to_le_bytes());
                }
                log.push(format!("m+0x0,{},{},{},0,1", idx, pl, rd(m, 0x6c) & 0x100));
                if FIGHT_WRITES {
                    m[0xa2..0xa4].copy_from_slice(&1i16.to_le_bytes());
                    m[0xa8] = 0;
                }
                if rd(&p[0], 0x2b4) & 0x400 != 0 {
                    p[0][0x1fc..0x200].copy_from_slice(&0i32.to_le_bytes());
                }
                if idx >= 0
                    && pl >= 0
                    && (rd16(m, 0xa2) != idx || m[0xa8] as i8 as i32 != pl)
                    && (m[0xa8] as i8 as i32 != pl || a[1] != rd16(m, 0xa2))
                {
                    let v = rd(m, 0x6c) & !0x100;
                    m[0x6c..0x70].copy_from_slice(&v.to_le_bytes());
                }
                if rd16(m, 0xa2) < 0 && (idx as u16) >= 0x8000 {
                    m[0xa2..0xa4].copy_from_slice(&(-2i16).to_le_bytes());
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "cavarch_fight — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cavarch_fight — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "cavarch_fight — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "cavarch_fight — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Unit::cavarch_fight@005ff4b0: {n} rows agree");
}
