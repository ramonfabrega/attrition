//! `BuildQueueData::get_queue@00630670` under the emulator against the port (item 1819).

/// `BuildQueueData::get_queue@00630670`, run by `tools/emu/sweep_build_queue_data_get_queue.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the queue data, one stack argument: `queue_size` at +0, the entries behind +4 (20 bytes each); for an index below the size the sign-extended word at entry+4, else -1 — negative indices included, read from the entry before the array. The entry at the index carries the word, every other the word 0x0101.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_data_get_queue.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (size, idx, v) = (a[0], a[1], a[2]);
        let got = if idx < size {
            (v ^ 0x8000) - 0x8000
        } else {
            -1
        };
        assert_eq!(got, want, "BuildQueueData::get_queue — row `{line}`");
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("BuildQueueData::get_queue@00630670: {n} rows agree");
}
