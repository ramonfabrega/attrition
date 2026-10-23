/* queue_test.c — the `queued` byte of an `@` line on the host (item 3504).
 *
 * Built and run by `tools/explore/test_issue_queue.py`. Exits non-zero,
 * naming the case, when `queue.h` gives a verb the wrong `QueuePos`. */
#include <stdio.h>

typedef int i32;
#include "queue.h"

static int failed;

static void expect(const char *name, i32 got, i32 want) {
    if (got != want) {
        printf("%s: %d, want %d\n", name, got, want);
        failed = 1;
    }
}

int main(void) {
    expect("QUEUE_FIRST is the PDB's 0", QUEUE_FIRST, 0);
    expect("QUEUE_LAST is the PDB's 1", QUEUE_LAST, 1);
    expect("QUEUE_NEW is the PDB's 2", QUEUE_NEW, 2);
    expect("@repair", issue_queue(0, 0), 2);
    expect("@attackadd", issue_queue(1, 0), 1);
    expect("@repairins", issue_queue(0, 1), 0);
    if (!failed) printf("ok\n");
    return failed;
}
