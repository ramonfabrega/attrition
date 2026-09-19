/* First native restore entry and the wrapper's delegation boundary.
 * No return replacement, search-state writes, or allocator adapter. */
#include "register_image_stub.h"
typedef struct {
    u32 magic, version, frame, unit;
    u32 before_regs[9], before_stack[4], dependencies[9];
    u32 after_regs[9], after_stack[12], after_modes[2];
    u32 flags_mask, registry[4];
} RestoreProbe;
static RestoreProbe restore_probe;
static int restore_claimed, restore_pending;
static int restore_read(u32 address, void *out, u32 size) {
    u32 copied=0;
    if (address<0x10000u || address>0xffffffffu-size ||
        !ReadProcessMemory(g_proc,(void *)address,out,size,&copied) || copied!=size) {
        emit(K_INFO,163,1,address,size,copied,0);restore_pending=0;return 0;
    }
    return 1;
}
static void __cdecl restore_enter(u32 *regs) {
    if (restore_claimed || g_frame<0 || g_frame>1400) return;
    restore_claimed=1;
    restore_probe.magic=0x31545352;restore_probe.version=2;restore_probe.frame=(u32)g_frame;
    restore_probe.flags_mask=0x8d5;
    copy_register_image(restore_probe.before_regs,regs);
    u32 *args=restore_probe.before_stack, *d=restore_probe.dependencies;
    if (!restore_read(restore_probe.before_regs[3],args,16)) return;
    u32 owner=args[2],id=args[3];
    if (owner>=8 || id>=512) {emit(K_INFO,163,2,owner,id,0,0);return;}
    if (!restore_read(0xc061bc,d,4) || !restore_read(0xc0618c,d+1,4) ||
        !restore_read(d[0]+owner*4,d+4,4) ||
        !restore_read(d[1]+0x14+owner*0x1c,d+2,4) ||
        !restore_read(d[2]+id*4,d+3,4) || !restore_read(d[3]+0x10,d+5,8) ||
        !restore_read(0xe85ec0,d+7,8)) return;
    u32 *registry=restore_probe.registry,unit;
    if (!restore_read(0xc0aeb4+owner*0x1c,registry,16)) return;
    if (id>=registry[0] || registry[0]>registry[1] || registry[1]>32768 || !registry[3]) {
        emit(K_INFO,163,3,id,registry[0],registry[1],0);return;
    }
    if (!restore_read(registry[3]+id*4,&unit,4)) return;
    restore_probe.unit=unit;
    capture_search_graph(unit);
    if (graph_failed) return;
    restore_pending=1;
    emit(K_INFO,161,unit,owner,id,args[1],d[4]);
}
static void __cdecl restore_delegate(u32 *regs) {
    if (!restore_pending) return;
    restore_pending=0;
    copy_register_image(restore_probe.after_regs,regs);
    if (!restore_read(restore_probe.after_regs[3],restore_probe.after_stack,48) ||
        !restore_read(0xe85ec0,restore_probe.after_modes,8)) return;
    char path[320];path_join(path,"restore-prefix.bin");
    HANDLE file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    u32 written=0;
    i32 ok=file!=INVALID_HANDLE && WriteFile(file,&restore_probe,sizeof restore_probe,&written,0);
    if(file!=INVALID_HANDLE)CloseHandle(file);
    emit(K_INFO,162,ok && written==sizeof restore_probe?0:1,restore_probe.unit,
         written,sizeof restore_probe,restore_probe.after_modes[0]);
    flush();
}
static u32 restore_callback(u8 *s,void *fn) {
    return build_register_image_stub(s,(u32)fn);
}
static void restore_jump(u8 *p,u32 target,u8 opcode) {
    p[0]=opcode;*(u32 *)(p+1)=target-((u32)p+5);
}
static void install_restore_probe(void) {
    const u8 entry[]={0x55,0x8b,0xec,0xa1,0xbc,0x61,0xc0,0x00};
    const u8 delegate[]={0xe8,0x86,0x9f,0xff,0xff};
    if(g_base!=0x400000u || g_cover) {emit(K_INFO,163,4,g_base,(u32)g_cover,0,0);return;}
    u8 *a=(u8 *)0x688f40,*b=(u8 *)0x688fa5;
    for(u32 i=0;i<sizeof entry;i++) if(a[i]!=entry[i]) {emit(K_INFO,163,5,i,a[i],entry[i],0);return;}
    for(u32 i=0;i<sizeof delegate;i++) if(b[i]!=delegate[i]) {emit(K_INFO,163,6,i,b[i],delegate[i],0);return;}
    u8 *stub=VirtualAlloc(0,4096,MEM_COMMIT|MEM_RESERVE,PAGE_EXECUTE_READWRITE);
    if(!stub){emit(K_INFO,163,7,0,0,0,0);return;}
    u32 n=restore_callback(stub,(void *)restore_enter);
    memcpy(stub+n,a,sizeof entry);n+=sizeof entry;restore_jump(stub+n,(u32)a+sizeof entry,0xe9);
    n=256+restore_callback(stub+256,(void *)restore_delegate);
    restore_jump(stub+n,0x682f30,0xe8);n+=5;restore_jump(stub+n,0x688faa,0xe9);
    restore_jump(a,(u32)stub,0xe9);restore_jump(b,(u32)stub+256,0xe9);
    FlushInstructionCache(g_proc,stub,4096);FlushInstructionCache(g_proc,a,0x6a);
    emit(K_INFO,160,2,0x688f40,0x688fa5,0x682f30,0);
}
