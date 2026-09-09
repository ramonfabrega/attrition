/* Opt-in, one-match menu driver. Calls the original Start handler from the
 * setup modal's idle loop; no input events, simulation writes, or worker thread.
 * Reading/byte evidence and limitations: docs/audit/2026-09-09-autostart.md.
 */
IMPORT(void, ExitProcess, (u32));
IMPORT(void *, AddVectoredExceptionHandler, (u32, void *));
IMPORT(i32, ReadProcessMemory, (HANDLE, const void *, void *, u32, u32 *));
static i32 WINAPI auto_fault(u32 *pointers) {
    u32 *record = (u32 *)pointers[0], *ctx = (u32 *)pointers[1];
    if (record[0] != 0xc0000005) return 0;
    /* Windows x86 CONTEXT: Ebp 180, Eip 184, Esp 196. */
    u32 words[8], n=0;
    emit(K_INFO, 176, ctx[46], ctx[49], ctx[45], record[6], 0);
    if (ReadProcessMemory(GetCurrentProcess(), (void *)ctx[49], words, sizeof words, &n) && n==sizeof words) {
        emit(K_INFO, 177, words[0], words[1], words[2], words[3], 0);
        emit(K_INFO, 177, words[4], words[5], words[6], words[7], 1);
    }
    flush();
    return 0;
}
static u32 auto_menu_count, auto_ready, auto_started;
static void (*auto_idle_original)(void);

static i32 __thiscall auto_menu(void *self) {
    (void)self;
    u32 result = auto_menu_count++ == 0 ? 1 : 21; /* GAME_SOLO, GAME_QUIT */
    emit(K_INFO, 170, result, auto_menu_count, 0, 0, 0);
    flush();
    if (result == 21) ExitProcess(auto_started && g_frame >= 0 ? 0 : 6);
    return (i32)result;
}
static i32 __thiscall auto_submenu(void *self, void *name) {
    (void)name;
    return auto_menu(self);
}
static void __attribute__((used,noinline)) auto_modal_enter(void *self, i32 mode) {
    auto_ready = (u32)self;
    emit(K_INFO, 171, (u32)self, (u32)mode, 0, 0, 0);
}
/* set_modal receives a by-value std::function and a co-modal window after
 * mode. Preserve its complete original stack; a C wrapper taking just mode
 * silently truncates these arguments. EAX is the caller's original vtable. */
static void __attribute__((naked)) auto_modal(void) {
    __asm__ volatile(
        "pushfl\n\tpushal\n\t"
        "movl 24(%esp), %ecx\n\tmovl 40(%esp), %eax\n\t"
        "pushl %eax\n\tpushl %ecx\n\tcall _auto_modal_enter\n\t"
        "addl $8, %esp\n\tpopal\n\tpopfl\n\tjmp *0x3c0(%eax)"
    );
}
static void auto_idle(void) {
    auto_idle_original();
    if (!auto_ready || auto_started) return;
    /* Set before dispatch: validation may itself pump modal messages. */
    auto_started = 1;
    u32 current = *(u32 *)(g_base + 0x8ab3a0);
    if (current != auto_ready) {
        emit(K_INFO, 174, current, auto_ready, 0, 0, 0);
        flush();
        return;
    }
    u32 strings = *(u32 *)(g_base + 0x806378);
    u32 start = *(u32 *)(strings + 0x10) + 0x57d0;
    emit(K_INFO, 172, current, start, 0, 0, 0);
    flush();
    typedef void (__thiscall *Start)(void *, void *);
    ((Start)(g_base + 0x1c5b50))((void *)current, (void *)start);
    emit(K_INFO, 173, current, 0, 0, 0, 0);
    flush();
}
static void auto_branch(u32 rva, u32 target, u8 opcode) {
    u8 bytes[5];
    bytes[0] = opcode;
    *(u32 *)(bytes + 1) = target - (g_base + rva + 5);
    patch5((u8 *)(g_base + rva), bytes);
}
static i32 auto_different(const void *a, const void *b, u32 n) {
    const u8 *x=a, *y=b;
    for (u32 i=0; i<n; i++) if (x[i] != y[i]) return 1;
    return 0;
}
static void install_autostart(void) {
    AddVectoredExceptionHandler(1, (void *)auto_fault);
    static const u8 menu[] = {0x55,0x8b,0xec,0x64,0xa1,0,0,0,0};
    static const u8 idle[] = {0x55,0x8b,0xec,0x83,0xe4,0xf8};
    static const u8 modal[] = {0xff,0x90,0xc0,3,0,0};
    static const u8 current[] = {0x89,0x35,0xa0,0xb3,0xca,0};
    /* Validate every patch before modifying any of them. */
    if (g_cover || g_base != 0x400000 ||
        auto_different((void *)(g_base+0x19bf70), menu, sizeof menu) ||
        auto_different((void *)(g_base+0x19c030), menu, sizeof menu) ||
        auto_different((void *)(g_base+0x1994a0), idle, sizeof idle) ||
        auto_different((void *)(g_base+0x1c7e37), modal, sizeof modal) ||
        auto_different((void *)(g_base+0x1c7cd2), current, sizeof current)) {
        emit(K_INFO, 174, 1, g_cover, g_base, 0, 0);
        flush();
        return;
    }
    u8 *tramp = VirtualAlloc(0, 32, 0x3000, PAGE_READWRITE);
    if (!tramp) { emit(K_INFO, 174, 2, 0, 0, 0, 0); flush(); return; }
    memcpy(tramp, idle, sizeof idle);
    tramp[6] = 0xe9;
    *(u32 *)(tramp+7) = g_base+0x1994a6 - ((u32)tramp+11);
    u32 old;
    if (!VirtualProtect(tramp, 32, PAGE_EXECUTE_READ, &old)) {
        emit(K_INFO, 174, 3, 0, 0, 0, 0); flush(); return;
    }
    FlushInstructionCache(GetCurrentProcess(), tramp, 32);
    auto_idle_original = (void (*)(void))tramp;
    auto_branch(0x19bf70, (u32)auto_menu, 0xe9);
    auto_branch(0x19c030, (u32)auto_submenu, 0xe9);
    auto_branch(0x1994a0, (u32)auto_idle, 0xe9);
    auto_branch(0x1c7e37, (u32)auto_modal, 0xe8);
    *(u8 *)(g_base+0x1c7e3c) = 0x90;
    emit(K_INFO, 175, 1, 0, 0, 0, 0);
}
