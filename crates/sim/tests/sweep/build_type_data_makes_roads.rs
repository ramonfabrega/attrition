//! `BuildTypeData::makes_roads@00639910` under the emulator against the port (item 2112; tools/sweep_gen.py, vcall --base --base-call).

/// `BuildTypeData::makes_roads@00639910`, run by `tools/emu/sweep_build_type_data_makes_roads.py` under unicorn
/// (tools/emu): slot +0x64 of `this` against the base 0x00472bc0, a row each arm: the base arm calls slot +0x60 handed `414,0`, the other jumps into the slot with the 0 own argument(s); the slots' answer over the signed-32 edges and random words, every byte held.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_makes_roads.py") else {
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
        let handed: Vec<String> = ["414", "0"]
            .iter()
            .map(|t: &&str| match t.strip_prefix('p') {
                Some(i) => own[i.parse::<usize>().expect("pN")].to_string(),
                None => t.to_string(),
            })
            .collect();
        let own_s: String = own.iter().map(|x| format!(",{x}")).collect();
        let want_calls = if mode == 1 {
            format!(
                "base,this{}",
                handed.iter().map(|h| format!(",{h}")).collect::<String>()
            )
        } else {
            format!("slot,this{own_s}")
        };
        assert_eq!(
            calls.trim(),
            want_calls,
            "makes_roads — calls, row `{line}`"
        );
        let got: i64 = {
            let (mode, v, own) = (mode, v, own);
            let _ = (mode, v, own);
            i64::from(v)
        };
        assert_eq!(
            got,
            eax.trim().parse::<i64>().expect("an integer"),
            "makes_roads — eax, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::makes_roads@00639910: {n} rows agree");
}
