/* The key lines (item 3442, `docs/GOLDEN.md` §146): a keystroke the game
 * takes from inside its own process, with no macOS keyboard, focus or TCC
 * grant. Included by `tracer.c` after `click.h`.
 *
 *   `@key <vk>`    a key pressed and released: `OnKey(vk, 1)`, `OnKey(vk, 0)`
 *   `@char <c>`    the `WM_CHAR` a printable key's press adds: `OnChar(c)`
 *
 * **What the game reads of the keyboard, and where.** As with the mouse
 * (`click.h`), the window procedure is `d3dgl.dll`'s and the executable sees
 * the key through `Window::message_handler`'s slots:
 *
 * - `WM_KEYDOWN` reaches `d3dgl` 0x10012750, which folds the numpad's digits
 *   onto '0'..'9' (0x60..0x69 less 0x30) and 0x6e onto 0xbe, and calls slot
 *   +0xc `SysMessageHandler::OnKey@00549450(vk, 1)`; `WM_KEYUP` calls the
 *   same slot with 0 (0x100122b1, 0x100122e5, 0x100122fb). `WM_CHAR` calls
 *   +0x10 `OnChar@00549510(ch & 0xffff)` (0x100122c9). An F-key makes no
 *   `WM_CHAR`, so `@key` sends none; a letter's line is followed by `@char`.
 * - `OnKey` ORs the modifiers in itself, from `GetKeyState`: 0x10000 shift,
 *   0x20000 ctrl, 0x40000 alt. It hands the key to
 *   `Window::get_key_window@00a4f5b0` (the key capture, the modal's focus,
 *   else the input window) at vslot +0x3ec, `Window::on_key@00a54ef0`, which
 *   sends a down to `key_down_event`. In a game that is
 *   `Console::on_key_down@007cc210` → `KeyMap::lookup@007d4780(options,
 *   options->mode, key, 0)` → `Console::exec_keymap@007c2ce0` → for a unit
 *   action, `Options::exec(keymap_options[k][0], [k][1])`.
 * - The executable's other key reads are `GetKeyState(0x10/0x11/0x12)` in
 *   `OnKey`, `Options::exec`, `do_formation` and `do_rotate` (the queue
 *   position and the rotation step), and `GetAsyncKeyState` behind
 *   `ron::GetKeyStateAsync@00a427f0` (scrolling, `map_click`, the editor).
 *   No line here holds a modifier: the pool's keyboard is untouched, so
 *   each reads 0, and the queue position is QUEUE_NEW. A modifier would be
 *   a `GetKeyState` IAT hook — one import of the executable's own, not
 *   another module's — and is not built (`docs/GOLDEN.md` §146).
 *
 * So `@key` calls the shipped slot directly, at the top of the stanza's
 * `do_frame` like every other `@` line, as `click.h` calls the mouse slots.
 * The slot is read from the shipped vftable at 0xb64d18, and the handler
 * must be on it or on `click.h`'s copy. No pin is installed for a key: a
 * capture with only key lines runs every other byte as before.
 *
 * A key is bound in the profile (`tools/gamelog/bindkey.py`; the pool's
 * `unattended_capture.py --bindkey ACTION=VK`). Before the press the line
 * asks `KeyMap::lookup` what the key means in the options' current mode,
 * with no modifier — the same call `on_key_down` makes, which only reads —
 * so the record says which action the press reached, or `KEYMAP_NONE`.
 *
 * Records: I_KEY per line, a = frame, b = line index | refusal << 16 (0
 * ran; 2 the handler is on neither vftable; 3 no options object; 5 the text
 * did not parse), c = vk | kind << 16 (0 `@key`, 1 `@char`), d = the
 * lookup's `KeyMapType` (−1 for `@char`), e = `options->mode`. */

#define RVA_OPTIONS_REF 0x806204u /* MiscAccess::options, VA 0xc06204 (Options *) */
#define OPTIONS_MODE 0x74u
#define RVA_KEYMAP_LOOKUP 0x3d4780u /* KeyMap::lookup@007d4780 */
#define MSG_SLOT_ONKEY (0x0cu / 4)
#define MSG_SLOT_ONCHAR (0x10u / 4)

/* An `@key` or `@char` line; `t` is past the verb. */
static void key_line(i32 frame, u32 idx, const u16 *t, i32 kind) {
    i32 vk = 0, why = 0, mapped = -1, mode = -1;
    const u32 *shipped = (const u32 *)(g_base + RVA_MSG_VFTABLE);
    u32 *handler = (u32 *)(g_base + RVA_MSG_HANDLER);
    u8 *options = *(u8 **)(g_base + RVA_OPTIONS_REF);
    if (!issue_int(&t, &vk) || vk <= 0 || vk > 0xffff) why = 5;
    else if (*handler != (u32)shipped && *handler != (u32)&g_msg_vt[1]) why = 2;
    else if (!options) why = 3;
    if (!why) {
        typedef i32(__thiscall *lookup_fn)(void *, i32, i32, i32);
        typedef void(__thiscall *key_fn)(void *, i32, i32);
        typedef void(__thiscall *char_fn)(void *, i32);
        mode = *(i32 *)(options + OPTIONS_MODE);
        if (kind == 0) {
            mapped = ((lookup_fn)(g_base + RVA_KEYMAP_LOOKUP))(options, mode, vk, 0);
            ((key_fn)shipped[MSG_SLOT_ONKEY])(handler, vk, 1);
            ((key_fn)shipped[MSG_SLOT_ONKEY])(handler, vk, 0);
        } else {
            ((char_fn)shipped[MSG_SLOT_ONCHAR])(handler, vk);
        }
    }
    emit(K_INFO, I_KEY, (u32)frame, idx | ((u32)why << 16), (u32)vk | (u32)kind << 16, (u32)mapped, (u32)mode);
}
