#include "restore_packet_path.h"
/* Opt-in post-return observation. Own bounded storage. Limit intervention requires its own opt-in.
 * Included after RestoreProbe, restore_read and the register-image adapter.
 * INFO 181 success; INFO 182 failure. Neither implies pre-payload success. */
#ifndef RON_RESTORE_SECOND
#ifndef restore_sequence_fail
#define restore_sequence_fail(reason) ((void)0)
#endif
#endif
#define RESTORE_PATH_CAP 4096u
#define RESTORE_POST_FIXED 628u
static struct {
    u32 magic,version,frame,unit,prefix_bytes,path_bytes,flags_mask,boundary;
    RestoreProbe prefix;
    u32 registers[9];
    u8 unit_data[0x158],path_data[RESTORE_PATH_CAP*16u];
} restore_poststate;
_Static_assert(sizeof restore_poststate==RESTORE_POST_FIXED+RESTORE_PATH_CAP*16u,
               "post-state packet layout differs");
static int restore_post_pending;
#ifdef RON_RESTORE_LIMIT95
#include "live_restore_limit.h"
#endif
#ifdef RON_RESTORE_POSTGRAPH
#include "live_restore_postgraph.h"
#endif
static void __cdecl restore_returned(u32 *regs) {
    if (!restore_post_pending) return;
    restore_post_pending=0;
#ifdef RON_RESTORE_LIMIT95
    /* Runs before every observer failure path; never depend on file success. */
    int with_limit=1;
#ifdef RON_RESTORE_SECOND
    with_limit=restore_packet_index==0;
#endif
    int limit_ok=with_limit?restore_limit_finish():!restore_limit_active;
#endif
    u32 fail=1,written=0,size=0,capacity,length,pointer;
    u8 check[0x158];
    copy_register_image(restore_poststate.registers,regs);
#ifdef RON_RESTORE_SECOND
    u32 observed_modes[2]={0,0},modes_read=0;
#endif
    restore_poststate.magic=0x31505352;restore_poststate.version=1;
#ifdef RON_RESTORE_LIMIT95
    if(with_limit)restore_poststate.version=2;
    fail=6;if(!limit_ok)goto failed;
#ifdef RON_RESTORE_SECOND
    if(!with_limit) {
        modes_read=restore_read(0xe85ec0,observed_modes,8);
        fail=7;if(!modes_read)goto failed;
        /* Modes are callee outputs: native completion can clear saving. */
        restore_poststate.version=3;
    }
#endif
#endif
    restore_poststate.frame=restore_probe.frame;restore_poststate.unit=restore_probe.unit;
    restore_poststate.prefix_bytes=sizeof restore_probe;restore_poststate.flags_mask=0x8d5;
    restore_poststate.boundary=0x688faa;
    memcpy(&restore_poststate.prefix,&restore_probe,sizeof restore_probe);
    fail=9;if((u32)g_frame!=restore_probe.frame)goto failed;
    fail=10;if(restore_probe.after_regs[3]>0xffffffffu-24u)goto failed;
    fail=11;if(restore_poststate.registers[3]!=restore_probe.after_regs[3]+24u)goto failed;
    fail=2;
    if (!restore_read(restore_probe.unit,restore_poststate.unit_data,sizeof check)) goto failed;
    if (restore_poststate.unit_data[9]!=restore_probe.before_stack[2] ||
        *(unsigned short *)(restore_poststate.unit_data+10)!=restore_probe.before_stack[3]) goto failed;
    pointer=*(u32 *)(restore_poststate.unit_data+0xb8);
    capacity=*(u32 *)(restore_poststate.unit_data+0xbc);
    length=*(u32 *)(restore_poststate.unit_data+0xc0);
    fail=3;
    if (capacity>RESTORE_PATH_CAP || length>capacity) goto failed;
    restore_poststate.path_bytes=capacity*16u;
    if (capacity && !restore_read(pointer,restore_poststate.path_data,capacity*16u)) goto failed;
    fail=4;
    if (!restore_read(restore_probe.unit,check,sizeof check)) goto failed;
    for(u32 i=0;i<sizeof check;i++)if(check[i]!=restore_poststate.unit_data[i])goto failed;
    fail=5;
    char path[320];restore_packet_path(path,"restore-poststate.bin");
    HANDLE file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    size=RESTORE_POST_FIXED+restore_poststate.path_bytes;
    if(file==INVALID_HANDLE)goto failed;
    i32 ok=WriteFile(file,&restore_poststate,size,&written,0);
#ifdef RON_RESTORE_LIMIT95
    u32 trailer_written=0;
    if(with_limit) {
    if(ok && written==size) {
        ok=WriteFile(file,&restore_limit,sizeof restore_limit,&trailer_written,0);
        written+=trailer_written;
    } else ok=0;
    size+=sizeof restore_limit;
    }
#endif
#ifdef RON_RESTORE_SECOND
    if(!with_limit) {
        u32 modes_written=0;
        if(ok && written==size) {
            ok=WriteFile(file,observed_modes,sizeof observed_modes,&modes_written,0);
            written+=modes_written;
        } else ok=0;
        size+=sizeof observed_modes;
    }
#endif
    CloseHandle(file);
    if(!ok || written!=size)goto failed;
#ifdef RON_RESTORE_SECOND
    if(!with_limit)emit(K_INFO,195,restore_probe.unit,observed_modes[0],observed_modes[1],size,restore_poststate.registers[7]);
#endif
    emit(K_INFO,181,0,restore_probe.unit,size,capacity,restore_poststate.registers[7]);
    flush();
#ifdef RON_RESTORE_SECOND
    int suspended=1;
    for(u32 i=0;i<5;i++)if(!*(u32 *)(restore_poststate.unit_data+0x104+4*i))suspended=0;
    if(!restore_sequence_return(&restore_sequence,restore_probe.unit,(u32)g_frame,
                               restore_poststate.registers[7],suspended)) {
        restore_sequence_fail(8);return;
    }
    emit(K_INFO,191,restore_packet_index,restore_probe.unit,restore_poststate.registers[7],0,0);flush();
#endif
#ifdef RON_RESTORE_POSTGRAPH
    capture_restore_postgraph(size);
#endif
    return;
failed:
#ifdef RON_RESTORE_SECOND
    /* Failure-only observations; never manufacture a successful post packet.
     * INFO 193: index, unit, actual ESP, delegation ESP, outer return.
     * INFO 194: index, reason, mode read succeeded, limit, saving.
     * The trace frame is the actual callback frame in both records. */
    emit(K_INFO,193,restore_packet_index,restore_probe.unit,
         restore_poststate.registers[3],restore_probe.after_regs[3],restore_poststate.registers[7]);
    emit(K_INFO,194,restore_packet_index,fail,modes_read,observed_modes[0],observed_modes[1]);
#endif
    emit(K_INFO,182,fail,restore_probe.unit,written,size,0);restore_sequence_fail(9);flush();
}
