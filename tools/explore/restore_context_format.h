#define RESTORE_TABLE_MAX (4096u*192u)
typedef struct {
    u32 magic,version,frame,unit,prefix_bytes,table_bytes,teb,origin,center;
    u32 tib[7];
    RestoreProbe prefix;
    u8 fxsave[512];
    u8 unit_data[0x158]; /* Unit/UnitData extent in the matched PDB. */
    u8 table[RESTORE_TABLE_MAX];
} RestoreContext;
