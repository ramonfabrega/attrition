#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef unsigned int u32;typedef int i32;typedef void *HANDLE;
#define K_INFO 5
static int g_frame=224;static HANDLE g_proc;
static u32 events[4096][8],count,reads,fail_at,short_at;
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e) {
 assert(count<4096);u32 row[]={k,tag,a,b,c,d,e,(u32)g_frame};memcpy(events[count++],row,sizeof row);
}
static int ReadProcessMemory(HANDLE h,const void *ptr,void *out,u32 n,u32 *copied) {
 (void)h;u32 address=(u32)(uintptr_t)ptr;u32 data[86]={0};reads++;
 if(reads==fail_at){*copied=0;return 0;}
 if(address==0xe85e80) {assert(n==136);for(u32 i=0;i<5;i++)data[i]=0x30000+i*0x100;data[5]=0x20000;}
 else if(address==0x20000){assert(n==344);}
 else if(address>=0x30000 && address<=0x30400){assert(n==24 || n==28);data[2]=1;data[3]=0x40000;}
 else {assert((address==0xc8d9b0 || address==0xc8d820) && n==16);data[0]=0x50000;data[1]=64;data[2]=2;data[3]=64;}
 memcpy(out,data,n);*copied=n-(reads==short_at);return 1;
}
#include "live_shared_search.h"
static void reset(void){memset(&shared_search,0,sizeof shared_search);count=reads=fail_at=short_at=0;g_frame=224;}
static void pair(void) {
 emit(7,0,0xe85e40,0x200b8,48,0,0);shared_enter(0xe85e40,0x200b8);
 emit(8,0,0,0,0,0,0xffffffff);shared_return(0);
}
int main(int argc,char **argv) {
 assert(argc==2);reset();pair();assert(!shared_search.failed && !shared_search.active);
 FILE *f=fopen(argv[1],"wb");assert(f);assert(fwrite(events,32,count,f)==count);fclose(f);
 for(u32 i=1;i<=18;i++) {
  reset();fail_at=i;pair();assert(shared_search.failed);
  assert(events[count-1][1]==204 || events[count-2][1]==204);
  reset();short_at=i;pair();assert(shared_search.failed);
 }
 reset();g_frame=223;shared_enter(0xe85e40,0x200b8);shared_return(0);assert(!count && !reads);
 reset();shared_enter(0xe85e40,0x200b8);shared_enter(0xe85e40,0x200b8);assert(shared_search.failed);
 reset();shared_enter(0xe85e40,0x200b8);g_frame=225;shared_return(0);assert(shared_search.failed);
 reset();shared_enter(0xe85e40,0x201b8);assert(shared_search.failed);
 reset();for(int i=0;i<9;i++)pair();assert(shared_search.failed && shared_search.count==8);
 return 0;
}
