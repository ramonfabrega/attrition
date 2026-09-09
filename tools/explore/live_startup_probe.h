/* Diagnostic call-site wrappers for WinMain's unconditional Media Foundation
 * startup/shutdown pair. The original imports and arguments remain intact.
 * INFO 180: site (1=startup, 2=shutdown), phase (0=enter, 1=return),
 * version, flags, result. Only these two calls are wrapped, not the whole IAT.
 */
typedef i32 (WINAPI *StartupMF)(u32, u32);
typedef i32 (WINAPI *ShutdownMF)(void);
static i32 WINAPI startup_mf(u32 version, u32 flags) {
    emit(K_INFO, 180, 1, 0, version, flags, 0); flush();
    i32 result = (*(StartupMF *)(g_base + 0x8a9f30))(version, flags);
    emit(K_INFO, 180, 1, 1, version, flags, (u32)result); flush();
    return result;
}
static i32 WINAPI shutdown_mf(void) {
    emit(K_INFO, 180, 2, 0, 0, 0, 0); flush();
    i32 result = (*(ShutdownMF *)(g_base + 0x8a9f2c))();
    emit(K_INFO, 180, 2, 1, 0, 0, (u32)result); flush();
    return result;
}
static void install_startup_probe(void) {
    /* Two indirect CALL operands, established from the installed listing and
     * delay-import directory. Do not replace delay-loader-managed IAT slots. */
    static const u8 startup_call[] = {0xff,0x15,0x30,0x9f,0xca,0};
    static const u8 shutdown_call[] = {0xff,0x15,0x2c,0x9f,0xca,0};
    if (auto_different((void *)(g_base+0x16096d), startup_call, 6) ||
        auto_different((void *)(g_base+0x16097a), shutdown_call, 6)) {
        emit(K_INFO,174,4,0,0,0,0); flush(); return;
    }
    auto_branch(0x16096d,(u32)startup_mf,0xe8);
    *(u8 *)(g_base+0x160972)=0x90;
    auto_branch(0x16097a,(u32)shutdown_mf,0xe8);
    *(u8 *)(g_base+0x16097f)=0x90;
}
