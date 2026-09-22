#include "restore_packet_path.h"
/* Post-return graph envelope, opt-in only. Embedded post-state binds the event.
 * INFO 186 success, 187 failure. Restored limit and unit/path receipt precede us.
 * Acquisition deadline is checked between reads, not an OS I/O cancellation. */
static void capture_restore_postgraph(u32 post_bytes) {
    u32 fail=1,written=0,total=0,elapsed=0;
    HANDLE file=INVALID_HANDLE;
    graph_failure_tag=187;graph_timed=1;graph_started=GetTickCount();
    if (!collect_search_graph(restore_probe.unit)) {
        if (!graph_failed) emit(K_INFO,187,1,restore_probe.unit,0,0,0);
        goto done;
    }
    u8 final_unit[72];
    if (!graph_read(restore_probe.unit+0x104,final_unit,sizeof final_unit)) goto done;
    fail=12;
    if ((u32)g_frame!=restore_probe.frame) goto failed;
    /* The unit view is first in the graph. Bind every byte to the paired witness. */
    for(u32 i=0;i<72;i++)
        if (((u8 *)graph_storage)[48+i]!=restore_poststate.unit_data[0x104+i] ||
            final_unit[i]!=restore_poststate.unit_data[0x104+i]) goto failed;
    elapsed=GetTickCount()-graph_started;
    if (elapsed>=2000u) {fail=11;goto failed;}
    u32 header[]={0x31504752,1,restore_probe.frame,restore_probe.unit,
                  post_bytes,graph_used,elapsed,0x688faa};
    char path[320];restore_packet_path(path,"restore-postgraph.bin");
    fail=13;
    file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    if(file==INVALID_HANDLE)goto failed;
    if(!WriteFile(file,header,sizeof header,&written,0) || written!=sizeof header)goto failed;
    total+=written;
    u32 base=RESTORE_POST_FIXED+restore_poststate.path_bytes;
    if(!WriteFile(file,&restore_poststate,base,&written,0) || written!=base)goto failed;
    total+=written;
#ifdef RON_RESTORE_LIMIT95
    if(!WriteFile(file,&restore_limit,sizeof restore_limit,&written,0) || written!=sizeof restore_limit)goto failed;
    total+=written;
#endif
    if(!WriteFile(file,graph_storage,graph_used,&written,0) || written!=graph_used)goto failed;
    total+=written;
    if(total!=32u+post_bytes+graph_used)goto failed;
    emit(K_INFO,186,0,restore_probe.unit,total,graph_records,elapsed);
    goto done;
failed:
    emit(K_INFO,187,fail,restore_probe.unit,total,written,elapsed);
done:
    if(file!=INVALID_HANDLE)CloseHandle(file);
    graph_timed=0;graph_failure_tag=151;flush();
}
