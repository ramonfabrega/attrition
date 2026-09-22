/* Authored post-return observer inputs. No original executable bytes. */
#include <assert.h>
#include <stdint.h>
#include <stddef.h>
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
typedef struct {
    u32 magic,version,frame,unit;
    u32 before_regs[9],before_stack[4],dependencies[9];
    u32 after_regs[9],after_stack[12],after_modes[2];
    u32 flags_mask,registry[4];
} RestoreProbe;
static RestoreProbe restore_probe;
static int g_frame=230;
static u8 unit[344],path[32],output[692];
#ifdef RON_RESTORE_LIMIT95
static u32 mode_word=300,save_word=1,payload_status,payload_copied=4,mode_read_fail,trailer_fail,trailer_short;
static struct {u32 count,bytes,frame,unit;} payload_header={1,4,230,0x14000};
#define RESTORE_LIMIT_WORD mode_word
#endif
static u32 reads,fail_at,mutate,write_fail,write_short,create_fail,good,bad,closed;
static void emit(u32 k,u32 tag,u32 a,u32 b,u32 c,u32 d,u32 e){
    (void)k;(void)b;(void)d;(void)e;
    #ifdef RON_RESTORE_LIMIT95
    if(tag==183 || tag==184)return;
    if(tag==185){bad++;return;}
    const u32 trailer=32;
#else
    const u32 trailer=0;
#endif
    if(tag==181){assert(a==0 && c==628+((u32 *)unit)[0xbc/4]*16+trailer);good++;}else {assert(tag==182);bad++;}
}
static void flush(void){}
static void path_join(char *p,const char *n){strcpy(p,n);}
static HANDLE CreateFileA(const char *p,u32 a,u32 b,void *c,u32 d,u32 e,void *f){
    (void)a;(void)b;(void)c;(void)d;(void)e;(void)f;
    assert(!strcmp(p,"restore-poststate.bin"));return create_fail?INVALID_HANDLE:(HANDLE)1;
}
static i32 WriteFile(HANDLE h,const void *p,u32 n,u32 *written,void *x){
    (void)h;(void)x;assert(n<=sizeof output);
    if(n==32)memcpy(output+660,p,n);else memcpy(output,p,n);
#ifdef RON_RESTORE_LIMIT95
    if(n==32){*written=n-trailer_short;return !trailer_fail;}
#endif
    *written=n-write_short;return !write_fail;
}
static void CloseHandle(HANDLE h){(void)h;closed++;}
static int restore_read(u32 a,void *out,u32 n){
#ifdef RON_RESTORE_LIMIT95
    if(a==0xe85ec0 && n==8){if(mode_read_fail)return 0;((u32 *)out)[0]=mode_word;((u32 *)out)[1]=save_word;return 1;}
#endif
    reads++;if(reads==fail_at)return 0;
    if(a==0x14000 && n==344){memcpy(out,unit,n);if(mutate && reads==3)((u8 *)out)[42]^=1;return 1;}
    if(a==0x20000 && n==32){memcpy(out,path,n);return 1;}
    return 0;
}
#include "register_image_stub.h"
#include "live_restore_poststate.h"
static u32 regs[9];
static void reset(void){
    memset(&restore_probe,0,sizeof restore_probe);memset(unit,0,sizeof unit);memset(regs,0,sizeof regs);
    restore_probe.magic=0x31545352;restore_probe.version=2;restore_probe.frame=230;restore_probe.unit=0x14000;
    u32 before[]={7,8,9,0x50004000,10,11,12,13,0};memcpy(restore_probe.before_regs,before,sizeof before);
    u32 args[]={0x12345678,0x140b8,0,1};memcpy(restore_probe.before_stack,args,sizeof args);
    u32 deps[]={0x11000,0x12000,0x13000,0x14000,2,4000^0x63637,8000^0x63637,125,0};
    memcpy(restore_probe.dependencies,deps,sizeof deps);
    u32 after[]={7,0,0x50003ffc,0x50003fe0,1,8000,4000,13,0};memcpy(restore_probe.after_regs,after,sizeof after);
    u32 stack[]={0x140b8,4000,8000,0,1,0,8,9,0x12345678,0x140b8,0,1};
    memcpy(restore_probe.after_stack,stack,sizeof stack);restore_probe.after_modes[0]=75;restore_probe.after_modes[1]=1;
    restore_probe.flags_mask=0x8d5;restore_probe.registry[0]=2;restore_probe.registry[1]=2;restore_probe.registry[3]=0x15000;
    unit[10]=1;((u32 *)unit)[0xb8/4]=0x20000;((u32 *)unit)[0xbc/4]=2;((u32 *)unit)[0xc0/4]=1;
    for(u32 i=0;i<32;i++)path[i]=(u8)i;
    regs[3]=0x50003ff4;regs[7]=8;restore_post_pending=1;g_frame=230;
    reads=fail_at=mutate=write_fail=write_short=create_fail=good=bad=closed=0;
#ifdef RON_RESTORE_LIMIT95
    restore_limit_active=0;mode_word=300;save_word=1;payload_status=0;
    payload_header.count=1;payload_header.bytes=4;payload_header.frame=230;payload_header.unit=0x14000;
    payload_copied=4;restore_probe.after_modes[0]=300;mode_read_fail=trailer_fail=trailer_short=0;
#endif
}
static void run(void){restore_returned(regs);assert(!restore_post_pending);}
int main(int argc,char **argv){
    assert(sizeof(RestoreProbe)==216 && offsetof(__typeof__(restore_poststate),path_data)==628);
#ifdef RON_RESTORE_LIMIT95
    reset();assert(restore_limit_apply() && mode_word==95);
    run();assert(mode_word==300 && good==1 && !bad);
    if(argc==2){FILE *f=fopen(argv[1],"wb");assert(f);assert(fwrite(output,1,692,f)==692);fclose(f);}
    assert(!restore_limit_finish());
    for(u32 i=1;i<=3;i++){reset();assert(restore_limit_apply());fail_at=i;run();assert(mode_word==300 && bad && !good);}
    reset();assert(restore_limit_apply());create_fail=1;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());write_short=1;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());write_fail=1;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());g_frame++;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());mode_word=96;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());save_word=2;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());mode_read_fail=1;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());trailer_fail=1;run();assert(mode_word==300 && bad && !good);
    reset();assert(restore_limit_apply());trailer_short=1;run();assert(mode_word==300 && bad && !good);
    for(u32 i=0;i<8;i++){
        reset();
        switch(i){case 0:payload_status=1;break;case 1:payload_header.count=0;break;
        case 2:payload_copied=3;break;case 3:payload_header.frame++;break;
        case 4:payload_header.unit++;break;case 5:restore_probe.after_modes[0]=75;break;
        case 6:save_word=0;break;case 7:g_frame++;break;}
        assert(!restore_limit_apply() && mode_word==300 && bad && !restore_limit_active);
    }
    puts("limit95: packet, input refusals and restoration before observer failures pass");return 0;
