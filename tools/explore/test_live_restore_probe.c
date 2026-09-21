/* Authored memory for exact restore callbacks; no game bytes or native code. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;
#define __cdecl
#define K_INFO 5
#define INVALID_HANDLE ((HANDLE)(intptr_t)-1)
#define GENERIC_WRITE 1
#define FILE_SHARE_READ 1
#define FILE_ATTRIBUTE_NORMAL 1
#define MEM_COMMIT 1
#define MEM_RESERVE 2
#define PAGE_EXECUTE_READWRITE 4
static HANDLE g_proc;
static u32 g_base;
static i32 g_frame=224,g_cover;
static int graph_failed;
static u32 calls,fail_at,short_read,graphs,errors,entries,delegates,write_fail,write_short;
static u32 stack[12],modes[2],output[54];
static u8 context_output[984];
static u32 contexts,prefix_reads,context_write_fail,context_write_short,context_create_fail;
static u32 bad_tib,bad_extent;
static u32 rd_fs(u32 off){assert(off==0 || off==0x18);return off?0x16000:0xffffffffu;}
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e) {
    (void)k;(void)b;(void)c;(void)d;(void)e;
    if(tag==163)errors++;
    if(tag==161)entries++;
    if(tag==164){assert(!a);contexts++;}
    if(tag==162) {if(a)errors++;else delegates++;}
}
static void flush(void){}
static void path_join(char *p,const char *n){strcpy(p,n);}
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f){
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    if(!strcmp(p,"restore-context.bin"))return context_create_fail?INVALID_HANDLE:(HANDLE)2;
    return(HANDLE)1;
}
static i32 WriteFile(HANDLE h,const void *p,u32 n,u32 *written,void *x){
    (void)x;
    if(h==(HANDLE)2){assert(n==sizeof context_output);memcpy(context_output,p,n);
        *written=context_write_short?n-1:n;return !context_write_fail;}
    prefix_reads=calls;assert(n==sizeof output);memcpy(output,p,n);
    *written=write_short?n-1:n;return !write_fail;
}
static void CloseHandle(HANDLE h){(void)h;}
static void *VirtualAlloc(void *p,u32 a,u32 b,u32 c){(void)p;(void)a;(void)b;(void)c;return 0;}
static void FlushInstructionCache(HANDLE h,void *p,u32 n){(void)h;(void)p;(void)n;}
static void capture_search_graph(u32 unit){assert(unit==0x14000);graphs++;}
static i32 ReadProcessMemory(HANDLE h,const void *pointer,void *out,u32 n,u32 *copied){
    (void)h;u32 a=(u32)(uintptr_t)pointer,d[4]={0};calls++;
    if(calls==fail_at){*copied=short_read?n-1:0;return (i32)short_read;}
    if(a==0x50004000){assert(n==16);memcpy(out,stack+8,n);}
    else if(a==0x50003fe0){assert(n==48);memcpy(out,stack,n);}
    else if(a==0xe85ec0){assert(n==8);memcpy(out,modes,n);}
    else if(a==0x16000){
        u32 tib[]={0xffffffffu,0x50008000,0x50000000,0,0,0,0x16000};assert(n==sizeof tib);
        if(bad_tib)tib[bad_tib-1]^=0x100000;memcpy(out,tib,n);
    }
    else if(a==0x50003da0){assert(n==512);memset(out,0x5a,n);}
    else if(a==0x2000000){
        assert(n==192);for(int i=0;i<48;i++){int v=i-24;v=v<0?(v-2)/3:v/3;memcpy((u8 *)out+4*i,&v,4);}
    }
    else {
        if(a==0xcab3ac)d[0]=0x2000000;
        else if(a==0xcae5fc)d[0]=0x2000060+bad_extent;
        else if(a==0xc061bc)d[0]=0x11000;
        else if(a==0xc0618c)d[0]=0x12000;
        else if(a==0x11000)d[0]=1;
        else if(a==0x12014)d[0]=0x13000;
        else if(a==0x13004)d[0]=0x14000;
        else if(a==0x14010){d[0]=4000^0x63637;d[1]=8000^0x63637;assert(n==8);}
        else if(a==0xc0aeb4){d[0]=2;d[1]=2;d[3]=0x15000;assert(n==16);}
        else if(a==0x15004)d[0]=0x14000;
        else assert(!"unexpected memory read");
        memcpy(out,d,n);
    }
    *copied=n;return 1;
}
#include "live_restore_probe.h"
static void reset(void){
    calls=fail_at=short_read=graphs=errors=entries=delegates=write_fail=write_short=0;
    graph_failed=restore_claimed=restore_pending=0;
    contexts=prefix_reads=context_write_fail=context_write_short=context_create_fail=bad_tib=bad_extent=0;
    memset(&restore_probe,0,sizeof restore_probe);
    u32 s[]={0x140b8,4000,8000,0,1,0,8,9,0x12345678,0x140b8,0,1};
    memcpy(stack,s,sizeof s);modes[0]=125;modes[1]=0;
}
static void capture(void){
    u32 before[]={7,8,9,0x50003ffc,10,11,12,13,0x202};
    u32 after[]={7,0,0x50003ffc,0x50003fdc,1,8000,4000,13,0x202};
    restore_enter(before);modes[0]=300;modes[1]=1;restore_delegate(after);
}
int main(int argc,char **argv){
    reset();capture();assert(graphs==1 && entries==1 && delegates==1 && contexts==1 && !errors);
    assert(output[2]==224 && output[3]==0x14000 && output[47]==300);
    assert(output[1]==2 && output[49]==0x8d5 && output[50]==2 && output[53]==0x15000);
    assert(output[12]==0 && output[34]==0);
    u32 total=calls, prefix_total=prefix_reads;
    for(u32 partial=0;partial<2;partial++)for(u32 i=1;i<=total;i++){
        reset();fail_at=i;short_read=partial;capture();assert(errors && !contexts && calls==i);assert(delegates==(i>prefix_total));
    }
    reset();context_write_fail=1;capture();assert(errors && delegates && !contexts);
    reset();context_write_short=1;capture();assert(errors && delegates && !contexts);
    reset();context_create_fail=1;capture();assert(errors && delegates && !contexts);
    for(u32 field=1;field<=7;field+=6){reset();bad_tib=field;capture();assert(errors && !contexts);}
    reset();bad_extent=1;capture();assert(errors && !contexts);
    reset();bad_extent=786432;capture();assert(errors && !contexts);
    reset();graph_failed=1;capture();assert(graphs==1 && !entries && !delegates);
    reset();write_fail=1;capture();assert(errors && !delegates);
    reset();write_short=1;capture();assert(errors && !delegates);
    reset();stack[10]=8;capture();assert(errors && !graphs && !delegates);
    reset();capture();capture();assert(entries==1 && delegates==1);
    reset();install_restore_probe();assert(errors==1);
    u8 stub[256]={0};u32 length=restore_callback(stub,(void *)0x12345678);
    assert(length>15 && length+13<256 && stub[0]==0x50 && stub[1]==0x9f);
    const u32 bits[]={0,2,4,6,7,11};
    for(u32 combination=0;combination<64;combination++) {
        u32 flags=0,in[9]={0},out[9];
        for(u32 i=0;i<6;i++)if(combination&(1u<<i))flags|=1u<<bits[i];
        in[8]=0xabcd0000u|((flags&0xd5u)<<8)|((flags>>11)&1u);
        copy_register_image(out,in);assert(out[3]==4 && out[8]==flags);
    }
    if(argc>=2) {
        reset();capture();FILE *file=fopen(argv[1],"wb");assert(file);
        assert(fwrite(output,1,sizeof output,file)==sizeof output);assert(!fclose(file));
        if(argc==3){file=fopen(argv[2],"wb");assert(file);
            assert(fwrite(context_output,1,sizeof context_output,file)==sizeof context_output);assert(!fclose(file));}
    }
    printf("restore callbacks: success, %u read refusals, bounds, writes and single-shot checks\n",2*total);
}
