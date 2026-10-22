/* hover_test.c — the hover re-assert's switch on the host (item 3494).
 *
 * Built and run by `tools/explore/test_hover_hook.py`. Exits non-zero,
 * naming the case, when `hover.h` turns the hook on without `hover=1` or
 * leaves it off with it, or re-sends without an installed, moved pin. */
#include <stdio.h>

typedef int i32;
typedef unsigned int u32;
#include "hover.h"

static int failed;

/* A cfg text read the way `read_cfg` reads it: one key per line, the
 * hover key tried at each line's start. */
static i32 want_after(const char *cfg) {
    i32 want = 0;
    const char *p = cfg;
    while (*p) {
        hover_cfg(p, &want);
        while (*p && *p != '\n') p++;
        while (*p == '\n' || *p == '\r') p++;
    }
    return want;
}

static void check(int got, int want, const char *what) {
    if (got != want) {
        printf("FAIL %s: %d, want %d\n", what, got, want);
        failed = 1;
    }
}

int main(void) {
    check(want_after(""), 0, "an empty cfg");
    check(want_after("window=600-640\ncover=1\ncursor=1400,1000\n"), 0, "a cfg without the key");
    check(want_after("window=600-640\nhover=1\n"), 1, "hover=1");
    check(want_after("hover=1\r\ncover=1\r\n"), 1, "hover=1 under CRLF");
    check(want_after("hover=0\n"), 0, "hover=0");
    check(want_after("hover=1\nhover=0\n"), 0, "the last line wins");
    check(want_after("hovering=1\n"), 0, "a longer key");
    check(want_after("# hover=1\n"), 0, "a comment");
    check(hover_due(0, 0, 1), 0, "off");
    check(hover_due(1, -1, 1), 0, "no pin tried");
    check(hover_due(1, 4, 1), 0, "a refused pin");
    check(hover_due(1, 0, 0), 0, "a pin no click line has moved");
    check(hover_due(1, 0, 1), 1, "on, pinned and moved");
    if (!failed) printf("ok\n");
    return failed;
}
