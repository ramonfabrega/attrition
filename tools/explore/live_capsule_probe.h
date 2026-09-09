/* Opt-in, one live CommandPackage::clear capsule. Listing @0094c1c0:
 * a leaf with two writes and a singleton read, no stack locals or imports.
 * The return address is redirected to an exit witness; all GPRs and EFLAGS
 * are preserved. The captured stack contains that explicit stop address.
 */
#include "probe_code_hash.h"
#define CLEAR_CODE_HASH 0x19e6f1197d276c80ULL

typedef struct {
    u32 magic, version, entry, stop, frame, self, global, game, caller;
    u32 before_regs[9], after_regs[9]; /* pushad order, then EFLAGS; ESP normalized */
    u8 code[24], before[0x218], after[0x218];
    u32 seed_before, seed_after, ptr_before, ptr_after, stack_return;
} LiveCapsule;
static LiveCapsule capsule;
static volatile i32 capsule_claimed;
static u32 capsule_caller;

static void __cdecl capsule_enter(u32 *regs) {
    if (g_frame < 20 || g_frame > 35 || capsule_claimed) return;
    u8 *self = (u8 *)regs[6];
    if (!self || !*(u16 *)(self + 0x10)) return;
    if (!__sync_bool_compare_and_swap(&capsule_claimed, 0, 1)) return;
    capsule.frame = (u32)g_frame;
    capsule.self = (u32)self;
    capsule.ptr_before = *(u32 *)capsule.global;
    capsule.game = capsule.ptr_before;
    capsule.seed_before = *(u32 *)(capsule.game + 0x10);
    memcpy(capsule.before, self, sizeof capsule.before);
    memcpy(capsule.before_regs, regs, sizeof capsule.before_regs);
    capsule.before_regs[3] += 4; /* pushad saved ESP after pushfd */
    u32 *stack = (u32 *)capsule.before_regs[3];
    capsule.caller = capsule_caller = *stack;
    *stack = capsule.stop;
    capsule.stack_return = *stack;
}

static void __cdecl capsule_leave(u32 *regs) {
    memcpy(capsule.after_regs, regs, sizeof capsule.after_regs);
    capsule.after_regs[3] += 4;
    memcpy(capsule.after, (void *)capsule.self, sizeof capsule.after);
    capsule.ptr_after = *(u32 *)capsule.global;
    capsule.seed_after = *(u32 *)(capsule.game + 0x10);
    char path[320];
    path_join(path, "capsule.bin");
    HANDLE file = CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ, 0, 1,
                              FILE_ATTRIBUTE_NORMAL, 0); /* CREATE_NEW */
    u32 written = 0;
    i32 ok = file != INVALID_HANDLE && WriteFile(file, &capsule, sizeof capsule, &written, 0);
    if (file != INVALID_HANDLE) CloseHandle(file);
    emit(K_INFO, 120, ok && written == sizeof capsule ? 0 : 1,
         capsule.self, capsule.frame, written, sizeof capsule);
    flush();
}

static u32 capsule_callback(u8 *s, void *fn) {
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

static void install_capsule(void) {
    u8 *target = (u8 *)(g_base + 0x54c1c0u);
    if (g_base != 0x400000u || g_cover) {
        emit(K_INFO, 121, 1, g_base, (u32)g_cover, 0, 0); return;
    }
    unsigned long long actual_hash = probe_code_hash(target, 21);
    if (actual_hash != CLEAR_CODE_HASH) {
        emit(K_INFO, 121, 2, (u32)actual_hash, (u32)(actual_hash >> 32), 21, 0); return;
    }
    u8 *stub = VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!stub) { emit(K_INFO, 121, 3, 0, 0, 0, 0); return; }
    capsule.magic = 0x31504352u; capsule.version = 1;
    capsule.entry = (u32)target; capsule.stop = (u32)(stub+64);
    capsule.global = g_base + 0x8061ecu;
    memcpy(capsule.code, target, sizeof capsule.code);
    u32 n = capsule_callback(stub, (void *)capsule_enter);
    stub[n++] = 0xe9; *(u32 *)(stub+n) = (u32)(stub+128) - (u32)(stub+n+4);
    n = 64 + capsule_callback(stub+64, (void *)capsule_leave);
    stub[n++] = 0xff; stub[n++] = 0x25; *(u32 *)(stub+n) = (u32)&capsule_caller;
    memcpy(stub+128, target, 6);
    stub[134] = 0xe9; *(u32 *)(stub+135) = (u32)(target+6) - (u32)(stub+139);
    target[0] = 0xe9; *(u32 *)(target+1) = (u32)stub - (u32)(target+5); target[5] = 0x90;
    FlushInstructionCache(g_proc, stub, 4096);
    FlushInstructionCache(g_proc, target, 6);
}

/* Invoke the original leaf on an exact copy of the live, nonempty package.
 * The command manager keeps its original package for the normal turn pump.
 * This is a probe-created call with live-derived state, not a natural call. */
static void capsule_request_copy(u8 *package) {
    u32 copy[0x218 / 4];
    memcpy(copy, package, sizeof copy);
    typedef void(__thiscall *clear_fn)(void *);
    ((clear_fn)(g_base + 0x54c1c0u))(copy);
    emit(K_INFO, 122, 0, *(u16 *)(package+0x10), *(u16 *)((u8 *)copy+0x10), 0, 0);
    flush();
}
