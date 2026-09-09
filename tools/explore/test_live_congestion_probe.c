/* Authored registry fixtures; no game bytes or native calls. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;
#define IMPORT(ret,name,args) ret name args
#define __thiscall
#define K_INFO 5
static u32 g_base = 0x400000;
static HANDLE g_proc;
static u32 output[256][7], used, short_read, registry_count, registry_capacity;
static u32 units[66][0x90/4];
static void emit(u32 k,u32 a,u32 b,u32 c,u32 d,u32 e,u32 f) {
    assert(used < 256); u32 row[] = {k,a,b,c,d,e,f};
    memcpy(output[used++],row,sizeof row);
}
static void flush(void) {}
#include "live_congestion_probe.h"
i32 ReadProcessMemory(HANDLE p,const void *address,void *out,u32 size,u32 *copied) {
    (void)p;
    u32 a = (u32)(uintptr_t)address;
    *copied = short_read ? size-1 : size;
    if (short_read) return 1;
    if (a == 0xc0aeb4) {
        u32 header[] = {registry_count,registry_capacity,0,0x1000};
        assert(size == sizeof header); memcpy(out,header,size);
    } else if (a == 0x1000) {
        assert(size == registry_count*4);
        for (u32 i=0;i<registry_count;i++) ((u32 *)out)[i] = 0x2000+i*0x100;
    } else if (a >= 0x2000 && a < 0x6200 && !(a&255)) {
        assert(size == 0x90); memcpy(out,units[(a-0x2000)/0x100],size);
    } else if (a == 0xc06210) { assert(size == 4); *(u32 *)out=0x10000; }
    else if (a == 0x102a0) { assert(size == 4); *(u32 *)out=0; }
    else { assert(!"unexpected read / native issue reached"); }
    return 1;
}
static void reset(void) {
    used=short_read=0; registry_count=1; registry_capacity=66;
    congestion_ready=congestion_failed=0;
    memset(congestion_seen,0,sizeof congestion_seen);
    memset(congestion_counts,0,sizeof congestion_counts);
    memset(units,0,sizeof units);
    for (u32 i=0;i<66;i++) {
        u8 *u=(u8 *)units[i]; u[8]=1;
        *(u16 *)(u+0xa)=(u16)i; *(u16 *)(u+0x30)=(u16)(i+100);
        *(u16 *)(u+0x8e)=0x8000;
        units[i][4]=(4000+i*10)^0x63637; units[i][5]=28000^0x63637;
    }
}
int main(void) {
    reset(); probe_congestion(149); assert(congestion_ready==1);
    registry_count=33; probe_congestion(210);
    assert(congestion_ready==2 && used==33);
    assert(congestion_counts[0]==16 && congestion_counts[1]==16);
    assert(congestion_ids[0][0]==1 && congestion_ids[1][15]==32);
    assert(congestion_x==4165 && congestion_y==28000);
    /* A recycled object id must be revalidated before entering the issuer. */
    *(u16 *)((u8 *)units[1]+0x30)=999;
    probe_congestion(220); assert(congestion_failed && output[used-1][2]==8);
    reset(); probe_congestion(149); probe_congestion(149);
    assert(congestion_failed && output[used-1][2]==12);
    reset(); short_read=1; probe_congestion(149); assert(congestion_failed);
    reset(); registry_count=513; probe_congestion(149); assert(congestion_failed);
    reset(); registry_capacity=0; probe_congestion(149); assert(congestion_failed);
    reset(); probe_congestion(149); probe_congestion(210); assert(congestion_failed);
    reset(); registry_count=3; probe_congestion(210); assert(congestion_failed);
    reset(); probe_congestion(149); registry_count=66; probe_congestion(210);
    assert(congestion_failed && output[used-1][2]==3);
    reset(); probe_congestion(149); registry_count=4;
    ((u8 *)units[1])[9]=1;
    probe_congestion(210); assert(congestion_ready==2 && congestion_counts[0]==1);
    puts("congestion probe: roster, bounds, short read, stale identity and missing baseline checked");
}
