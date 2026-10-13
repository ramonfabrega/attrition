//! `BuildQueueData::get_job_counter@00630690` under the emulator against the port (item 1819).

/// `BuildQueueData::get_job_counter@00630690`, run by `tools/emu/sweep_build_queue_data_get_job_counter.py` under unicorn
/// (tools/emu): `__thiscall`, `ecx` = the queue data, one stack argument: `queue_size` at +0, the entries behind +4 (20 bytes each); for an index below the size the dword at entry+0, else -1 — negative indices included. The entry at the index carries the dword, every other the dword 0x01010101.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_queue_data_get_job_counter.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (a, want) = super::row(line);
        let (size, idx, v) = (a[0], a[1], a[2]);
        let got = if idx < size { v } else { -1 };
        assert_eq!(got, want, "BuildQueueData::get_job_counter — row `{line}`");
        n += 1;
    }
    assert!(n >= 500, "only {n} rows");
    eprintln!("BuildQueueData::get_job_counter@00630690: {n} rows agree");
}
