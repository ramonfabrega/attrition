/* Authored API fixtures execute the production writer, including its failures. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;typedef unsigned int u32;typedef int i32;typedef void *HANDLE;
#define __cdecl
#define IMPORT(ret,name,args) static ret name args
#define RON_STATE_PLAN "test_frame_snapshot_plan.h"
#define RON_STATE_FRAME 24
#define K_INFO 5
#define GENERIC_WRITE 1
#define FILE_SHARE_READ 1
#define FILE_ATTRIBUTE_NORMAL 1
#define INVALID_HANDLE ((HANDLE)(intptr_t)-1)
static HANDLE g_proc;static u32 g_base=0x400000;static i32 g_frame=23;
static u8 memory[0x3000],output[0x10000];static u32 out_size,reads,writes,queries,clock_calls;
static u32 bad_read,bad_write,bad_query,drift,slow,close_bad,create_bad,success,failure,after_count,after_changed,after_status;
static void path_join(char *p,const char *n){strcpy(p,n);}
static u32 rd_fs(u32 n){return n==8?0x800000:0x801000;}
static u32 GetCurrentThreadId(void){return 1;}
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e){
    (void)b;(void)c;(void)d;(void)e;assert(k==5);if(tag==180){success++;assert(a==24);}else if(tag==181){assert(a);failure++;}else if(tag==182){assert(a==24);after_count++;after_changed=c;after_status=e;}else{assert(tag==184 && e);}
}
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f){
    (void)a;(void)b;(void)c;(void)e;(void)f;assert(d==1 && !strcmp(p,"frame-snapshot.bin"));return create_bad?INVALID_HANDLE:(HANDLE)1;
}
static i32 CloseHandle(HANDLE h){assert(h==(HANDLE)1);return !close_bad;}
static i32 WriteFile(HANDLE h,const void *p,u32 n,u32 *got,void *v){
    (void)v;assert(h==(HANDLE)1);writes++;assert(out_size+n<=sizeof output);memcpy(output+out_size,p,n);out_size+=n;*got=writes==bad_write?n-1:n;return 1;
}
static u32 GetTickCount(void){clock_calls++;return slow && clock_calls>=slow?5001:0;}
static i32 ReadProcessMemory(HANDLE h,const void *p,void *out,u32 n,u32 *got){
    (void)h;u32 a=(u32)(uintptr_t)p;reads++;assert(a>=0x400000 && a+n<=0x403000);
    memcpy(out,memory+a-0x400000,n);if(reads==drift)((u8 *)out)[0]^=1;
    *got=reads==bad_read?n-1:n;return 1;
}
static void GetSystemInfo(void *p){u32 a[9]={0,4096,0x10000,0x900fff,0,0,0,0,0};memcpy(p,a,sizeof a);}
static u32 VirtualQueryEx(HANDLE h,const void *p,void *out,u32 n){
    (void)h;assert(n==28);u32 a=(u32)(uintptr_t)p;queries++;
    u32 rows[][7]={
        {0x10000,0,0,0x3f0000,0x10000,0,0},
        {0x400000,0x400000,4,0x3000,0x1000,4,0x1000000},
        {0x403000,0,0,0x3fd000,0x10000,0,0},
        {0x800000,0x800000,4,0x1000,0x1000,4,0x20000},
        {0x801000,0,0,0xff000,0x10000,0,0},
        {0x900000,0x900000,4,0x1000,0x1000,4,0x1000000}};
    for(u32 i=0;i<6;i++)if(rows[i][0]==a){memcpy(out,rows[i],28);if(queries==bad_query)((u32 *)out)[0]^=4;return 28;}
    assert(0);return 0;
}
#include "live_frame_snapshot.h"
static void reset(void){
    out_size=reads=writes=queries=clock_calls=bad_read=bad_write=bad_query=drift=slow=close_bad=create_bad=success=failure=after_count=after_changed=after_status=0;
    memset(memory,0x5a,sizeof memory);u32 game=0x400100,frame=24,start=20,end=30;
    memcpy(memory,&game,4);memcpy(memory+0x100,&frame,4);memcpy(memory+0x204,&start,4);memcpy(memory+0x208,&end,4);
}
static void run(void){fs_capture(0x400100,24,0x400200,0x900100,0x800000,0x801000);}
int main(int argc,char **argv){
    assert(argc==2);reset();run();assert(success==1 && !failure);u32 nr=reads,nw=writes,nq=queries,nc=clock_calls;
    FILE *f=fopen(argv[1],"wb");assert(f);assert(fwrite(output,1,out_size,f)==out_size);assert(!fclose(f));
    for(u32 i=1;i<=nr;i++){reset();bad_read=i;run();assert(failure==1 && !success);}
    for(u32 i=1;i<=nw;i++){reset();bad_write=i;run();assert(failure==1 && !success);}
    for(u32 i=1;i<=nq;i++){reset();bad_query=i;run();assert(failure==1 && !success);}
    reset();drift=6;run();assert(failure==1 && !success);
    reset();drift=9;run();assert(failure==1 && !success);
    reset();slow=nc-1;run();assert(failure==1 && !success);
    reset();close_bad=1;run();assert(failure==1 && !success);
    reset();create_bad=1;run();assert(failure==1 && !success);
    reset();run();frame_snapshot_after(0);assert(after_count==1 && !after_changed && !after_status);
    frame_snapshot_after(0);assert(after_count==1);
    reset();run();memory[0x1000]^=1;frame_snapshot_after(0);assert(after_count==1 && after_changed==1);
    reset();run();bad_read=reads+1;frame_snapshot_after(0);assert(after_count==1 && after_status==4);
    /* A rejected log window must never open an output file. */
    reset();u32 end=24;memcpy(memory+0x208,&end,4);run();assert(failure==1 && !out_size);
    /* Boundary callback ignores other frames; no fabricated successful packet. */
    reset();fs_claimed=0;u32 frame=23,regs[9]={0};memcpy(memory+0x100,&frame,4);frame_snapshot_enter(regs);assert(!fs_claimed && !out_size);
    return 0;
}
