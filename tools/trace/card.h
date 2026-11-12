/* The card probe (item 4165): what the original hands the device at each draw.
 *
 * Built only under `RON_CARD_PROBE`. The executable never calls Direct3D. It
 * calls `graphsys`, which is `d3dgl.dll`'s `D3D11GLSys`, and that DLL turns the
 * fixed-function state into a pixel shader permutation and four constant
 * buffers (`docs/RENDERER.md` §5.3, "The device's state"). So the whole state of
 * a draw is in three places at `D3D11GLSys::render`'s entry, and this probe
 * copies all three, per draw, over a window of sim-frames:
 *
 *   - `GraphSys::current_state` (`this +0x190`, the `RenderState`, 0x168 bytes):
 *     enables, tfactor, blend, alpha test, both stages' ops and arguments;
 *   - `D3D11Context`'s shadows of the constant buffers (`this +0x4f0 +0x70`,
 *     `PerScene`, `PerObject`, `PerMaterial`, `LightData`, 0x250 bytes) and
 *     `mEnableLight` (`+0x340`): world, ambient, material, texture factor,
 *     the light;
 *   - `D3D11GLGraphicCard` (`d3d_card`): `mShaderOptions` (the permutation's
 *     defines), `mActivePixelShader`, and stages 0 and 1's textures, each named
 *     by its `TextureCacheEntry::name` read through `ReadProcessMemory`.
 *
 * `begin_drawing` and `end_drawing` bound a pass. The addresses are d3dgl's
 * preferred-base VAs (its own map, `sbl/d3dgl.map`); the module's load base is
 * asked of the loader and each site's prologue is checked before a byte is
 * written. Nothing is installed until the first `Game::do_frame`, by when
 * `GraphSys::init_system` has loaded the DLL.
 *
 * Records are `K_CARD` (40), `a` the subtype:
 *   0 draw:  b site (0 indexed, 1 plain), c draw in pass, d pass, e caller, f this
 *   1 args:  b..f the call's stack arguments 0..4 (args_ok in the draw's high bit)
 *   2 state: b byte offset, c..f four dwords of `current_state`
 *   3 cbuf:  b byte offset from `mSceneData`, c..f four dwords
 *   4 card:  b `mShaderOptions` count, c `mActivePixelShader`, d tex0, e tex1, f mEnableLight
 *   5 opts:  b index, c..f four option values
 *   6 name:  b stage, c byte offset, d..f twelve bytes of the entry's name
 *   8 begin / 9 end of a pass: b pass, c draws in it
 *   10 installed: b d3dgl base, c sites hooked, d sites refused
 */
#define K_CARD 40
#ifndef RON_STATE_FRAME /* live_frame_snapshot.h declares it there */
IMPORT(i32, ReadProcessMemory, (HANDLE, const void *, void *, u32, u32 *));
#endif

#ifndef RON_CARD_LO
#define RON_CARD_LO 3260
#endif
#ifndef RON_CARD_HI
#define RON_CARD_HI 3300
#endif
#ifndef RON_CARD_PASSES
#define RON_CARD_PASSES 8
#endif

#define CARD_PREF 0x10000000u
#define CARD_RENDER_IB 0x10023ac0u /* D3D11GLSys::render(int, int, ib*, PrimitiveType, int) */
#define CARD_RENDER 0x10023d30u /* D3D11GLSys::render(int, int, PrimitiveType, int) */
#define CARD_BEGIN 0x10020f10u /* D3D11GLSys::begin_drawing */
#define CARD_END 0x10022f40u /* D3D11GLSys::end_drawing */
#define CARD_D3D_CARD 0x100fb39cu /* d3d_card, D3D11GLGraphicCard * */

static u32 g_card_base;
static i32 g_card_state; /* 0 not tried, 1 installed, 2 refused */
static i32 g_card_pass = -1; /* the pass being logged, or -1 */
static i32 g_card_npass; /* passes logged */
static u32 g_card_draw; /* draws in the current pass */

static void card_dump(u32 sub, const u8 *p, u32 n, u32 at) {
    for (u32 o = 0; o < n; o += 16) {
        const u32 *w = (const u32 *)(p + o);
        emit(K_CARD, sub, at + o, w[0], w[1], w[2], w[3]);
    }
}

static u32 card_rd(u32 addr) {
    u32 v = 0, got = 0;
    if (addr < 0x10000u) return 0;
    if (!ReadProcessMemory(g_proc, (void *)addr, &v, 4, &got) || got != 4) return 0;
    return v;
}

static void card_name(u32 stage, u32 tex) {
    /* D3D11GLTexture +0x18 mEntry -> TextureCacheEntry +0x10 name -> String +0 data */
    u32 entry = card_rd(tex + 0x18);
    u32 data = entry ? card_rd(entry + 0x10) : 0;
    u8 buf[96];
    u32 got = 0;
    memset(buf, 0, sizeof buf);
    if (data >= 0x10000u) ReadProcessMemory(g_proc, (void *)data, buf, sizeof buf, &got);
    for (u32 o = 0; o < sizeof buf; o += 12) {
        const u32 *w = (const u32 *)(buf + o);
        emit(K_CARD, 6, stage, o, w[0], w[1], w[2]);
    }
}

/* The hook stub's observer: (kind, ecx, ebp, caller, arg0). The stub pushed,
 * above its five arguments, the packed flags, edx, ecx, eax and then the
 * original return address, so the call's own arguments start five dwords past
 * `arg0`'s slot; the word before them must be `caller`, which is checked. */
