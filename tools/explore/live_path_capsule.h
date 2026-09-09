/* Natural path-segment deletion, restricted to no suspended search.
 * The original uses stack scratch. Only semantic memory is captured here;
 * the strict replay tracks scratch initialization separately. */
typedef struct {
    u32 magic, version, entry, stop, frame, self, path, count, caller;
    u32 before_regs[9], after_regs[9];
    u8 code[80], clear_head[20], clear_tail[4];
    u8 before[12], after[12];
    u32 open_before, open_after;
    u8 path_before[1024], path_after[1024];
    u32 stack_return;
} PathCapsule;
static PathCapsule path_capsule;
static volatile i32 path_capsule_claimed;
static u32 path_capsule_caller;

static void __cdecl path_capsule_enter(u32 *regs) {
    if (g_frame < 0 || g_frame > 240 || path_capsule_claimed) return;
    u8 *self = (u8 *)regs[6];
    if (!self || *(u32 *)(self+0x104)) return;
    i32 count = *(i32 *)(self+0xc0);
    u8 *path = *(u8 **)(self+0xb8);
    if (count <= 0 || count > 64 || !path) return;
    if (!__sync_bool_compare_and_swap(&path_capsule_claimed, 0, 1)) return;
    path_capsule.frame = (u32)g_frame; path_capsule.self = (u32)self;
    path_capsule.path = (u32)path; path_capsule.count = (u32)count;
    memcpy(path_capsule.before, self+0xb8, 12);
    path_capsule.open_before = *(u32 *)(self+0x104);
    memcpy(path_capsule.path_before, path, (u32)count*16);
    memcpy(path_capsule.before_regs, regs, sizeof path_capsule.before_regs);
    path_capsule.before_regs[3] += 4;
    u32 *stack = (u32 *)path_capsule.before_regs[3];
    path_capsule.caller = path_capsule_caller = *stack;
    *stack = path_capsule.stop; path_capsule.stack_return = *stack;
}

static void __cdecl path_capsule_leave(u32 *regs) {
    memcpy(path_capsule.after_regs, regs, sizeof path_capsule.after_regs);
    path_capsule.after_regs[3] += 4;
    memcpy(path_capsule.after, (u8 *)path_capsule.self+0xb8, 12);
    path_capsule.open_after = *(u32 *)(path_capsule.self+0x104);
    memcpy(path_capsule.path_after, (void *)path_capsule.path, path_capsule.count*16);
    char path[320]; path_join(path, "path-capsule.bin");
    HANDLE file = CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ, 0, 1, FILE_ATTRIBUTE_NORMAL, 0);
    u32 written = 0;
    i32 ok = file != INVALID_HANDLE && WriteFile(file, &path_capsule, sizeof path_capsule, &written, 0);
    if (file != INVALID_HANDLE) CloseHandle(file);
    emit(K_INFO, 126, ok && written == sizeof path_capsule ? 0 : 1,
         path_capsule.self, path_capsule.frame, written, sizeof path_capsule);
    flush();
}

static u32 path_capsule_callback(u8 *s, void *fn) {
    u32 n = 0;
    s[n++] = 0x9c; /* pushfd */
    s[n++] = 0x60; /* pushad */
    s[n++] = 0x54; /* push esp */
    s[n++] = 0xb8; *(u32 *)(s+n) = (u32)fn; n += 4;
    s[n++] = 0xff; s[n++] = 0xd0; /* call eax */
    s[n++] = 0x83; s[n++] = 0xc4; s[n++] = 4;
    s[n++] = 0x61; s[n++] = 0x9d; /* popad; popfd */
    return n;
}

static void install_path_capsule(void) {
    const u8 signature[6] = {0x55,0x8b,0xec,0x83,0xe4,0xf8};
    u8 *target = (u8 *)(g_base+0x1e31d0u);
    if (g_base != 0x400000u || g_cover) {
        emit(K_INFO, 127, 1, g_base, (u32)g_cover, 0, 0); return;
    }
    for (u32 i=0; i<sizeof signature; i++) if (target[i] != signature[i]) {
        emit(K_INFO, 127, 2, i, target[i], signature[i], 0); return;
    }
    u8 *stub = VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!stub) { emit(K_INFO, 127, 3, 0, 0, 0, 0); return; }
    path_capsule.magic = 0x31504350u; path_capsule.version = 1;
    path_capsule.entry = (u32)target; path_capsule.stop = (u32)(stub+64);
    memcpy(path_capsule.code, target, 80);
    memcpy(path_capsule.clear_head, (void *)(g_base+0x1e3920u), 20);
    memcpy(path_capsule.clear_tail, (void *)(g_base+0x1e3bc0u), 4);
    u32 n = path_capsule_callback(stub, (void *)path_capsule_enter);
    stub[n++] = 0xe9; *(u32 *)(stub+n) = (u32)(stub+128)-(u32)(stub+n+4);
    n = 64+path_capsule_callback(stub+64, (void *)path_capsule_leave);
    stub[n++] = 0xff; stub[n++] = 0x25; *(u32 *)(stub+n) = (u32)&path_capsule_caller;
    memcpy(stub+128, target, 6);
    stub[134] = 0xe9; *(u32 *)(stub+135) = (u32)(target+6)-(u32)(stub+139);
    target[0] = 0xe9; *(u32 *)(target+1) = (u32)stub-(u32)(target+5); target[5] = 0x90;
    FlushInstructionCache(g_proc, stub, 4096);
    FlushInstructionCache(g_proc, target, 6);
}
