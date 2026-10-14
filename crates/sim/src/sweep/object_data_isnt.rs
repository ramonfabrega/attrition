//! `ObjectData::isnt@00653480` under the emulator against the port (item 2112; tools/sweep_gen.py, vcall --base --base-call).

/// `ObjectData::isnt@00653480`, run by `tools/emu/sweep_object_data_isnt.py` under unicorn
/// (tools/emu): slot +0xf8 of at `this+0x18` against the base 0x00661b80, a row each arm: the base arm calls slot +0x60 handed `p0,p1`, and answers that slot's answer `== 0`, the other jumps into the slot with the 2 own argument(s); the slots' answer over the signed-32 edges and random words, every byte held.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_isnt.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<mode> <v> <own…> -> <eax> | <calls>`");
        let (eax, calls) = rhs.split_once(" | ").expect("`<eax> | <calls>`");
        let a: Vec<i64> = lhs
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let (mode, v, own) = (a[0], i32::try_from(a[1]).expect("a 32-bit answer"), &a[2..]);
        let handed: Vec<String> = ["p0", "p1"]
            .iter()
            .map(|t: &&str| match t.strip_prefix('p') {
                Some(i) => own[i.parse::<usize>().expect("pN")].to_string(),
                None => t.to_string(),
            })
            .collect();
        let own_s: String = own.iter().map(|x| format!(",{x}")).collect();
        let want_calls = if mode == 1 {
            format!(
                "base,rec{}",
                handed.iter().map(|h| format!(",{h}")).collect::<String>()
            )
        } else {
            format!("slot,rec{own_s}")
        };
        assert_eq!(calls.trim(), want_calls, "isnt — calls, row `{line}`");
        let got: i64 = {
            let (mode, v, own) = (mode, v, own);
            let _ = (mode, v, own);
            if mode == 1 {
                i64::from(v == 0)
            } else {
                i64::from(v)
            }
        };
        assert_eq!(
            got,
            eax.trim().parse::<i64>().expect("an integer"),
            "isnt — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::isnt@00653480: {n} rows agree");
}
