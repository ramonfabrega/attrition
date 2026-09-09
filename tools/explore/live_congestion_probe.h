/* Opt-in scenario driver. Orders enter the original command manager; no search
 * state or limit is changed. See the congestion-probe audit for evidence. */
IMPORT(i32, ReadProcessMemory, (HANDLE, const void *, void *, u32, u32 *));
#define CONGESTION_SLOTS 512
static u16 congestion_uid[CONGESTION_SLOTS];
static u8 congestion_seen[CONGESTION_SLOTS];
static u16 congestion_ids[2][32], congestion_uids[2][32];
static u32 congestion_counts[2];
static i32 congestion_x, congestion_y;
static int congestion_ready, congestion_failed;

static int congestion_fail(u32 why, u32 address) {
    emit(K_INFO, 144, why, address, 0, 0, 0);
    congestion_failed = 1;
    return 0;
}
static int congestion_read(u32 address, void *out, u32 size) {
    u32 copied = 0;
    if (!ReadProcessMemory(g_proc, (void *)address, out, size, &copied) || copied != size)
        return congestion_fail(1, address);
    return 1;
}
static int congestion_registry(u32 *slots, u32 *count) {
    u32 h[4];
    if (!congestion_read(g_base+0x80aeb4u, h, sizeof h)) return 0;
    if (h[0] > CONGESTION_SLOTS || h[0] > h[1] || h[1] > 32768 || !h[3])
        return congestion_fail(2, h[0]);
    *count = h[0];
    return congestion_read(h[3], slots, *count*4);
}
static int congestion_captain(const u8 *unit, u32 id) {
    return (unit[8]&1) && unit[9] == 0 && *(const u16 *)(unit+0xa) == id &&
           (*(const u16 *)(unit+0x8e)&0x8000);
}
static void probe_congestion(i32 frame) {
    if (congestion_failed) return;
    /* One match per process: never reuse roster counts after a frame reset. */
    if (frame == 149 && congestion_ready) { congestion_fail(12, (u32)frame); return; }
    if (frame == 149 || frame == 210) {
        static u32 slots[CONGESTION_SLOTS];
        u32 count;
        if (g_base != 0x400000u || !congestion_registry(slots, &count)) return;
        i32 sx = 0, sy = 0;
        u32 total = 0;
        for (u32 i = 0; i < count; i++) {
            if (!slots[i]) continue;
            u32 words[0x90/4];
            u8 *unit = (u8 *)words;
            if (!congestion_read(slots[i], unit, sizeof words)) return;
            if (!(unit[8]&1)) continue;
            u16 uid = *(u16 *)(unit+0x30);
            if (frame == 149) {
                congestion_seen[i] = 1; congestion_uid[i] = uid;
                continue;
            }
            if ((congestion_seen[i] && congestion_uid[i] == uid) ||
                !congestion_captain(unit, i)) continue;
            if (total == 64) { congestion_fail(3, total); return; }
            i32 x = (i32)(words[4]^0x63637u), y = (i32)(words[5]^0x63637u);
            if (x < 0 || x > 1000000 || y < 0 || y > 1000000) {
                congestion_fail(4, i); return;
            }
            u32 group = total & 1, n = congestion_counts[group]++;
            congestion_ids[group][n] = (u16)i; congestion_uids[group][n] = uid;
            sx += x; sy += y; total++;
            emit(K_INFO, 140, group, i, uid, (u32)x, (u32)y);
        }
        if (frame == 149) { congestion_ready = 1; return; }
        if (!congestion_ready || total < 2) { congestion_fail(5, total); return; }
        congestion_x = sx/(i32)total; congestion_y = sy/(i32)total;
        congestion_ready = 2;
        emit(K_INFO, 141, congestion_counts[0], congestion_counts[1],
             (u32)congestion_x, (u32)congestion_y, count);
    }
    if (frame < 220 || frame > 1021 || (frame-220)%200 > 1 || congestion_ready != 2) return;
    u32 side = (u32)(frame-220)%200, n = congestion_counts[side];
    static u32 slots[CONGESTION_SLOTS];
    u32 count, console;
    if (!congestion_registry(slots, &count) ||
        !congestion_read(g_base+0x806210u, &console, 4)) return;
    i32 owner;
    if (!console || !congestion_read(console+0x2a0, &owner, 4)) return;
    if (owner != 0) { congestion_fail(6, (u32)owner); return; }
    for (u32 j = 0; j < n; j++) {
        u32 id = congestion_ids[side][j], words[0x90/4];
        if (id >= count || !slots[id]) { congestion_fail(7, id); return; }
        if (!congestion_read(slots[id], words, sizeof words)) return;
        u8 *unit = (u8 *)words;
        if (!congestion_captain(unit, id) || *(u16 *)(unit+0x30) != congestion_uids[side][j]) {
            congestion_fail(8, id); return;
        }
    }
    const u8 signature[] = {0x55,0x8b,0xec,0x83,0xec,0x18,0xb9,0x60,0xff,0xe8,0x00};
    u8 actual[sizeof signature];
    if (!congestion_read(g_base+0x541720u, actual, sizeof actual)) return;
    for (u32 i = 0; i < sizeof signature; i++)
        if (actual[i] != signature[i]) { congestion_fail(9, i); return; }
    u8 *package = (u8 *)(g_base+0xa8ff88u);
    u16 before;
    if (!congestion_read((u32)package+0x10, &before, 2)) return;
    /* Worst case: fresh group (3 + 2*n bytes), 22-byte command, 2 padding
     * bytes reserved for each native writer. Never rely on group elision. */
    if (before > 512-(29+2*n)) { congestion_fail(10, before); return; }
    static u32 group[0x9d0/4];
    memset(group, 0, sizeof group); group[3] = n;
    for (u32 j = 0; j < n; j++) *(u16 *)((u8 *)group+0x8cc+2*j) = congestion_ids[side][j];
    i32 direction = ((side + (u32)(frame-220)/200)&1) ? 1 : -1;
    i32 x = congestion_x+direction*1536, y = congestion_y;
    typedef void(__thiscall *issue_fn)(void *, void *, i32, i32, i32, i32,
                                      i32, i32, i32, i32, i32);
    ((issue_fn)(g_base+0x541720u))((void *)(g_base+0xa8ff60u), group, x,y,2,0,0,0,0,0,0);
    u16 after;
    if (!congestion_read((u32)package+0x10, &after, 2)) return;
    emit(K_INFO, 142, side, n, before, after, (u32)frame);
    emit(K_INFO, 143, side, (u32)x, (u32)y, 0, 0);
    if (after <= before || after > 512) congestion_fail(11, after);
    flush();
}
