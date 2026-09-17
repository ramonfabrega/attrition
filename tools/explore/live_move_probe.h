/* Opt-in live experiment, compiled only with RON_COMMAND_PROBE.
 * See command_oracle.py and the command-loop audit for address/layout evidence.
 * Single owner-zero captain, one MoveTo at frame 20. The regular turn pump
 * processes the resulting package; this never calls process_turn itself.
 */
static void probe_move(i32 frame) {
    static int attempted;
    u8 *slots = *(u8 **)(g_base + 0x80aec0u);
    if (!slots || frame < 19 || frame > 35) return;
    u8 *unit = ((u8 **)slots)[1];
    if (!unit) return;
    emit(K_INFO, 101, (u32)frame, *(u32 *)(unit + 0x10) ^ 0x63637u, *(u32 *)(unit + 0x14) ^ 0x63637u,
         *(u32 *)(unit + 0x70), *(u32 *)(unit + 0x74));
    if (frame != 20 || attempted) return;
    attempted = 1;
    u8 *package = (u8 *)(g_base + 0xa8ff88u);
    u32 before = *(u16 *)(package + 0x10);
    u8 *console = *(u8 **)(g_base + 0x806210u);
    if (!console || *(i32 *)(console + 0x2a0) != 0 || !(unit[8] & 1) ||
        unit[9] != 0 || *(u16 *)(unit + 0xa) != 1 ||
        !(*(u16 *)(unit + 0x8e) & 0x8000) || before > 485) {
        emit(K_INFO, 100, 1, before, 0, 0, 0);
        return;
    }
    const u8 signature[] = {0x55, 0x8b, 0xec, 0x83, 0xec, 0x18, 0xb9, 0x60, 0xff, 0xe8, 0x00};
    for (u32 i = 0; i < sizeof signature; i++) {
        if (*(u8 *)(g_base + 0x541720u + i) != signature[i]) {
            emit(K_INFO, 100, 2, i, 0, 0, 0);
            return;
        }
    }
    u32 group[0x9d0 / 4];
    memset(group, 0, sizeof group);
    group[3] = 1;
    *(u16 *)((u8 *)group + 0x8cc) = 1;
    i32 x = (i32)(*(u32 *)(unit + 0x10) ^ 0x63637u) + 2560;
    i32 y = (i32)(*(u32 *)(unit + 0x14) ^ 0x63637u);
    typedef void(__thiscall *issue_fn)(void *, void *, i32, i32, i32, i32,
                                      i32, i32, i32, i32, i32);
    ((issue_fn)(g_base + 0x541720u))((void *)(g_base + 0xa8ff60u),
                                   group, x, y, 2, 0, 0, 0, 0, 0, 0);
    emit(K_INFO, 100, 0, before, *(u16 *)(package + 0x10), (u32)x, (u32)y);
    flush();
#ifdef RON_CAPSULE_PROBE
    capsule_request_copy(package);
#endif
}
