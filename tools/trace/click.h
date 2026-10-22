/* The click lines (item 3437, `docs/GOLDEN.md` §144.3): a mouse click the
 * game takes from inside its own process, with no macOS cursor, focus or TCC
 * grant. Included by `tracer.c` after `emit` and `g_base`.
 *
 *   `@click <x> <y>`    a left click at the client point (x, y)
 *   `@rclick <x> <y>`   a right click there
 *   `@cursor <x> <y>`   the cursor moved there, no button
 *
 * **What the game reads of the mouse, and where.** The executable never
 * calls `GetCursorPos`: the window procedure belongs to the renderer DLL
 * that `GraphSys::init_system@0054abf0` loads (`d3dgl.dll`, the only module
 * importing it), and the exe sees the mouse through two doors only.
 *
 * 1. Events. `bighuge_init_real@00539630` hands the renderer
 *    `Window::message_handler` (VA 0xebe4dc, a `SysMessageHandler` whose
 *    vftable is 0xb64d18); the window procedure decodes a `WM_*BUTTON*` or
 *    `WM_MOUSEMOVE` lParam and calls the handler's slot: +0x14 `OnMouseMove
 *    (x, y)`, +0x20 `OnLButtonDown(dbl, x, y)`, +0x24 `OnLButtonUp(x, y)`,
 *    +0x28 `OnRButtonDown(dbl, x, y)`, +0x2c `OnRButtonUp(x, y)`, the middle
 *    and extra buttons at +0x18..+0x1c and +0x30..+0x34, the wheel at +0x44.
 *    Each divides the point by `graphsys->screen_scale_x/y` (+0x350, +0x354),
 *    finds the window under it (`Window::get_mouse_window`) and calls that
 *    window's `on_l_button_down` (+0x3f4) and the rest; the up handler's
 *    click test is the down's capture and the window's rectangle
 *    (`Window::on_l_button_up@00a54d40`), not a timer.
 * 2. A polled position. `graphsys` slot +0x38 returns a cached pair at
 *    +0x460/+0x464 (`d3dgl` 0x1000c4e0), and slot +0x34 refreshes it from
 *    `GetCursorPos`, `ScreenToClient` on the window at +0x45c and the same
 *    scale (`d3dgl` 0x1000c470). The refresh's one caller is
 *    `Window::update_cursor@00a502e0`; the getter's fifteen include
 *    `Window::get_mouse_pos@00a54170` (which of them a hover reads is not
 *    established).
 *
 * So the DLL drives the first door with the original handlers, called at the
 * top of the stanza's `do_frame` like every other `@` line, and pins the
 * second: the renderer object's vftable pointer is swapped for a copy whose
 * refresh writes the pinned point, and `message_handler`'s for a copy whose
 * mouse slots drop the window procedure's own events — a real cursor over
 * the window (the pool's resting one, Loop 3432) is an input nobody chose.
 * Posting `WM_*` messages instead would land on whichever pump turn ran
 * next and would leave the polled position at the real cursor.
 *
 * The pin is installed at the first `do_frame` when `rontrace.cmd` holds a
 * click line or `rontrace.cfg` a `cursor=X,Y` (the resting point; default
 * 700,300, `handdriver.sh`'s), and never otherwise: a capture without either
 * runs exactly as before. A click leaves the cursor where it clicked, as a
 * hand does. Points are client pixels; the pin refuses a scale other than
 * 1.0, so they are the game's own logical pixels too (1920 x 1080).
 *
 * Records: I_PIN once, a = frame, b = 0 installed or the refusal (1 no
 * renderer object, 2 its getter is not the shipped bytes, 3 nor its
 * refresh, 4 the scale is not 1.0, 5 `message_handler` is not on its
 * shipped vftable) | the renderer vftable entries copied << 8, c = x,
 * d = y, e = the x scale's bits. I_CLICK per line, a = frame, b = line
 * index | refusal << 16 (0 ran; 1 no pin; 5 the text did not parse),
 * c = x | y << 16, d = 0 cursor, 1 left, 2 right, e = the window
 * procedure's mouse events dropped so far. */