static void card_on_hook(u32 kind, u32 ecx, u32 ebp, u32 caller, u32 arg0) {
    (void)ebp;
    if (kind == 1 || kind == 2) { /* begin / end */
        if (kind == 1) {
            if (g_card_pass < 0 && g_card_npass < RON_CARD_PASSES && g_frame >= RON_CARD_LO &&
                g_frame <= RON_CARD_HI) {
                g_card_pass = g_card_npass++;
                g_card_draw = 0;
                emit(K_CARD, 8, (u32)g_card_pass, 0, caller, ecx, 0);
            }
        } else if (g_card_pass >= 0) {
            emit(K_CARD, 9, (u32)g_card_pass, g_card_draw, caller, ecx, 0);
            g_card_pass = -1;
            flush();
        }
        return;
    }
    if (g_card_pass < 0) return;
    u32 site = kind == 3 ? 0 : 1;
    u32 *orig = (u32 *)&arg0 + 5;
    u32 ok = orig[-1] == caller;
    emit(K_CARD, 0, site | (ok << 31), g_card_draw, (u32)g_card_pass, caller, ecx);
    emit(K_CARD, 1, orig[0], orig[1], orig[2], orig[3], site == 0 ? orig[4] : 0);
    card_dump(2, (const u8 *)(ecx + 0x190), 0x168, 0);
    card_dump(3, (const u8 *)(ecx + 0x4f0 + 0x70), 0x250, 0);
    u32 card = *(u32 *)(g_card_base + CARD_D3D_CARD - CARD_PREF);
    u32 nopt = 0, opts = 0, ps = 0, t0 = 0, t1 = 0;
    if (card) {
        nopt = *(u32 *)(card + 0x390 + 4);
        opts = *(u32 *)(card + 0x390 + 0x10);
        ps = *(u32 *)(card + 0x328);
        t0 = *(u32 *)(card + 0x26c);
        t1 = *(u32 *)(card + 0x270);
    }
    emit(K_CARD, 4, nopt, ps, t0, t1, *(u8 *)(ecx + 0x4f0 + 0x340));
    if (opts && nopt <= 64)
        for (u32 i = 0; i < nopt; i += 4) {
            u32 v[4] = {0, 0, 0, 0};
            for (u32 j = 0; j < 4 && i + j < nopt; j++) v[j] = ((u32 *)opts)[i + j];
            emit(K_CARD, 5, i, v[0], v[1], v[2], v[3]);
        }
    if (t0) card_name(0, t0);
    if (t1) card_name(1, t1);
    g_card_draw++;
}

typedef struct {
    u32 va, len, kind;
    u8 expect[10];
    u32 reloc_at; /* offset of an absolute immediate in `expect`, or 0 */
} CardSite;

static const CardSite CARD_SITES[] = {
    /* begin_drawing: push ebp; mov ebp,esp; sub esp,0x14 */
    {CARD_BEGIN, 6, 1, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x14}, 0},
    /* end_drawing: push ebp; mov ebp,esp; and esp,-8 */
    {CARD_END, 6, 2, {0x55, 0x8b, 0xec, 0x83, 0xe4, 0xf8}, 0},
    /* render(ib): push ebp; mov ebp,esp; push -1; push 0x100bcb80 (relocated with the module) */
    {CARD_RENDER_IB, 10, 3, {0x55, 0x8b, 0xec, 0x6a, 0xff, 0x68, 0x80, 0xcb, 0x0b, 0x10}, 6},
    /* render: push ebp; mov ebp,esp; sub esp,0x1c */
    {CARD_RENDER, 6, 4, {0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c}, 0},
};
#define NCARD (sizeof(CARD_SITES) / sizeof(CARD_SITES[0]))

static void card_install(void) {
    g_card_state = 2;
    g_card_base = (u32)GetModuleHandleA("d3dgl.dll");
    if (!g_card_base) {
        emit(K_CARD, 10, 0, 0, NCARD, 0, 0);
        return;
    }
    u8 *page = (u8 *)VirtualAlloc(0, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if (!page) return;
    u32 used = 0, hooked = 0, refused = 0;
    for (u32 i = 0; i < NCARD; i++) {
        const CardSite *c = &CARD_SITES[i];
        u8 want[10];
        memcpy(want, c->expect, c->len);
        if (c->reloc_at) *(u32 *)(want + c->reloc_at) += g_card_base - CARD_PREF;
        u8 *t = (u8 *)(g_card_base + c->va - CARD_PREF);
        int ok = 1;
        for (u32 j = 0; j < c->len; j++)
            if (t[j] != want[j]) ok = 0;
        u32 old;
        if (!ok || !VirtualProtect(t, 16, PAGE_EXECUTE_READWRITE, &old)) {
            emit(K_INFO, I_HOOK_MISMATCH, c->va, *(u32 *)t, *(u32 *)(t + 4), *(u32 *)(t + 8), 0);
            refused++;
            continue;
        }
        u8 *stub = page + used;
        used += (build_hook_stub(stub, (u32)stub, (u32)t, c->kind, (u32)(void *)card_on_hook, t,
                                 c->len) +
                 15) &
                ~15u;
        t[0] = 0xE9;
        *(u32 *)(t + 1) = (u32)stub - ((u32)t + 5);
        for (u32 j = 5; j < c->len; j++) t[j] = 0xCC;
        VirtualProtect(t, 16, old, &old);
        FlushInstructionCache(g_proc, t, 16);
        hooked++;
    }
    g_card_state = refused ? 2 : 1;
    emit(K_CARD, 10, g_card_base, hooked, refused, RON_CARD_LO, RON_CARD_HI);
    flush();
}

/* From the frame hook: install once, at the first `Game::do_frame`. */
static void card_tick(void) {
    if (!g_card_state) card_install();
}
