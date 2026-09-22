/* Authored graph + post-state fixtures; exercises the actual postgraph producer. */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned char u8;
typedef unsigned int u32;
typedef int i32;
typedef void *HANDLE;
#define K_INFO 5
#define INVALID_HANDLE ((HANDLE)(intptr_t)-1)
#define GENERIC_WRITE 1
#define FILE_SHARE_READ 1
#define FILE_ATTRIBUTE_NORMAL 1
#define IMPORT(t,n,a) static t n a
#define RON_RESTORE_POSTGRAPH
#define RESTORE_POST_FIXED 628u
static HANDLE g_proc;
static u32 g_frame, ticks, tick_step, reads, fail_read, writes, fail_write, short_write, fail_create, good, bad, closed, mutate_last;
static u32 source[65536], output[90000], source_bytes, output_bytes;
static struct {u32 frame,unit;} restore_probe;
static struct {u32 header[8];u8 prefix[216];u32 regs[9];u8 unit_data[344],path_data[65536];} restore_poststate;
#ifdef RON_RESTORE_LIMIT95
static u32 restore_limit[8];
#endif
static u32 GetTickCount(void){u32 t=ticks;ticks+=tick_step;return t;}
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e){
    (void)k;(void)a;(void)b;(void)d;(void)e;
    if(tag==186){assert(c==output_bytes);good++;}else{assert(tag==187);bad++;}
}
static void flush(void){}
static void path_join(char *p,const char *n){strcpy(p,n);}
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f){
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    assert(!strcmp(p,"restore-postgraph.bin"));return fail_create?INVALID_HANDLE:(HANDLE)1;
}
static i32 WriteFile(HANDLE h,const void *p,u32 n,u32 *written,void *x){
    (void)h;(void)x;writes++;assert(output_bytes+n<=sizeof output);
    memcpy((u8 *)output+output_bytes,p,n);output_bytes+=n;
    *written=n-(writes==short_write);return writes!=fail_write;
}
static void CloseHandle(HANDLE h){(void)h;closed++;}
static i32 ReadProcessMemory(HANDLE h,const void *pointer,void *out,u32 size,u32 *copied){
    (void)h;u32 address=(u32)(uintptr_t)pointer;reads++;if(reads==fail_read){*copied=0;return 0;}
    for(u32 at=32;at<source_bytes;){u32 *p=source+at/4;at+=16+p[3];
        if(address==p[2] && size==p[3]){memcpy(out,p+4,size);if(mutate_last && address==restore_probe.unit+0x104 && reads>1)((u8 *)out)[40]^=1;*copied=size;return 1;}}
    *copied=0;return 0;
}
#include "live_search_graph.h"
/* The real producer uses this named field. */
#define path_bytes header[5]
#include "live_restore_postgraph.h"
#undef path_bytes
static void reset(void){ticks=tick_step=reads=fail_read=writes=fail_write=short_write=fail_create=good=bad=closed=output_bytes=mutate_last=0;}
static void run(u32 n){capture_restore_postgraph(n);assert(!graph_timed && graph_failure_tag==151);}
int main(int argc,char **argv){
    assert(argc==4);FILE *f=fopen(argv[1],"rb");assert(f);source_bytes=fread(source,1,sizeof source,f);fclose(f);
    f=fopen(argv[2],"rb");assert(f);u32 n=fread(&restore_poststate,1,sizeof restore_poststate,f);fclose(f);
#ifdef RON_RESTORE_LIMIT95
    assert(n>=660);memcpy(restore_limit,(u8 *)&restore_poststate+n-32,32);
#endif
    restore_probe.frame=g_frame=restore_poststate.header[2];restore_probe.unit=restore_poststate.header[3];
    reset();run(n);assert(good==1 && !bad && closed==1);u32 total_reads=reads,total_writes=writes;
    f=fopen(argv[3],"wb");assert(f);assert(fwrite(output,1,output_bytes,f)==output_bytes);fclose(f);
    for(u32 i=1;i<=total_reads;i++){reset();fail_read=i;run(n);assert(bad && !good && !writes);}
    for(u32 i=1;i<=total_writes;i++)for(u32 shortness=0;shortness<2;shortness++){
        reset();if(shortness)short_write=i;else fail_write=i;run(n);assert(bad && !good && closed==1);}
    reset();fail_create=1;run(n);assert(bad && !good && !closed);
    reset();tick_step=1000;run(n);assert(bad && !good && !writes);
    reset();ticks=0xfffffff0u;run(n);assert(good && !bad);
    reset();g_frame++;run(n);assert(bad && !good && !writes);g_frame--;
    reset();restore_poststate.unit_data[0x140]^=1;run(n);assert(bad && !good && !writes);restore_poststate.unit_data[0x140]^=1;
    reset();mutate_last=1;run(n);assert(bad && !good && !writes);
    reset();graph_records=4096;graph_failure_tag=187;
    assert(!graph_record(1,0,restore_probe.unit+0x104,72) && bad && !reads);graph_failure_tag=151;
    reset();run(n+4);assert(bad && !good && closed==1);
    puts("postgraph: paired output, every read/write failure, deadline, identity and restoration pass");
}
