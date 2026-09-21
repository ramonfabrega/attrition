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
