/* The hover re-assert (item 3494, parked 3463a): a pinned cursor's tooltip.
 * Included by `tracer.c` after `click.h`, and by `hover_test.c` on the host,
 * so the part a test can reach — the cfg key and the predicate — is pure.
 *
 * **Why a pin alone raises no tooltip.** An `@cursor` line calls the
 * handler's `OnMouseMove` at the top of `do_frame` (`click.h`), and over the
 * resource bar that reaches `IFaceResources::on_spot_move@0081b4e0`, which
 * writes `mouseover->type |= 0x200` and `t = good` (run1082 entered it).
 * But `Scene::render@008b3270.c:161-162` sets `region = MOUSEOVER_OTHER` and
 * `type = 0` before `Scene::describe_mouseover@008b47e0` (`:517`), and
 * `MouseOver::describe@007d09b0` takes no case for type 0. The pin drops
 * the window procedure's own mouse events, so nothing re-sets the type
 * between the two; what does in a hand-held game is not read.
 *
 * **The hook.** With `hover=1` in `rontrace.cfg`, the entry of
 * `describe_mouseover` is hooked, and while the pin holds a point a click
 * line has moved it to, the hook **re-sends the pinned `OnMouseMove`**
 * through the shipped slot (`g_msg_orig[0x14 / 4]`) before the body runs.
 * That, not a direct `on_spot_move` call with its last arguments, because
 * it is the path the `@cursor` line itself took: the same window lookup,
 * the same spot test, and no second hook to capture arguments with. It is
 * harmless to `describe`'s gate: `IFaceMainBase`'s `on_mouse_move` stamps
 * `last_move` only when the point changes (run1129 never enters it), and `same_as_before`
 * holds for an unchanged good, so the second drawn frame describes.
 *
 * Off by default: without the key the site is not hooked and every byte
 * runs as before. When it is on, `describe_mouseover` has no coverage stub
 * (a hooked entry never has one, `is_hook_site`), so a hover trace does not
 * record that function's own `HIT`; its I_HOVER records stand for it.
 *
 * Records: I_HOVER once per frame the re-send runs, a = frame, b = re-sends
 * so far, c = x | y << 16, d = `mouseover->type` before | after << 16
 * (low halves), e = `mouseover->t` after. */

#ifndef HOVER_H
#define HOVER_H

/* `hover=N` at the start of a cfg line: sets `*want` and returns 1; any
 * other line returns 0 and leaves it. */
static int hover_cfg(const char *p, i32 *want) {
    static const char key[] = "hover=";
    for (u32 i = 0; i < sizeof key - 1; i++)
        if (p[i] != key[i]) return 0;
    p += sizeof key - 1;
    i32 v = 0;
    while (*p >= '0' && *p <= '9') v = v * 10 + (*p++ - '0');
    *want = v != 0;
    return 1;
}

/* Re-send at this entry: the key is on, the pin is installed (`click.h`'s
 * state 0) and a click line has run, so the pin names a chosen point. */
static int hover_due(i32 want, i32 pin_state, i32 moved) {
    return want && pin_state == 0 && moved;
}

#endif
