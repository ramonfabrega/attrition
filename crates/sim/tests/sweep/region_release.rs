//! `Region::release@00680560` under the emulator against the port (item 2044; hand-written on tools/emu/hooks.py).

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Region::release@00680560`, run by `tools/emu/sweep_region_release.py` under unicorn (tools/emu): each of
/// the region's listed tiles has its short at `+4` zeroed, `size` and the word at
/// `+0x70` are cleared, and nothing else of the region or the tiles is written; a `size` below 1 clears the
/// two words only. The census's `port` mark is `pivot.rs`'s unrelated `release`.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_region_release.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("`<args> | <before> -> <eax> | <after>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (_, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let mut f = args.split_whitespace();
        let w: usize = f.next().expect("width").parse().expect("an integer");
        let _size: i32 = f.next().expect("size").parse().expect("an integer");
        let listed = f.next().expect("the list");
        let mut b = before.split_whitespace().map(unhex);
        let (mut this, mut tiles) = (b.next().expect("this"), b.next().expect("tiles"));
        if listed != "-" {
            for p in listed.split(';') {
                let (x, y) = p.split_once(',').expect("x,y");
                let (x, y): (usize, usize) = (x.parse().expect("x"), y.parse().expect("y"));
                let t = (x + w * y) * 0x1c + 4;
                tiles[t..t + 2].fill(0);
            }
        }
        this[0x14..0x18].fill(0);
        this[0x70..0x74].fill(0);
        let mut a = after.split_whitespace().map(unhex);
        assert_eq!(
            this,
            a.next().expect("this"),
            "release — the region, row `{line}`"
        );
        assert_eq!(
            tiles,
            a.next().expect("tiles"),
            "release — the tiles, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Region::release@00680560: {n} rows agree");
}
