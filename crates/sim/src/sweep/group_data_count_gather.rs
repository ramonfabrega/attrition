//! `GroupData::count@00711720` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::count@00711720`, run by `tools/emu/sweep_group_data_count_gather.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 3 argument(s), `p` windows in order: p0..p37 the place(s), in flag order; vtable slot +0x10 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; vtable slot +0x178 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00633f50 (1 args) stubbed; callee 0x00639de0 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 72 random bytes less `0` (placed); the word at `p0+4` points at 72 random bytes less `0` (placed); the word at `p0+8` points at 72 random bytes less `0` (placed); the word at `p0+12` points at 72 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 72 random bytes less `0` (placed); the word at `p5+4` points at 72 random bytes less `0` (placed); the word at `p5+8` points at 72 random bytes less `0` (placed); the word at `p5+12` points at 72 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 152 random bytes less `0` (placed); the word at `p10+4` points at 152 random bytes less `0` (placed); the word at `p10+8` points at 152 random bytes less `0` (placed); the word at `p10+12` points at 152 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 152 random bytes less `0` (placed); the word at `p15+4` points at 152 random bytes less `0` (placed); the word at `p15+8` points at 152 random bytes less `0` (placed); the word at `p15+12` points at 152 random bytes less `0` (placed); stub 1's answer points at 96 random bytes less `0` (placed); the word at `p20+0x18` points at 704 random bytes less `0` (placed); the word at `p1+0x18` points at 704 random bytes less `0` (placed); the word at `p2+0x18` points at 704 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); the word at `p4+0x18` points at 704 random bytes less `0` (placed); the word at `p6+0x18` points at 704 random bytes less `0` (placed); the word at `p7+0x18` points at 704 random bytes less `0` (placed); the word at `p8+0x18` points at 704 random bytes less `0` (placed); the word at `p9+0x18` points at 704 random bytes less `0` (placed); the word at `p11+0x18` points at 704 random bytes less `0` (placed); the word at `p12+0x18` points at 704 random bytes less `0` (placed); the word at `p13+0x18` points at 704 random bytes less `0` (placed); the word at `p14+0x18` points at 704 random bytes less `0` (placed); the word at `p16+0x18` points at 704 random bytes less `0` (placed); the word at `p17+0x18` points at 704 random bytes less `0` (placed); the word at `p18+0x18` points at 704 random bytes less `0` (placed); the word at `p19+0x18` points at 704 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); the word at `p20+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 440 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_count_gather.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `GroupData::count`, arm COUNT_GATHER_POINT (23), inlined: the crate has no `count` of a group's
                // list. Windows as the units slice, the object's `slot 0x3c` answering the placed record p[20]
                // (its vtable the image's `Build::vftable` or the stubbed one, laid half the rows by the
                // sweep; its `+0x18` type record p[21]). Stubs, each recorded and answered in call order
                // (a[10 + 8 q + j] for the six that draw answers, q their order): 0 the object's slot 0x10,
                // 1 its slot 0x3c, 2 the build's slot 0x178, 3 its slot 0xb8, 4 the type record's slot 0x60
                // (what the `Build::vftable` arm calls in place of slot 0xb8, 0x211/0x21a/0x1b6/0x1a4/0x208 and
                // 0 handed), 5 `BuildTypeData::get_garrison_limit` (the byte at +9, on the build's type),
                // 6 `BuildTypeData::is_training_building`.
                let num = rd(m, 12);
                let who = usize::from(m[0x4a]);
                let mut turn = [0usize; 7];
                let mut calls: Vec<Vec<String>> = vec![Vec::new(); 7];
                let mut ask = |s: usize, ecx: &str, args: &str| -> i32 {
                    calls[s].push(format!("{ecx}{args}"));
                    if s == 1 {
                        return 0;
                    }
                    let q = [0usize, 0, 1, 2, 3, 4, 5][s];
                    let v = a[10 + 8 * q + turn[s].min(7)];
                    turn[s] += 1;
                    v
                };
                let b = &p[20];
                let build = rd(b, 0) == 0xb42174;
                let (bn, btn) = ("p20+0x0", "p21+0x0");
                let is = |ask: &mut dyn FnMut(usize, &str, &str) -> i32, ty: i32| {
                    let args = format!(",{ty},0");
                    if build {
                        ask(4, btn, &args)
                    } else {
                        ask(3, bn, &args)
                    }
                };
                let mut cnt = 0i32;
                for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i =
                        usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                            .expect("an index");
                    let obj = &p[5 * who + 1 + i];
                    if rd(obj, 8) & 1 == 0 {
                        continue;
                    }
                    let on = format!("p{}+0x0", 5 * who + 1 + i);
                    if ask(0, &on, "") == 0 {
                        continue;
                    }
                    ask(1, &on, "");
                    // Whether the chain reaches the final pair of tests (any early answer does).
                    let mut reach = (if build {
                        ask(5, btn, &format!(",{}", b[9]))
                    } else {
                        ask(2, bn, "")
                    }) != 0;
                    if !reach {
                        ask(1, &on, "");
                        reach = ask(6, btn, "") != 0;
                    }
                    for ty in [0x211, 0x21a] {
                        if !reach {
                            ask(1, &on, "");
                            reach = is(&mut ask, ty) != 0;
                        }
                    }
                    if !reach {
                        ask(1, &on, "");
                        if is(&mut ask, 0x1b6) == 0 {
                            continue;
                        }
                    }
                    ask(1, &on, "");
                    if is(&mut ask, 0x1a4) != 0 {
                        continue;
                    }
                    ask(1, &on, "");
                    if is(&mut ask, 0x208) == 0 {
                        cnt += 1;
                    }
                }
                for c in &calls {
                    log.push(if c.is_empty() {
                        "-".into()
                    } else {
                        c.join("/")
                    });
                }
                i64::from(cnt)
            }
        };
        assert_eq!(got, want_eax, "count — eax, row `{line}`");
        assert_eq!(log, want_log, "count — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 440, "only {n} rows");
    eprintln!("GroupData::count@00711720: {n} rows agree");
}
