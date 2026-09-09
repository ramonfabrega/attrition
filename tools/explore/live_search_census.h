/* Metadata only, at a natural astar_path suspension return. No graph mutation.
 * Layout evidence: docs/FORMATS.md, suspended-search census. */
IMPORT(i32, ReadProcessMemory, (HANDLE, const void *, void *, u32, u32 *));
static u32 search_census_count;

static i32 census_read(u32 sequence, u32 address, void *out, u32 size) {
    u32 copied = 0;
    i32 ok = ReadProcessMemory(g_proc, (void *)address, out, size, &copied);
    if (!ok || copied != size) {
        emit(K_INFO, 133, sequence, address, size, copied, (u32)ok);
        return 0;
    }
    return 1;
}

static void census_suspension(void) {
    u32 sequence = ++search_census_count;
    if (sequence > 64) {
        if (sequence == 65) emit(K_INFO, 135, sequence, 64, 0, 0, 0);
        return;
    }
    u32 unit, modes[2], pointers[5];
    if (!census_read(sequence, 0xe85e94u, &unit, 4) ||
        !census_read(sequence, 0xe85ec0u, modes, 8)) return;
    emit(K_INFO, 130, sequence, unit, modes[0], modes[1], 0);
    if (!census_read(sequence, unit+0x104u, pointers, sizeof pointers)) return;
    for (u32 i = 0; i < 5; i++) {
        u32 header[4] = {0, 0, 0, 0};
        if (pointers[i] && !census_read(sequence, pointers[i], header, sizeof header)) return;
        emit(K_INFO, 131, sequence, i, pointers[i], header[2], header[3]);
    }
    const u32 pools[7] = {0xc8d810, 0xc8d950, 0xc8d860, 0xc8d880, 0xc8d9a0,
                          0xc8d9b0, 0xc8da70};
    for (u32 i = 0; i < 7; i++) {
        u32 header[3];
        if (!census_read(sequence, pools[i], header, sizeof header)) return;
        emit(K_INFO, 132, sequence, i, header[0], header[1], header[2]);
    }
    emit(K_INFO, 134, sequence, 5, 7, 0, 0);
}