#define RVA_GRAPHSYS 0xa335a0u /* graphsys, VA 0xe335a0 (GraphSys *) */
#define RVA_MSG_HANDLER 0xabe4dcu /* Window::message_handler, VA 0xebe4dc */
#define RVA_MSG_VFTABLE 0x764d18u /* SysMessageHandler::vftable, VA 0xb64d18 */
#define GS_SCALE_X 0x350u
#define GS_SCALE_Y 0x354u
#define GS_CURSOR_X 0x460u
#define GS_CURSOR_Y 0x464u
#define GS_SLOT_REFRESH (0x34u / 4)
#define GS_SLOT_GET (0x38u / 4)
#define GS_VT_MAX 512
#define MSG_VT_SLOTS 20 /* +0x0 .. +0x4c */
#define FLOAT_ONE 0x3f800000u
#define RVA_MOUSEOVER 0x806230u /* MiscAccess::mouseover, VA 0xc06230 (MouseOver *) */
#define MOUSEOVER_TYPE 0x4u
#define MOUSEOVER_T 0x48u

static i32 g_pin_want, g_pin_state = -1; /* -1 not tried, 0 installed, > 0 refused */
static i32 g_pin_x = 700, g_pin_y = 300;
static u8 *g_gs;
static u32 g_gs_vt[GS_VT_MAX + 1]; /* [0] the RTTI locator, then the slots */
static u32 g_msg_vt[MSG_VT_SLOTS + 1];
static u32 g_msg_orig[MSG_VT_SLOTS];
static u32 g_mouse_dropped;

static void __thiscall pin_refresh(void *gs) {
    *(i32 *)((u8 *)gs + GS_CURSOR_X) = g_pin_x;
    *(i32 *)((u8 *)gs + GS_CURSOR_Y) = g_pin_y;
}
static void __thiscall drop2(void *self, i32 a, i32 b) {
    (void)self, (void)a, (void)b;
    g_mouse_dropped++;
}
static void __thiscall drop3(void *self, i32 a, i32 b, i32 c) {
    (void)self, (void)a, (void)b, (void)c;
    g_mouse_dropped++;
}

/* `d3dgl` 0x1000c4e0: `push ebp; mov ebp, esp; mov eax, [ebp+8]; mov edx,
 * [ecx+0x460]; mov [eax], edx` */
static const u8 GS_GET[14] = {0x55, 0x8b, 0xec, 0x8b, 0x45, 0x08, 0x8b, 0x91, 0x60, 0x04, 0x00, 0x00, 0x89, 0x10};
/* `d3dgl` 0x1000c470 at +0x0b: `xor eax, ebp; mov [ebp-4], eax; push esi;
 * lea eax, [ebp-0xc]; mov esi, ecx; push eax; call [GetCursorPos]` */
static const u8 GS_REFRESH[14] = {0x33, 0xc5, 0x89, 0x45, 0xfc, 0x56, 0x8d, 0x45, 0xf4, 0x8b, 0xf1, 0x50, 0xff, 0x15};

static int bytes_are(const u8 *p, const u8 *want, u32 n) {
    for (u32 i = 0; i < n; i++)
        if (p[i] != want[i]) return 0;
    return 1;
}

/* The module holding `fn`: module bases are 64 KiB aligned and an image is
 * mapped whole from its headers, so the walk down stays inside it. */
static u8 *module_of(u32 fn) {
    u32 b = fn & ~0xffffu;
    for (u32 i = 0; i < 256 && b; i++, b -= 0x10000)
        if (*(u16 *)b == 0x5a4d) return (u8 *)b;
    return 0;
}

