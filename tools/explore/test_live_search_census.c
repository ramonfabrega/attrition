/* Authored host fixtures for the exact census callback compiled into the DLL.
 * No game memory, native executable bytes, or allocator is used. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;
#define IMPORT(ret, name, args) ret name args
#define K_INFO 5
static HANDLE g_proc;
static u32 output[1024][7], used, reads, fail_at, short_read;
static void emit(u32 k, u32 a, u32 b, u32 c, u32 d, u32 e, u32 f) {
    assert(used < 1024);
    u32 row[7] = {k, a, b, c, d, e, f};
    memcpy(output[used++], row, sizeof row);
}
#include "live_search_census.h"

i32 ReadProcessMemory(HANDLE process, const void *address, void *out, u32 size, u32 *copied) {
    (void)process;
    u32 a = (u32)(uintptr_t)address, data[5] = {0};
    reads++;
    if (reads == fail_at) {
        *copied = short_read ? size-1 : 0;
        return (i32)short_read;
    }
    if (a == 0xe85e94) { assert(size == 4); data[0] = 0x1000; }
    else if (a == 0xe85ec0) { assert(size == 8); data[0] = 300; data[1] = 1; }
    else if (a == 0x1104) {
        assert(size == 20);
        for (u32 i = 0; i < 5; i++) data[i] = 0x2000+i*32;
    } else if (a >= 0x2000 && a <= 0x2080) {
        assert(size == 16 && (a-0x2000)%32 == 0);
        data[2] = (a-0x2000)/32+1; data[3] = 0x4000;
    } else {
        const u32 pools[] = {0xc8d810,0xc8d950,0xc8d860,0xc8d880,0xc8d9a0,0xc8d9b0,0xc8da70};
        u32 found = 0;
        for (u32 i = 0; i < 7; i++) found |= a == pools[i];
        assert(found && size == 12);
        data[0] = 0x8000; data[1] = 64; data[2] = 3;
    }
    memcpy(out, data, size); *copied = size; return 1;
}

static void reset(void) {
    used = reads = fail_at = short_read = search_census_count = 0;
}

int main(void) {
    reset(); census_suspension();
    assert(reads == 15 && used == 14);
    assert(output[0][1] == 130 && output[0][3] == 0x1000);
    for (u32 i = 0; i < 5; i++) {
        assert(output[i+1][1] == 131 && output[i+1][3] == i);
        assert(output[i+1][4] == 0x2000+i*32 && output[i+1][5] == i+1);
    }
    assert(output[13][1] == 134);
    for (u32 partial = 0; partial <= 1; partial++) {
        for (u32 failure = 1; failure <= 15; failure++) {
            reset(); fail_at = failure; short_read = partial; census_suspension();
            assert(reads == failure && output[used-1][1] == 133);
            for (u32 i = 0; i < used; i++) assert(output[i][1] != 134);
        }
    }
    reset();
    for (u32 i = 0; i < 66; i++) census_suspension();
    assert(reads == 64*15 && used == 64*14+1 && output[used-1][1] == 135);
    puts("census callback: success, 30 read failures, and cap verified");
    return 0;
}
