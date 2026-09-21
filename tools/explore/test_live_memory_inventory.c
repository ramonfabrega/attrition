/* Authored virtual ranges for the exact optional collector. No OS inspection. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;
typedef struct {u32 magic,version,frame,unit,rest[50];} RestoreProbe;
static RestoreProbe restore_probe;
static HANDLE g_proc;
static u32 g_base=0x400000;
#define IMPORT(ret,name,args)
#define K_INFO 5
#define GENERIC_WRITE 1
#define FILE_SHARE_READ 1
#define FILE_ATTRIBUTE_NORMAL 1
#define INVALID_HANDLE ((HANDLE)(intptr_t)-1)
static u32 source[32][7],source_count,query_count,clock_calls,success,failure,last_status;
static u32 bad_system,fail_query,short_query,bad_range,slow,wrap_clock,exhaust,fail_create,fail_write,short_write;
static u8 output[300000];static u32 output_size;
static void path_join(char *p,const char *name){strcpy(p,name);}
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e){
    (void)k;(void)b;(void)c;(void)d;(void)e;
    if(tag==165){assert(!a);success++;}else{assert(tag==166);failure++;last_status=a;}
}
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f){
    (void)p;(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    return fail_create?INVALID_HANDLE:(HANDLE)1;
}
static i32 WriteFile(HANDLE h,const void *p,u32 n,u32 *written,void *v){
    (void)h;(void)v;assert(n<=sizeof output);memcpy(output,p,n);output_size=n;
    *written=short_write?n-1:n;return !fail_write;
}
static void CloseHandle(HANDLE h){(void)h;}
static void GetSystemInfo(void *p){
    u32 sys[]={0,4096,0x10000,0x5fffffff,1,1,586,65536,0};
    if(bad_system==1)sys[1]=3;
    if(bad_system==2)sys[3]=0xffffffffu;
    if(bad_system==3)sys[0]=9;
    if(bad_system==4)sys[2]=0x10001;memcpy(p,sys,sizeof sys);
}
static u32 GetTickCount(void){
    u32 call=clock_calls++;
    if(wrap_clock)return call?0x10u:0xfffffff0u;
    return slow && call?2000u:0u;
}
static u32 VirtualQueryEx(HANDLE h,const void *p,void *out,u32 n){
    (void)h;assert(n==28);u32 address=(u32)(uintptr_t)p;query_count++;
    if(query_count==fail_query)return short_query?24:0;
    if(exhaust){u32 row[]={address,address,4,4096,0x1000,4,0x20000};memcpy(out,row,n);return n;}
    assert(query_count<=source_count);u32 row[7];memcpy(row,source[query_count-1],n);assert(row[0]==address);
    if(bad_range && query_count==2) {
        if(bad_range==1)row[0]+=4096;
        if(bad_range==2)row[3]=0;
        if(bad_range==3)row[3]=0xfffff000;
        if(bad_range==4)row[4]=0;
    }
    memcpy(out,row,n);return n;
}
#include "live_memory_inventory.h"
static void add(u32 base,u32 size,u32 state,u32 protect,u32 type,u32 allocation){
    u32 cursor=source_count?source[source_count-1][0]+source[source_count-1][3]:0x10000;
    if(base>cursor){u32 free_row[]={cursor,0,0,base-cursor,0x10000,0,0};memcpy(source[source_count++],free_row,28);}
    u32 row[]={base,allocation,protect,size,state,protect,type};memcpy(source[source_count++],row,28);
}
static void reset(void){
    query_count=clock_calls=success=failure=last_status=bad_system=fail_query=short_query=bad_range=slow=wrap_clock=exhaust=fail_create=fail_write=short_write=output_size=0;
}
int main(int argc,char **argv){
    add(0x10000,0x10000,0x1000,4,0x20000,0x10000);
    add(0x400000,0x1000,0x1000,0x20,0x1000000,0x400000);
    add(0x401000,0xbff000,0x1000,2,0x1000000,0x400000);
    add(0x2000000,0x2000,0x1000,4,0x20000,0x2000000);
    add(0x2002000,0x1000,0x1000,0x104,0x20000,0x2000000);
    add(0x2003000,0x1000,0x1000,0x40,0x20000,0x2000000);
    add(0x2004000,0x1000,0x2000,0,0x20000,0x2000000);
    add(0x10000000,0x10000,0x1000,4,0x1000000,0x10000000);
    add(0x50000000,0x8000,0x1000,4,0x20000,0x50000000);
    add(0x50008000,0x60000000-0x50008000,0x10000,0,0,0);
    restore_probe.frame=230;restore_probe.unit=0x14000;
    reset();capture_memory_inventory(0x10000000);assert(success==1 && !failure && memory_inventory.count==source_count);
    for(u32 partial=0;partial<2;partial++)for(u32 i=1;i<=source_count;i++) {
        reset();fail_query=i;short_query=partial;capture_memory_inventory(0x10000000);
        assert(!success && failure==1 && last_status==2 && memory_inventory.count==i-1);
    }
    for(u32 i=1;i<=4;i++){reset();bad_range=i;capture_memory_inventory(0x10000000);assert(!success && last_status==3);}
    for(u32 i=1;i<=4;i++){reset();bad_system=i;capture_memory_inventory(0x10000000);assert(!success && last_status==1);}
    reset();wrap_clock=1;capture_memory_inventory(0x10000000);assert(success && !failure && memory_inventory.elapsed==32);
    reset();slow=1;capture_memory_inventory(0x10000000);assert(!success && last_status==5);
    reset();exhaust=1;capture_memory_inventory(0x10000000);assert(!success && last_status==4 && memory_inventory.count==INVENTORY_MAX);
    reset();fail_create=1;capture_memory_inventory(0x10000000);assert(!success && last_status==6);
    reset();fail_write=1;capture_memory_inventory(0x10000000);assert(!success && last_status==6);
    reset();short_write=1;capture_memory_inventory(0x10000000);assert(!success && last_status==6);
    if(argc==3){
        FILE *f=fopen(argv[1],"rb");assert(f && fread(&restore_probe,1,sizeof restore_probe,f)==sizeof restore_probe);assert(!fclose(f));
        reset();capture_memory_inventory(0x10000000);assert(success && !failure);
        f=fopen(argv[2],"wb");assert(f && fwrite(output,1,output_size,f)==output_size);assert(!fclose(f));
    }
    printf("inventory: %u ranges; %u failed/short queries; bounds, cap, deadline and file controls pass\n",source_count,2*source_count);
}
