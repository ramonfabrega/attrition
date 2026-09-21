/* Opt-in metadata only. No process-memory payload, thread suspension or dump API.
 * Win32 MEMORY_BASIC_INFORMATION32 is seven DWORDs; SYSTEM_INFO is nine DWORDs
 * on this 32-bit target. See docs/lab/2026-09-21-memory-inventory.md. */
IMPORT(u32, VirtualQueryEx, (HANDLE, const void *, void *, u32));
IMPORT(void, GetSystemInfo, (void *));
IMPORT(u32, GetTickCount, (void));
#define INVENTORY_MAX 8192u
#define INVENTORY_MS 2000u
#define INVENTORY_MAGIC 0x31494d52u
typedef struct {
    u32 base,allocation,allocation_protect,size,state,protect,type;
} InventoryRange;
typedef struct {
    u32 magic,version,frame,unit,begin,end,page,record_bytes;
    u32 count,status,elapsed,cursor,main_base,observer,max_records,max_ms;
    RestoreProbe prefix;
    InventoryRange ranges[INVENTORY_MAX];
} MemoryInventory;
_Static_assert(sizeof(InventoryRange)==28,"inventory range wire size");
_Static_assert(__builtin_offsetof(MemoryInventory,ranges)==280,"inventory header wire size");
static MemoryInventory memory_inventory;

static void capture_memory_inventory(u32 observer_address) {
    MemoryInventory *m=&memory_inventory;
    u32 system[9]={0},started=GetTickCount();
    m->magic=INVENTORY_MAGIC;m->version=1;m->frame=restore_probe.frame;m->unit=restore_probe.unit;
    m->count=m->status=0;m->record_bytes=sizeof(InventoryRange);
    m->main_base=g_base;m->observer=observer_address;m->max_records=INVENTORY_MAX;m->max_ms=INVENTORY_MS;
    memcpy(&m->prefix,&restore_probe,sizeof restore_probe);
    GetSystemInfo(system);
    m->begin=system[2];m->end=system[3]+1;m->page=system[1];m->cursor=m->begin;
    if ((system[0]&0xffffu)!=0 || m->begin<0x10000 || m->begin>=m->end ||
        !m->page || m->page>65536 || (m->page&(m->page-1)) ||
        m->begin%m->page || m->end%m->page) m->status=1;
    while (!m->status && m->cursor<m->end) {
        if (GetTickCount()-started>=INVENTORY_MS) {m->status=5;break;}
        if (m->count==INVENTORY_MAX) {m->status=4;break;}
        InventoryRange r={0};
        if (VirtualQueryEx(g_proc,(const void *)m->cursor,&r,sizeof r)!=sizeof r) {
            m->status=2;break;
        }
        if (r.base!=m->cursor || !r.size || r.size>0xffffffffu-r.base ||
            r.base%m->page || r.size%m->page ||
            (r.state!=0x1000u && r.state!=0x2000u && r.state!=0x10000u)) {
            m->status=3;break;
        }
        m->ranges[m->count++]=r;
        u32 next=r.base+r.size;m->cursor=next<m->end?next:m->end;
    }
    m->elapsed=GetTickCount()-started;
    if (!m->status && m->elapsed>=INVENTORY_MS) m->status=5;
    char path[320];path_join(path,"memory-inventory.bin");
    HANDLE file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    u32 size=(u32)((u8 *)m->ranges-(u8 *)m)+m->count*sizeof(InventoryRange),written=0;
    i32 ok=file!=INVALID_HANDLE && WriteFile(file,m,size,&written,0);
    if (file!=INVALID_HANDLE)CloseHandle(file);
    if (!ok || written!=size) {emit(K_INFO,166,6,m->cursor,m->count,written,size);return;}
    if (m->status) emit(K_INFO,166,m->status,m->cursor,m->count,m->elapsed,0);
    else emit(K_INFO,165,0,m->unit,size,m->count,m->elapsed);
}
