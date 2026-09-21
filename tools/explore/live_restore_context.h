/* Sidecar for the same delegation event as restore-prefix.bin. Original bytes
 * remain in the capture directory. Bounds are experimental caps, not game maxima.
 * The observer is ordinary C without an SEH frame; rd_fs reads this thread.
 */
#include "restore_context_format.h"
static RestoreContext restore_context;
#if defined(RON_MEMORY_PAYLOAD) && !defined(RON_MEMORY_INVENTORY)
#error RON_MEMORY_PAYLOAD requires RON_MEMORY_INVENTORY
#endif
#ifdef RON_MEMORY_INVENTORY
#include "live_memory_inventory.h"
#endif
#ifdef RON_MEMORY_PAYLOAD
#include "live_memory_payload.h"
#endif

static int capture_restore_context(const u32 *regs) {
    RestoreContext *c=&restore_context;
    c->magic=0x31585452;c->version=2;c->frame=restore_probe.frame;c->unit=restore_probe.unit;
    c->prefix_bytes=sizeof restore_probe;memcpy(&c->prefix,&restore_probe,sizeof restore_probe);
    c->teb=rd_fs(0x18);
    u32 head=rd_fs(0),esp=restore_probe.after_regs[3];
    if (!restore_read(c->teb,c->tib,sizeof c->tib)) return 0;
    if (c->teb<0x10000u || c->teb>0xffffffffu-28u || c->tib[6]!=c->teb || c->tib[0]!=head || c->tib[2]>=esp ||
        esp>0xffffffffu-48 || esp+48>c->tib[1]) {
        emit(K_INFO,163,20,c->teb,head,esp,0);return 0;
    }
    if (!restore_read(0xcab3ac,&c->origin,4) || !restore_read(0xcae5fc,&c->center,4)) return 0;
    if (c->origin<0x10000u || c->center<=c->origin ||
        c->center-c->origin>RESTORE_TABLE_MAX/2 || (c->center-c->origin)%96u) {
        emit(K_INFO,163,21,c->origin,c->center,0,0);return 0;
    }
    c->table_bytes=2*(c->center-c->origin);
    if (c->origin>0xffffffffu-c->table_bytes) {
        emit(K_INFO,163,21,c->origin,c->center,c->table_bytes,0);return 0;
    }
    /* The emitter stores ESP-4 in word 3. Its image is 36 bytes, followed
     * downward by a 528-byte reservation rounded down to a 16-byte boundary. */
    if (regs[3]<560u) {emit(K_INFO,163,22,regs[3],0,0,0);return 0;}
    u32 fx=(regs[3]-560u)&~15u;
    if (!restore_read(fx,c->fxsave,sizeof c->fxsave) ||
        !restore_read(c->origin,c->table,c->table_bytes)) return 0;
    if (!restore_read(c->unit,c->unit_data,sizeof c->unit_data)) return 0;
    u32 owner=c->prefix.before_stack[2],id=c->prefix.before_stack[3];
    if (c->unit_data[9]!=owner || *(const unsigned short *)(c->unit_data+0xa)!=id ||
        c->unit!=c->prefix.dependencies[3] ||
        (*(const u32 *)(c->unit_data+0x10)!=c->prefix.dependencies[5] ||
         *(const u32 *)(c->unit_data+0x14)!=c->prefix.dependencies[6])) {
        emit(K_INFO,163,24,c->unit,owner,id,0);return 0;
    }
    char path[320];path_join(path,"restore-context.bin");
    HANDLE file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    u32 written=0,size=(u32)((u8 *)c->table-(u8 *)c)+c->table_bytes;
    i32 ok=file!=INVALID_HANDLE && WriteFile(file,c,size,&written,0);
    if(file!=INVALID_HANDLE)CloseHandle(file);
    if(!ok || written!=size) {emit(K_INFO,163,23,written,size,0,0);return 0;}
    emit(K_INFO,164,0,c->unit,size,c->table_bytes,c->teb);
#ifdef RON_MEMORY_INVENTORY
    int inventory_ok=capture_memory_inventory((u32)&memory_inventory);
#ifdef RON_MEMORY_PAYLOAD
    if (inventory_ok) capture_memory_payload();
#else
    (void)inventory_ok;
#endif
#endif
    return 1;
}
