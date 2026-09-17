#include <assert.h>
#include <stdio.h>
#include "probe_code_hash.h"

int main(int argc, char **argv) {
    (void)argv;
    assert(probe_code_hash((const unsigned char *)"", 0) == 0xcbf29ce484222325ULL);
    assert(probe_code_hash((const unsigned char *)"a", 1) == 0xaf63dc4c8601ec8cULL);
    assert(probe_code_hash((const unsigned char *)"foobar", 6) == 0x85944171f73967e8ULL);
    if (argc > 1) {
        unsigned char bytes[21];
        assert(fread(bytes, 1, sizeof bytes, stdin) == sizeof bytes);
        assert(getchar() == EOF);
        unsigned long long expected = probe_code_hash(bytes, sizeof bytes);
        for (unsigned i=0; i<sizeof bytes; i++) {
            for (unsigned bit=0; bit<8; bit++) {
                bytes[i] ^= (unsigned char)(1u << bit);
                assert(probe_code_hash(bytes, sizeof bytes) != expected);
                bytes[i] ^= (unsigned char)(1u << bit);
            }
        }
        printf("%016llx: 168 one-bit mutations rejected\n", expected);
    } else puts("FNV-1a-64 authored vectors passed");
}
