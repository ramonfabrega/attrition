/* Authored callback integration. Context/payload I/O has its own failure suite. */
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
#define RON_RESTORE_SECOND
#define RON_RESTORE_POSTSTATE
#define RON_RESTORE_LIMIT95
#define RON_MEMORY_PAYLOAD
#define LIVE_RESTORE_CONTEXT_H
static HANDLE g_proc;
static u32 g_base;
static int g_frame=224,g_cover,graph_failed;
static u32 mode_word=300,save_word=1,payload_status,payload_copied;
static struct {u32 count,bytes,frame,unit;} payload_header;
#define RESTORE_LIMIT_WORD mode_word
static u32 owner,id=1,unit_pointer=0x14000,reads,fail_read,context_fail,graph_fail;
static u32 file_fail,write_fail,write_short,return_corrupt,context_mode_corrupt;
static u32 events[512][8],event_count,post_sizes[2],prefix_sizes[2];
static u8 post_data[2][720],prefix_data[2][216],unit[344],slots[32];
static char last_path[128];
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e) {
 assert(event_count<512);u32 row[]={k,tag,a,b,c,d,e,(u32)g_frame};
 memcpy(events[event_count++],row,sizeof row);
}
static void flush(void){}
static void path_join(char *out,const char *name){strcpy(out,name);}
#include "restore_packet_path.h"
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f) {
 (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;strcpy(last_path,p);
 if(file_fail)return INVALID_HANDLE;
 const char *leaf=restore_packet_index?"second-restore-prefix.bin":"restore-prefix.bin";
 if(!strcmp(p,leaf))return (HANDLE)(uintptr_t)(1+restore_packet_index*2);
 leaf=restore_packet_index?"second-restore-poststate.bin":"restore-poststate.bin";
 assert(!strcmp(p,leaf));return (HANDLE)(uintptr_t)(2+restore_packet_index*2);
}
static int WriteFile(HANDLE h,const void *data,u32 n,u32 *w,void *unused) {
 (void)unused;u32 v=(u32)(uintptr_t)h,index=(v-1)/2;
 if(v%2){assert(n==216);memcpy(prefix_data[index],data,n);prefix_sizes[index]=n;}
 else {assert(post_sizes[index]+n<=720);memcpy(post_data[index]+post_sizes[index],data,n);post_sizes[index]+=n;}
 *w=n-(write_short?1:0);return !write_fail;
}
static void CloseHandle(HANDLE h){(void)h;}
static void *VirtualAlloc(void *p,u32 a,u32 b,u32 c){(void)p;(void)a;(void)b;(void)c;return 0;}
static void FlushInstructionCache(HANDLE h,void *p,u32 n){(void)h;(void)p;(void)n;}
static void capture_search_graph(u32 address){assert(address==0x14000);graph_failed=graph_fail;}
static int capture_restore_context(const u32 *regs) {
 (void)regs;payload_header.count=1;payload_header.bytes=4;payload_header.frame=(u32)g_frame;
 payload_header.unit=0x14000;payload_copied=4;
 if(context_mode_corrupt)mode_word=299;
 if(!context_fail && !payload_status)emit(K_INFO,167,0,0x14000,4,1,0);
 return !context_fail;
}
static int ReadProcessMemory(HANDLE h,const void *pointer,void *out,u32 n,u32 *copied) {
 (void)h;u32 a=(u32)(uintptr_t)pointer;reads++;
 if(reads==fail_read){*copied=0;return 0;}
 u32 d[12]={0};
 if(a==0x50004000){u32 args[]={0x12345678,0x140b8,owner,id};assert(n==16);memcpy(out,args,n);}
 else if(a==0x50003fe0){assert(n==48);d[0]=0x140b8;d[1]=4000^0x63637;d[2]=8000^0x63637;d[4]=id;d[8]=0x12345678;d[9]=0x140b8;d[10]=owner;d[11]=id;memcpy(out,d,n);}
 else if(a==0xe85ec0){assert(n==8);d[0]=mode_word;d[1]=save_word;memcpy(out,d,n);}
 else if(a==0x14000){assert(n==344);memcpy(out,unit,n);if(return_corrupt)((u8*)out)[9]=1;}
 else if(a==0x20000){assert(n==32);memcpy(out,slots,n);}
 else {
  if(a==0xc061bc)d[0]=0x11000;
  else if(a==0xc0618c)d[0]=0x12000;
  else if(a==0x11000)d[0]=2;
  else if(a==0x12014)d[0]=0x13000;
  else if(a==0x13004)d[0]=0x14000;
  else if(a==0x14010){assert(n==8);d[0]=4000;d[1]=8000;}
  else if(a==0xc0aeb4){assert(n==16);d[0]=2;d[1]=2;d[3]=0x15000;}
  else if(a==0x15004)d[0]=unit_pointer;
  else assert(!"unexpected memory read");
  memcpy(out,d,n);
 }
 *copied=n;return 1;
}
#include "live_restore_probe.h"
static u32 before[]={0,0,0,0x50003ffc,0,0,0,0,0x202};
static u32 after[]={0,0,0,0x50003fdc,0,0,0,0,0x202};
static void reset(void) {
 memset(&restore_sequence,0,sizeof restore_sequence);memset(&restore_probe,0,sizeof restore_probe);
 memset(&restore_limit,0,sizeof restore_limit);memset(unit,0,sizeof unit);
 memset(post_sizes,0,sizeof post_sizes);memset(prefix_sizes,0,sizeof prefix_sizes);
 restore_packet_index=restore_claimed=restore_pending=restore_post_pending=restore_limit_active=0;
 owner=0;id=1;unit_pointer=0x14000;mode_word=300;save_word=1;g_frame=224;
 reads=fail_read=context_fail=graph_fail=graph_failed=payload_status=0;
 file_fail=write_fail=write_short=return_corrupt=context_mode_corrupt=event_count=0;
 unit[10]=1;u32 path[]={0x20000,2,1};memcpy(unit+0xb8,path,sizeof path);
 for(u32 i=0;i<5;i++){u32 p=0x30000+32*i;memcpy(unit+0x104+4*i,&p,4);}
}
static void enter_delegate(void){restore_enter(before);restore_delegate(after);if(restore_post_pending)emit(7,0,0,0x140b8,48,0,0);}
static void finish(u32 result){if(restore_post_pending)emit(8,0,result==8?1:result,0,0,0,0);u32 regs[]={0,0,0,0x50003ff4,0,0,0,result,0x202};restore_returned(regs);}
static void diagnostic(u32 reason,u32 read_ok,u32 limit,u32 saving) {
 assert(event_count>=4);
 u32 *r=events[event_count-4],*m=events[event_count-3];
 assert(r[1]==193 && r[2]==1 && r[3]==0x14000 && r[4]==0x50003ff8 && r[5]==restore_probe.after_regs[3] && r[6]==8);
 assert(m[1]==194 && m[2]==1 && m[3]==reason && m[4]==read_ok && m[5]==limit && m[6]==saving);
 assert(events[event_count-2][1]==182 && events[event_count-2][2]==reason);
 assert(events[event_count-1][1]==192 && post_sizes[1]==0);
}
static void first(void){enter_delegate();assert(mode_word==95);finish(0xffffffffu);assert(mode_word==300 && restore_sequence.stage==2);}
int main(int argc,char **argv) {
 reset();first();u32 saved_first=post_sizes[0],first_reads=reads;
 owner=1;restore_enter(before);assert(reads==first_reads+1 && restore_sequence.stage==2);
 restore_delegate(after);emit(7,0,0,0x160b8,48,0,0);emit(8,0,1,0,0,0,0);finish(8);
 assert(restore_sequence.stage==2 && !restore_post_pending);owner=0;
 g_frame=225;enter_delegate();assert(mode_word==300 && restore_sequence.stage==3 && restore_packet_index==1);
 memset(unit+0x104,0,20);finish(8);assert(restore_sequence.stage==4 && mode_word==300);
 assert(post_sizes[0]==saved_first && post_sizes[0]==692 && post_sizes[1]==660);
 assert(((u32 *)post_data[0])[1]==2 && ((u32 *)post_data[1])[1]==1);
 assert(prefix_sizes[0]==216 && prefix_sizes[1]==216);
 u32 count=event_count;restore_enter(before);restore_delegate(after);finish(8);assert(event_count==count);
 if(argc>=2){FILE *f=fopen(argv[1],"wb");assert(f);fwrite(post_data[0],1,post_sizes[0],f);fwrite(post_data[1],1,post_sizes[1],f);fclose(f);}
 if(argc==3){FILE *f=fopen(argv[2],"wb");assert(f);u32 header[]={0x544e4f52,2,0x400000,0,0,0,0,0};fwrite(header,1,sizeof header,f);fwrite(events,32,event_count,f);fclose(f);}
 /* Exhaust every entry/delegation read on each packet, independently. */
 for(u32 phase=0;phase<2;phase++) {
  reset();if(phase){first();g_frame=225;}u32 begin_reads=reads;enter_delegate();u32 entry_reads=reads-begin_reads;
  for(u32 i=1;i<=entry_reads;i++) {
   reset();if(phase){first();g_frame=225;}fail_read=reads+i;enter_delegate();
   assert(restore_sequence.stage==5 && !restore_post_pending && mode_word==300);
  }
  for(u32 which=0;which<3;which++) {
   reset();if(phase){first();g_frame=225;}if(which==0)file_fail=1;if(which==1)write_fail=1;if(which==2)write_short=1;
   enter_delegate();assert(restore_sequence.stage==5 && mode_word==300 && !restore_post_pending);
  }
 }
 reset();first();g_frame=225;enter_delegate();u32 saved_events=event_count;restore_delegate(after);assert(event_count==saved_events && mode_word==300);
 finish(8);saved_events=event_count;finish(8);assert(event_count==saved_events);
 reset();enter_delegate();memset(unit+0x104,0,20);finish(0xffffffffu);assert(restore_sequence.stage==5 && mode_word==300);
 for(u32 which=0;which<4;which++) {
  reset();first();u8 preserved[692];memcpy(preserved,post_data[0],692);g_frame=225;enter_delegate();
  if(which==0)file_fail=1;if(which==1)write_fail=1;if(which==2)write_short=1;if(which==3)return_corrupt=1;
  finish(8);assert(restore_sequence.stage==5 && mode_word==300 && !memcmp(preserved,post_data[0],692));
 }
 reset();enter_delegate();u32 end_reads=reads;finish(0xffffffffu);end_reads=reads-end_reads;
 for(u32 i=1;i<=end_reads;i++){reset();enter_delegate();fail_read=reads+i;finish(0xffffffffu);assert(mode_word==300 && restore_sequence.stage==5);}
 for(u32 fail=0;fail<4;fail++){reset();enter_delegate();if(fail==0)file_fail=1;if(fail==1)write_fail=1;if(fail==2)write_short=1;if(fail==3)return_corrupt=1;finish(0xffffffffu);assert(mode_word==300 && restore_sequence.stage==5);}
 reset();first();unit_pointer=0x16000;g_frame=225;restore_enter(before);assert(restore_sequence.stage==5 && !restore_pending);
 reset();first();g_frame=223;restore_enter(before);assert(restore_sequence.stage==5);
 reset();first();context_mode_corrupt=1;g_frame=225;enter_delegate();assert(restore_sequence.stage==5 && !restore_post_pending && mode_word==299);
 reset();context_fail=1;enter_delegate();assert(restore_sequence.stage==5 && mode_word==300);
 reset();payload_status=1;enter_delegate();assert(restore_sequence.stage==5 && mode_word==300);
 reset();graph_fail=1;enter_delegate();assert(restore_sequence.stage==5 && mode_word==300);
 reset();enter_delegate();finish(8);assert(restore_sequence.stage==5 && mode_word==300);
 reset();enter_delegate();restore_enter(before);finish(0xffffffffu);assert(restore_sequence.stage==5 && mode_word==300);
 reset();first();g_frame=225;enter_delegate();mode_word=299;finish(8);assert(restore_sequence.stage==5 && mode_word==299);diagnostic(8,1,299,1);
 reset();first();g_frame=225;enter_delegate();save_word=0;finish(8);diagnostic(8,1,300,0);
 reset();first();g_frame=225;enter_delegate();fail_read=reads+1;finish(8);diagnostic(7,0,0,0);
 reset();first();g_frame=225;enter_delegate();g_frame=226;finish(8);diagnostic(9,1,300,1);
 reset();first();g_frame=225;enter_delegate();restore_probe.after_regs[3]-=4;finish(8);diagnostic(11,1,300,1);
 puts("second restore: selection, isolation, first restoration and second no-intervention controls pass");
}
