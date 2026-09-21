/* Authored memory only. Mock APIs exercise the production streaming collector. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;typedef unsigned int u32;typedef int i32;typedef void *HANDLE;
typedef struct {u32 magic,version,frame,unit,rest[50];} RestoreProbe;
#include "memory_inventory_format.h"
#include "restore_context_format.h"
static MemoryInventory memory_inventory;
static RestoreContext restore_context;
static HANDLE g_proc;
#define K_INFO 5
#define GENERIC_WRITE 1
#define FILE_SHARE_READ 1
#define FILE_ATTRIBUTE_NORMAL 1
#define INVALID_HANDLE ((HANDLE)(intptr_t)-1)
static u8 output[8*1024*1024];
static u32 output_size,reads,writes,queries,clocks,success,failure,status,create_count,close_count;
static u32 fail_read,short_read,fail_write,short_write,fail_query,change_query,fail_create,fail_close,slow_at,wrap_clock,corrupt_read,corrupt_copy,short_query;
static void path_join(char *p,const char *name){strcpy(p,name);}
static u32 GetTickCount(void){u32 n=clocks++;if(slow_at && n>=slow_at)return 5000;return wrap_clock?0xfffffff0u+n:0;}
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e){
    (void)b;(void)c;(void)d;(void)e;assert(k==5);
    if(tag==167){assert(!a);success++;}else{assert(tag==168);failure++;status=a;}
}
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f){
    (void)a;(void)b;(void)c;(void)e;(void)f;assert(!strcmp(p,"memory-payload.bin") && d==1);create_count++;
    return fail_create?INVALID_HANDLE:(HANDLE)1;
}
static i32 CloseHandle(HANDLE h){assert(h==(HANDLE)1);close_count++;return !fail_close;}
static i32 WriteFile(HANDLE h,const void *p,u32 n,u32 *written,void *v){
    (void)v;assert(h==(HANDLE)1);writes++;
    assert(output_size+n<=sizeof output);memcpy(output+output_size,p,n);output_size+=n;
    *written=writes==short_write?n-1:n;return writes!=fail_write;
}
static u8 byte_at(u32 address){
    RestoreContext *c=&restore_context;
    if(c->unit<=address && address<c->unit+sizeof c->unit_data)return c->unit_data[address-c->unit];
    if(c->origin<=address && address<c->origin+c->table_bytes)return c->table[address-c->origin];
    u32 world=0x1008000;
    if(0xc06188<=address && address<0xc0618c)return ((u8 *)&world)[address-0xc06188];
    if(0xcab3ac<=address && address<0xcab3b0)return ((u8 *)&c->origin)[address-0xcab3ac];
    if(0xcae5fc<=address && address<0xcae600)return ((u8 *)&c->center)[address-0xcae5fc];
    return (u8)(address^(address>>8));
}
static i32 ReadProcessMemory(HANDLE h,const void *p,void *out,u32 n,u32 *got){
    (void)h;u32 address=(u32)(uintptr_t)p;reads++;
    for(u32 i=0;i<n;i++)((u8 *)out)[i]=byte_at(address+i);
    if(reads==corrupt_read)((u8 *)out)[0]^=1;
    if(corrupt_copy && n>=4096 && address<=restore_context.origin && restore_context.origin<address+n)
        ((u8 *)out)[restore_context.origin-address]^=1;
    *got=reads==short_read?n-1:n;return reads!=fail_read;
}
static u32 VirtualQueryEx(HANDLE h,const void *p,void *out,u32 n){
    (void)h;assert(n==28);queries++;if(queries==fail_query)return short_query?24:0;
    u32 address=(u32)(uintptr_t)p;
    for(u32 i=0;i<memory_inventory.count;i++)if(memory_inventory.ranges[i].base==address){
        memcpy(out,&memory_inventory.ranges[i],28);
        if(queries==change_query)((InventoryRange *)out)->protect=2;
        return 28;
    }
    assert(0);return 0;
}
#include "live_memory_payload.h"
static void add(u32 base,u32 size,u32 state,u32 protect,u32 kind,u32 allocation){
    MemoryInventory *m=&memory_inventory;
    u32 cursor=m->count?m->ranges[m->count-1].base+m->ranges[m->count-1].size:m->begin;
    if(base>cursor)m->ranges[m->count++]=(InventoryRange){cursor,0,0,base-cursor,0x10000,0,0};
    m->ranges[m->count++]=(InventoryRange){base,allocation,protect,size,state,protect,kind};
}
static void setup(void){
    MemoryInventory *m=&memory_inventory;RestoreContext *c=&restore_context;memset(m,0,sizeof *m);
    m->magic=INVENTORY_MAGIC;m->version=1;m->frame=c->frame;m->unit=c->unit;m->begin=0x10000;m->end=0x60000000;
    m->page=4096;m->record_bytes=28;m->cursor=m->end;m->main_base=0x400000;m->observer=0x10000000;
    m->max_records=INVENTORY_MAX;m->max_ms=INVENTORY_MS;m->prefix=c->prefix;
    add(0x10000,0x10000,0x1000,4,0x20000,0x10000);
    add(0x400000,0x1000,0x1000,0x20,0x1000000,0x400000);
    add(0xc00000,0xb0000,0x1000,8,0x1000000,0x400000);
    add(0x2000000,PAYLOAD_CHUNK+4096,0x1000,4,0x20000,0x2000000);
    add(0x3000000,4096,0x1000,0x104,0x20000,0x3000000);
    add(0x4000000,4096,0x1000,4,0x40000,0x4000000);
    add(0x5000000,4096,0x1000,0x40,0x20000,0x5000000);
    add(0x10000000,4096,0x1000,4,0x1000000,0x10000000);
    add(0x50000000,0x8000,0x1000,4,0x20000,0x50000000);
    add(0x50008000,m->end-0x50008000,0x10000,0,0,0);
}
static void reset(void){
    output_size=reads=writes=queries=clocks=success=failure=status=create_count=close_count=0;
    fail_read=short_read=fail_write=short_write=fail_query=change_query=fail_create=fail_close=slow_at=wrap_clock=corrupt_read=corrupt_copy=short_query=0;
    setup();
}
static void bad(u32 expected){capture_memory_payload();assert(!success && failure==1 && status==expected);assert(close_count==(!fail_create && create_count));}
int main(int argc,char **argv){
    RestoreContext *c=&restore_context;c->frame=230;c->unit=0x14000;c->origin=0x2000000;c->center=c->origin+96;c->table_bytes=192;
    c->tib[1]=0x50008000;c->tib[2]=0x50000000;
    if(argc==3){FILE *f=fopen(argv[1],"rb");assert(f && fread(c,1,1136+192,f)==1136+192);assert(!fclose(f));}
    reset();capture_memory_payload();assert(success==1 && !failure && close_count==1);
    assert(payload_header.count==3 && payload_copied==0x1c1000);
    u32 nr=reads,nw=writes,nq=queries,nc=clocks;
    for(u32 i=1;i<=nr;i++)for(u32 shorted=0;shorted<2;shorted++){
        reset();if(shorted)short_read=i;else fail_read=i;bad(4);
    }
    for(u32 i=1;i<=nw;i++)for(u32 shorted=0;shorted<2;shorted++){
        reset();if(shorted)short_write=i;else fail_write=i;bad(5);
    }
    for(u32 i=1;i<=nq;i++){reset();fail_query=i;bad(6);reset();fail_query=i;short_query=1;bad(6);reset();change_query=i;bad(6);}
    for(u32 i=1;i<nc;i++){reset();slow_at=i;bad(8);}
    reset();corrupt_read=2;bad(7); /* pre-copy unit */
    reset();corrupt_copy=1;bad(7); /* changed table bytes during the copy */
    reset();corrupt_read=nr-4;bad(7); /* post-copy unit */
    reset();fail_create=1;bad(9);
    reset();fail_close=1;bad(10);
    reset();memory_inventory.status=1;bad(1);assert(!reads && !create_count);
    reset();memory_inventory.ranges[0].protect=0x104;bad(2);assert(!reads && !create_count);
    reset();memory_inventory.ranges[memory_inventory.count-1]=(InventoryRange){0x50008000,0x50008000,4,0x50000000,0x1000,4,0x20000};
    memory_inventory.end=0xa0008000;memory_inventory.cursor=memory_inventory.end;bad(3);assert(!reads && !create_count);
    reset();u32 tail=memory_inventory.count-1;
    memory_inventory.ranges[tail]=(InventoryRange){0x50008000,0x50008000,4,PAYLOAD_CAP-0x1c1000,0x1000,4,0x20000};
    memory_inventory.end=0x50008000+PAYLOAD_CAP-0x1c1000;memory_inventory.cursor=memory_inventory.end;
    fail_read=1;bad(4);assert(payload_header.bytes==PAYLOAD_CAP); /* exact cap passes planning */
    reset();wrap_clock=1;capture_memory_payload();assert(success && !failure);
    reset();capture_memory_payload();assert(success && !failure);
    if(argc==3){FILE *f=fopen(argv[2],"wb");assert(f && fwrite(output,1,output_size,f)==output_size);assert(!fclose(f));}
    printf("payload: %u read, %u write, %u mapping and %u deadline positions checked; cap, anchors and cleanup pass\n",nr,nw,nq,nc-1);
}