static void pin_install(i32 frame) {
    u32 scale = 0, n = 0;
    i32 why = 0;
    g_gs = *(u8 **)(g_base + RVA_GRAPHSYS);
    u32 *vt = g_gs ? *(u32 **)g_gs : 0;
    u32 **msg = (u32 **)(g_base + RVA_MSG_HANDLER);
    if (!vt) why = 1;
    else if (!bytes_are((const u8 *)vt[GS_SLOT_GET], GS_GET, sizeof GS_GET)) why = 2;
    else if (!bytes_are((const u8 *)vt[GS_SLOT_REFRESH] + 0x0b, GS_REFRESH, sizeof GS_REFRESH)) why = 3;
    else if ((scale = *(u32 *)(g_gs + GS_SCALE_X)) != FLOAT_ONE || *(u32 *)(g_gs + GS_SCALE_Y) != FLOAT_ONE) why = 4;
    else if ((u32)*msg != g_base + RVA_MSG_VFTABLE) why = 5;
    if (!why) {
        /* The renderer's own code calls its slots through the same pointer,
         * so the copy is the whole vftable: every entry inside its module. */
        u8 *mod = module_of(vt[GS_SLOT_GET]);
        u32 lo = (u32)mod, hi = mod ? lo + *(u32 *)(mod + *(u32 *)(mod + 0x3c) + 0x50) : 0;
        while (n < GS_VT_MAX && vt[n] >= lo && vt[n] < hi) n++;
        g_gs_vt[0] = vt[-1];
        memcpy(&g_gs_vt[1], vt, n * 4);
        g_gs_vt[1 + GS_SLOT_REFRESH] = (u32)(void *)pin_refresh;
        pin_refresh(g_gs);
        *(u32 **)g_gs = &g_gs_vt[1];
        u32 *mv = *msg;
        g_msg_vt[0] = mv[-1];
        memcpy(&g_msg_vt[1], mv, MSG_VT_SLOTS * 4);
        memcpy(g_msg_orig, mv, MSG_VT_SLOTS * 4);
        static const u8 two[] = {0x14, 0x1c, 0x24, 0x2c}, three[] = {0x18, 0x20, 0x28, 0x30, 0x34, 0x44};
        for (u32 i = 0; i < sizeof two; i++) g_msg_vt[1 + two[i] / 4] = (u32)(void *)drop2;
        for (u32 i = 0; i < sizeof three; i++) g_msg_vt[1 + three[i] / 4] = (u32)(void *)drop3;
        *msg = &g_msg_vt[1];
    }
    g_pin_state = why;
    emit(K_INFO, I_PIN, (u32)frame, (u32)why | n << 8, (u32)g_pin_x, (u32)g_pin_y, scale);
}

static int pin_wanted_by(const u16 *text) {
    const u16 *t = text;
    return issue_verb(&t, "click ") || issue_verb(&t, "rclick ") || issue_verb(&t, "cursor ");
}

/* An `@click`, `@rclick` or `@cursor` line; `t` is past the verb. */
static void click_line(i32 frame, u32 idx, const u16 *t, i32 kind) {
    i32 x = 0, y = 0, why = 0;
    if (!issue_int(&t, &x) || !issue_int(&t, &y) || x < 0 || y < 0 || x > 0xffff || y > 0xffff) why = 5;
    else if (g_pin_state != 0) why = 1;
    if (!why) {
        typedef void(__thiscall *h2_fn)(void *, i32, i32);
        typedef void(__thiscall *h3_fn)(void *, i32, i32, i32);
        void *h = (void *)(g_base + RVA_MSG_HANDLER);
        g_pin_x = x, g_pin_y = y;
        g_hover_moved = 1;
        pin_refresh(g_gs);
        ((h2_fn)g_msg_orig[0x14 / 4])(h, x, y);
        if (kind) {
            ((h3_fn)g_msg_orig[(kind == 1 ? 0x20 : 0x28) / 4])(h, 0, x, y);
            ((h2_fn)g_msg_orig[(kind == 1 ? 0x24 : 0x2c) / 4])(h, x, y);
        }
    }
    emit(K_INFO, I_CLICK, (u32)frame, idx | ((u32)why << 16), (u32)x | (u32)y << 16, (u32)kind, g_mouse_dropped);
}