#endif
    reset();run();assert(good==1 && !bad && reads==3 && closed==1);
    if(argc==2){FILE *f=fopen(argv[1],"wb");assert(f);assert(fwrite(output,1,660,f)==660);fclose(f);}
    run();assert(good==1 && reads==3);
    for(u32 i=1;i<=3;i++){reset();fail_at=i;run();assert(bad==1 && !good && reads==i);}
    reset();mutate=1;run();assert(bad && !good);
    reset();write_fail=1;run();assert(bad && closed==1 && !good);
    reset();write_short=1;run();assert(bad && closed==1 && !good);
    reset();create_fail=1;run();assert(bad && !closed && !good);
    reset();g_frame++;run();assert(bad && !reads);
    reset();regs[3]++;run();assert(bad && !reads);
    reset();unit[9]=1;run();assert(bad && reads==1);
    reset();((u32 *)unit)[0xbc/4]=4097;run();assert(bad && reads==1);
    reset();((u32 *)unit)[0xc0/4]=3;run();assert(bad && reads==1);
    reset();((u32 *)unit)[0xb8/4]=0xfffffff0;run();assert(bad && !good);
    reset();((u32 *)unit)[0xbc/4]=0;((u32 *)unit)[0xc0/4]=0;run();assert(good && !bad && reads==2);
    puts("poststate: success, one-shot, read failures, identity, extent, consistency and file controls pass");
}
