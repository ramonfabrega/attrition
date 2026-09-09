/* Opt-in natural Unit::update_order leaf witness. No stack writes in the
 * original call; the exit callback therefore cannot overwrite captured data. */
typedef struct {
    u32 magic, version, entry, stop, frame, self, head, node, caller;
    u32 before_regs[9], after_regs[9];
    u8 code[64], before[20], after[20];
    u32 head_before, head_after;
    u8 node_before[8], node_after[8];
    u32 stack_return;
} OrderCapsule;
static OrderCapsule order_capsule;
static volatile i32 order_capsule_claimed;
static u32 order_capsule_caller;

static void __cdecl order_capsule_enter(u32 *regs) {
    if (g_frame < 0 || g_frame > 35 || order_capsule_claimed) return;
    u8 *self = (u8 *)regs[6];
    if (!self) return;
    u8 *head = *(u8 **)(self+0xdc);
    if (!head) return;
    u8 *node = *(u8 **)(head+4);
    if (!node) return;
    if (!__sync_bool_compare_and_swap(&order_capsule_claimed, 0, 1)) return;
    order_capsule.frame = (u32)g_frame;
    order_capsule.self = (u32)self; order_capsule.head = (u32)head; order_capsule.node = (u32)node;
    memcpy(order_capsule.before, self+0xcc, 20);
    order_capsule.head_before = *(u32 *)(head+4);
    memcpy(order_capsule.node_before, node+8, 8);
    memcpy(order_capsule.before_regs, regs, sizeof order_capsule.before_regs);
    order_capsule.before_regs[3] += 4;
    u32 *stack = (u32 *)order_capsule.before_regs[3];
    order_capsule.caller = order_capsule_caller = *stack;
    *stack = order_capsule.stop;
    order_capsule.stack_return = *stack;
}

static void __cdecl order_capsule_leave(u32 *regs) {
    memcpy(order_capsule.after_regs, regs, sizeof order_capsule.after_regs);
    order_capsule.after_regs[3] += 4;
    memcpy(order_capsule.after, (u8 *)order_capsule.self+0xcc, 20);
    order_capsule.head_after = *(u32 *)(order_capsule.head+4);
    memcpy(order_capsule.node_after, (u8 *)order_capsule.node+8, 8);
    char path[320]; path_join(path, "order-capsule.bin");
    HANDLE file = CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ, 0, 1, FILE_ATTRIBUTE_NORMAL, 0);
    u32 written = 0;
    i32 ok = file != INVALID_HANDLE && WriteFile(file, &order_capsule, sizeof order_capsule, &written, 0);
    if (file != INVALID_HANDLE) CloseHandle(file);
    emit(K_INFO, 124, ok && written == sizeof order_capsule ? 0 : 1,
         order_capsule.self, order_capsule.frame, written, sizeof order_capsule);
    flush();
}

static u32 order_capsule_callback(u8 *s, void *fn) {
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

static void install_order_capsule(void) {
    const u8 signature[7] = {0x83,0xb9,0xdc,0,0,0,0};
    u8 *target = (u8 *)(g_base+0x2179d0u);
    if (g_base != 0x400000u || g_cover) {
        emit(K_INFO, 125, 1, g_base, (u32)g_cover, 0, 0); return;
    }
    for (u32 i=0; i<sizeof signature; i++) if (target[i] != signature[i]) {
        emit(K_INFO, 125, 2, i, target[i], signature[i], 0); return;
    }
    u8 *stub = VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!stub) { emit(K_INFO, 125, 3, 0, 0, 0, 0); return; }
    order_capsule.magic = 0x314f4352u; order_capsule.version = 1;
    order_capsule.entry = (u32)target; order_capsule.stop = (u32)(stub+64);
    memcpy(order_capsule.code, target, 64);
    u32 n = order_capsule_callback(stub, (void *)order_capsule_enter);
    stub[n++] = 0xe9; *(u32 *)(stub+n) = (u32)(stub+128)-(u32)(stub+n+4);
    n = 64+order_capsule_callback(stub+64, (void *)order_capsule_leave);
    stub[n++] = 0xff; stub[n++] = 0x25; *(u32 *)(stub+n) = (u32)&order_capsule_caller;
    memcpy(stub+128, target, 7);
    stub[135] = 0xe9; *(u32 *)(stub+136) = (u32)(target+7)-(u32)(stub+140);
    target[0] = 0xe9; *(u32 *)(target+1) = (u32)stub-(u32)(target+5);
    target[5] = target[6] = 0x90;
    FlushInstructionCache(g_proc, stub, 4096);
    FlushInstructionCache(g_proc, target, 7);
}
