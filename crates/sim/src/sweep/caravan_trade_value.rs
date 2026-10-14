//! `Caravan::trade_value@0073d9d0` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `Caravan::trade_value@0073d9d0`, run by `tools/emu/sweep_caravan_trade_value.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00c061f0 points at 2304 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x0073d300 (4 args) stubbed; callee 0x007363f0 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 20 random bytes less `0` (placed); the word at `p6+4` points at 20 random bytes less `0` (placed); the word at `p7+0` points at 20 random bytes less `0` (placed); the word at `p7+4` points at 20 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3] (90%); stub 1 answered from [0, 1, 2, 3, 4, 6, 9, 12, 20] (90%); stub 2 answered from [0, 1] (90%); `__cdecl`: the caller pops the arguments; stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 is `__cdecl`: the caller pops its arguments; 2 rows 0x1c apart at `@0xc09970`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x6da4:0x4, +0x6dcc:0x4 (`p` windows after the pointed buffers, before the places); `ecx` an integer in 0..1, not the object, the row's last; `edx` an integer in 0..1, after `ecx` on the row; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_caravan_trade_value.py") else {
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
            // `a`: the partner's city index and leader (the two arguments), the three stubs'
            // answers, `get_trade_value`'s two in call order, then `ecx` (the near end's city
            // index) and `edx` (its leader). The cities table's two rows point at two-city arrays
            // (p6, p7), whose cities are p8..p11 (`8 + 2·leader + index`); the leaders' `rare` and
            // `rare_conquest` words are p2..p5 (two a row).
            let (ia, la, ib, lb) = (a[7] as usize, a[8] as usize, a[0] as usize, a[1] as usize);
            let (ka, kb) = (8 + 2 * la + ia, 8 + 2 * lb + ib);
            let pos = |k: usize| crate::world::Pos {
                x: rd(&p[k], 0xc),
                y: rd(&p[k], 0x10),
            };
            let rare = u64::from((rd(&p[2 + 2 * la], 0) | rd(&p[3 + 2 * la], 0)) as u32);
            let mut turn = 5;
            // The three callees' records share one log, each closure borrowing it in turn.
            let calls = std::cell::RefCell::new(Vec::new());
            let v = crate::caravan::trade_value_steps(
                (pos(ka), pos(kb)),
                la != lb,
                |pa, pb| {
                    calls
                        .borrow_mut()
                        .push(format!("s0:c1+0x0,{},{},{},{}", pa.x, pa.y, pb.x, pb.y));
                    a[2]
                },
                |end_a| {
                    calls
                        .borrow_mut()
                        .push(format!("s1:p{}+0x0", if end_a { ka } else { kb }));
                    turn += 1;
                    a[turn - 1]
                },
                |bonus| {
                    calls
                        .borrow_mut()
                        .push(format!("s2:{},{bonus}", 0xe3a390 + 0x6eec * la));
                    a[4] != 0
                },
                crate::rares::rare_bit(rare, crate::economy::SPICE),
                (rd(&c[0], 0x890), rd(&c[0], 0x8f0)),
            );
            log.extend(calls.into_inner());
            i64::from(v)
        };
        assert_eq!(got, want_eax, "trade_value — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "trade_value — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "trade_value — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "trade_value — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "trade_value — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Caravan::trade_value@0073d9d0: {n} rows agree");
}
