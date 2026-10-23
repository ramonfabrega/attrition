/* The held key (item 3502, parked 3442a and 3495a): a modifier the game
 * reads as held, with no macOS keyboard. Included by `tracer.c` after
 * `key.h`, and by `hold_test.c` on the host, so the parts a test can reach —
 * the import walk, the span table and the answer — are pure.
 *
 *   `@hold <vk> <until-frame>`   the key reads as down from this line's
 *                                frame through `until-frame`, inclusive
 *
 * **Why an import slot.** `@key` and `@click` call the shipped handler's
 * slots (`key.h`, `click.h`), and no slot carries a modifier: every place the
 * executable asks whether shift, ctrl or alt is down — `OnKey@00549450`,
 * `Options::exec`, `do_formation`, `do_rotate`,
 * `Console::execute_at_cursor@007c6630.c:328-343` — calls its own
 * `GetKeyState` import and tests the high byte (`(char)(r >> 8) != 0`). The
 * import is USER32's `GetKeyState` at IAT VA 0xac53e8 (`llvm-objdump -p`:
 * USER32's first thunk 0x6c53b4, the thirteenth name; parked 3495a), and no
 * other module's copy is asked, so one slot is the whole door.
 *
 * **The hook.** The first `@hold` line checks the slot by name — the
 * executable's import directory must put USER32's `GetKeyState` at that RVA
 * — saves what the loader wrote there and points it at `hold_get_key_state`,
 * which answers 0x8000 (the high byte, no toggle) for a key whose span holds
 * the current `do_frame` and asks the saved function otherwise. The slot is
 * never put back: past the last span every call is the real one. A capture
 * without an `@hold` line never touches the slot.
 *
 * Records: I_HOLD per line, a = frame, b = line index | refusal << 16 (0
 * held; 2 the import is not at the slot; 3 the slot could not be made
 * writable; 5 the text did not parse; 6 the span table is full), c = vk |
 * until << 16, d = the slot's RVA as found (0 when not), e = held answers
 * so far. I_HOLD_ASKED once per frame a held answer was given, a = frame,
 * b = vk, c = held answers so far. */

#ifndef HOLD_H
#define HOLD_H

#define HOLD_MAX 16

typedef struct {
    i32 vk, from, until;
} HoldSpan;

static int hold_name_is(const u8 *a, const char *b, int fold) {
    for (; *b; a++, b++) {
        u8 x = *a, y = (u8)*b;
        if (fold && x >= 'a' && x <= 'z') x = (u8)(x - 32);
        if (fold && y >= 'a' && y <= 'z') y = (u8)(y - 32);
        if (x != y) return 0;
    }
    return *a == 0;
}

/* The IAT RVA the image at `base` (mapped, PE32) binds `dll!fn` by name
 * at, or 0. The DLL's name compares without case, as the loader does. */
static u32 hold_import_slot(const u8 *base, const char *dll, const char *fn) {
    if (base[0] != 'M' || base[1] != 'Z') return 0;
    const u8 *nt = base + *(const u32 *)(base + 0x3c);
    if (*(const u32 *)nt != 0x4550 || *(const u16 *)(nt + 0x18) != 0x10b) return 0;
    u32 dir = *(const u32 *)(nt + 0x80);
    if (!dir || !*(const u32 *)(nt + 0x84)) return 0;
    for (const u32 *d = (const u32 *)(base + dir); d[3] || d[4]; d += 5) {
        if (!hold_name_is(base + d[3], dll, 1)) continue;
        const u32 *names = (const u32 *)(base + (d[0] ? d[0] : d[4]));
        for (u32 i = 0; names[i]; i++)
            if (!(names[i] & 0x80000000u) && hold_name_is(base + names[i] + 2, fn, 0)) return d[4] + 4 * i;
    }
    return 0;
}

/* A span `@hold vk until` on `frame`: the key a VK and the span not empty. */
static int hold_valid(i32 vk, i32 frame, i32 until) {
    return vk > 0 && vk < 0x100 && until >= frame;
}

/* Is `vk` held on `frame` by any span. */
static int hold_held(const HoldSpan *s, u32 n, i32 vk, i32 frame) {
    for (u32 i = 0; i < n; i++)
        if (s[i].vk == vk && frame >= s[i].from && frame <= s[i].until) return 1;
    return 0;
}

#endif
