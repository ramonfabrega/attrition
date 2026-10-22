/* hold_test.c — the held key's pure parts on the host (item 3502).
 *
 * Built and run by `tools/explore/test_hold_hook.py`. Exits non-zero,
 * naming the case, when `hold.h` finds an import at the wrong slot (or one
 * it should not find), accepts a span it should refuse, or answers held
 * outside a span's frames or for another key. */
#include <stdio.h>
#include <string.h>

typedef unsigned char u8;
typedef unsigned short u16;
typedef int i32;
typedef unsigned int u32;
#include "hold.h"

static int failed;

static void check(int got, int want, const char *what) {
    if (got != want) {
        printf("FAIL %s: %d (0x%x), want %d (0x%x)\n", what, got, got, want, want);
        failed = 1;
    }
}

/* A mapped PE32 with two imported DLLs, laid out by hand: KERNEL32 first,
 * then USER32 with GetAsyncKeyState, GetKeyboardState, GetKeyState and an
 * ordinal, its first thunk at 0x400 — so GetKeyState binds at 0x408. */
static u8 img[0x1000];

static void put32(u32 at, u32 v) { memcpy(img + at, &v, 4); }
static void put16(u32 at, u16 v) { memcpy(img + at, &v, 2); }
static void puts_at(u32 at, const char *s) { memcpy(img + at, s, strlen(s) + 1); }

static void build(const char *user32_name, int pe32plus) {
    memset(img, 0, sizeof img);
    img[0] = 'M', img[1] = 'Z';
    put32(0x3c, 0x80);
    put32(0x80, 0x4550);
    put16(0x80 + 0x18, pe32plus ? 0x20b : 0x10b);
    put32(0x80 + 0x80, 0x200); /* the import directory */
    put32(0x80 + 0x84, 60);
    /* KERNEL32: OFT 0x300, name 0x600, FT 0x380; imports GetKeyState too */
    put32(0x200, 0x300), put32(0x200 + 12, 0x600), put32(0x200 + 16, 0x380);
    puts_at(0x600, "KERNEL32.dll");
    put32(0x300, 0x700), put32(0x380, 0x700);
    puts_at(0x702, "GetKeyState");
    /* USER32: OFT 0x340, name 0x620, FT 0x400 */
    put32(0x214, 0x340), put32(0x214 + 12, 0x620), put32(0x214 + 16, 0x400);
    puts_at(0x620, user32_name);
    u32 names[] = {0x720, 0x740, 0x760, 0x80000007u};
    for (u32 i = 0; i < 4; i++) put32(0x340 + 4 * i, names[i]), put32(0x400 + 4 * i, names[i]);
    puts_at(0x722, "GetAsyncKeyState");
    puts_at(0x742, "GetKeyboardState");
    puts_at(0x762, "GetKeyState");
}

int main(void) {
    build("USER32.dll", 0);
    check((i32)hold_import_slot(img, "USER32.dll", "GetKeyState"), 0x408, "USER32!GetKeyState");
    check((i32)hold_import_slot(img, "USER32.dll", "GetAsyncKeyState"), 0x400, "the slot before it");
    check((i32)hold_import_slot(img, "KERNEL32.dll", "GetKeyState"), 0x380, "the same name in another DLL");
    check((i32)hold_import_slot(img, "USER32.dll", "GetKey"), 0, "a prefix of a name");
    check((i32)hold_import_slot(img, "USER32.dll", "MessageBeep"), 0, "a name not imported");
    check((i32)hold_import_slot(img, "GDI32.dll", "GetKeyState"), 0, "a DLL not imported");
    build("user32.DLL", 0);
    check((i32)hold_import_slot(img, "USER32.dll", "GetKeyState"), 0x408, "the DLL's name without case");
    put32(0x214, 0); /* no original first thunk: the names are read off the FT */
    check((i32)hold_import_slot(img, "USER32.dll", "GetKeyState"), 0x408, "a bound import with no OFT");
    build("USER32.dll", 1);
    check((i32)hold_import_slot(img, "USER32.dll", "GetKeyState"), 0, "a PE32+ image");
    build("USER32.dll", 0);
    img[0] = 0;
    check((i32)hold_import_slot(img, "USER32.dll", "GetKeyState"), 0, "no MZ");

    check(hold_valid(18, 610, 616), 1, "VK_MENU over 610..616");
    check(hold_valid(16, 610, 610), 1, "a one-frame span");
    check(hold_valid(18, 610, 609), 0, "a span ending before its line");
    check(hold_valid(0, 610, 616), 0, "vk 0");
    check(hold_valid(0x100, 610, 616), 0, "vk past a byte");
    check(hold_valid(-1, 610, 616), 0, "a negative vk");

    HoldSpan s[] = {{18, 610, 616}, {16, 700, 700}};
    check(hold_held(s, 2, 18, 609), 0, "alt before the span");
    check(hold_held(s, 2, 18, 610), 1, "alt on the line's frame");
    check(hold_held(s, 2, 18, 612), 1, "alt inside");
    check(hold_held(s, 2, 18, 616), 1, "alt on the last frame");
    check(hold_held(s, 2, 18, 617), 0, "alt after");
    check(hold_held(s, 2, 17, 612), 0, "ctrl while alt is held");
    check(hold_held(s, 2, 16, 612), 0, "shift outside its span");
    check(hold_held(s, 2, 16, 700), 1, "shift in its span");
    check(hold_held(s, 1, 16, 700), 0, "a span past the count");
    check(hold_held(s, 0, 18, 612), 0, "no spans");
    if (!failed) printf("ok\n");
    return failed;
}
