/* Optional broad candidate copy. Not an atomic snapshot; original bytes stay
 * outside git. The inventory receipt must succeed before entering this code. */
#define PAYLOAD_CAP (1024u*1024u*1024u)
#define PAYLOAD_CHUNK (1024u*1024u)
#define PAYLOAD_MS 5000u
typedef struct {
    u32 magic,version,frame,unit,count,bytes,cap,max_ms,chunk,inventory_bytes;
    u32 stack_low,stack_high,observer,world,reserved0,reserved1;
} PayloadHeader;
typedef struct {u32 magic,version,count,bytes,elapsed,world,anchor_bytes,reserved;} PayloadFooter;
_Static_assert(sizeof(PayloadHeader)==64,"payload header wire size");
_Static_assert(sizeof(PayloadFooter)==32,"payload footer wire size");
static u8 payload_buffer[PAYLOAD_CHUNK];
static u32 payload_started,payload_status,payload_copied,payload_index;
static PayloadHeader payload_header;

static int payload_time(void) {
    if(GetTickCount()-payload_started>=PAYLOAD_MS){payload_status=8;return 0;}return 1;
}
static int payload_read(u32 address,void *out,u32 size) {
    u32 got=0;
    if(!payload_time())return 0;
    if(!ReadProcessMemory(g_proc,(const void *)address,out,size,&got) || got!=size){payload_status=4;return 0;}
    return payload_time();
}
static int payload_write(HANDLE file,const void *data,u32 size) {
    u32 wrote=0;
    if(!payload_time())return 0;
    if(!WriteFile(file,data,size,&wrote,0) || wrote!=size){payload_status=5;return 0;}
    return payload_time();
}
static u32 payload_extent(const InventoryRange *r) {
    return r->size>memory_inventory.end-r->base?memory_inventory.end-r->base:r->size;
}
static int payload_selected(const InventoryRange *r) {
    u32 p=r->protect&0xff;
    return r->state==0x1000 && !(r->protect&0x100) && (p==2 || p==4 || p==8) &&
        r->allocation!=payload_header.observer &&
        !(r->base<payload_header.stack_high && r->base+payload_extent(r)>payload_header.stack_low) &&
        (r->type==0x20000 || (r->type==0x1000000 && r->allocation==memory_inventory.main_base));
}
static int payload_covered(u32 address,u32 bytes) {
    if(!bytes || address>0xffffffffu-bytes)return 0;
    u32 end=address+bytes;
    for(u32 i=0;i<memory_inventory.count && address<end;i++) {
        InventoryRange *r=&memory_inventory.ranges[i];
        if(payload_selected(r) && r->base<=address && address<r->base+payload_extent(r)) {
            u32 next=r->base+payload_extent(r);address=next<end?next:end;
        }
    }
    return address==end;
}
/* Compare any overlapping anchor bytes in a copied chunk, including chunks
 * that split an anchor. Before/after reads alone could miss an ABA change. */
static int payload_overlap(u32 address,u32 size,u32 anchor,const void *data,u32 bytes) {
    u32 left=address>anchor?address:anchor,right=address+size<anchor+bytes?address+size:anchor+bytes;
    const u8 *expected=(const u8 *)data;
    for(u32 at=left;at<right;at++)if(payload_buffer[at-address]!=expected[at-anchor]){payload_status=7;return 0;}
    return 1;
}
static int payload_chunk_anchors(u32 address,u32 size) {
    RestoreContext *c=&restore_context;
    return payload_overlap(address,size,c->unit,c->unit_data,sizeof c->unit_data) &&
        payload_overlap(address,size,c->origin,c->table,c->table_bytes) &&
        payload_overlap(address,size,0xcab3ac,&c->origin,4) &&
        payload_overlap(address,size,0xcae5fc,&c->center,4) &&
        payload_overlap(address,size,0xc06188,&payload_header.world,4);
}
static int payload_anchor_read(u32 address,const void *data,u32 size) {
    const u8 *expected=(const u8 *)data;
    for(u32 done=0;done<size;) {
        u32 n=size-done;if(n>PAYLOAD_CHUNK)n=PAYLOAD_CHUNK;
        if(!payload_read(address+done,payload_buffer,n))return 0;
        for(u32 j=0;j<n;j++)if(payload_buffer[j]!=expected[done+j]){payload_status=7;return 0;}
        done+=n;
    }
    return 1;
}
static int payload_anchors(void) {
    RestoreContext *c=&restore_context;
    return payload_anchor_read(c->unit,c->unit_data,sizeof c->unit_data) &&
        payload_anchor_read(c->origin,c->table,c->table_bytes) &&
        payload_anchor_read(0xcab3ac,&c->origin,4) &&
        payload_anchor_read(0xcae5fc,&c->center,4) &&
        payload_anchor_read(0xc06188,&payload_header.world,4);
}
static int payload_mapping(const InventoryRange *expected) {
    InventoryRange actual={0};
    if(!payload_time())return 0;
    if(VirtualQueryEx(g_proc,(const void *)expected->base,&actual,sizeof actual)!=sizeof actual) {
        payload_status=6;return 0;
    }
    const u32 *a=(const u32 *)&actual,*b=(const u32 *)expected;
    for(u32 i=0;i<7;i++)if(a[i]!=b[i]){payload_status=6;return 0;}
    return payload_time();
}
static void capture_memory_payload(void) {
    MemoryInventory *m=&memory_inventory;RestoreContext *c=&restore_context;
    PayloadHeader *h=&payload_header;HANDLE file=INVALID_HANDLE;
    payload_started=GetTickCount();payload_status=payload_copied=payload_index=0;
    memset(h,0,sizeof *h);h->magic=0x31504d52;h->version=1;h->frame=m->frame;h->unit=m->unit;
    h->cap=PAYLOAD_CAP;h->max_ms=PAYLOAD_MS;h->chunk=PAYLOAD_CHUNK;
    h->inventory_bytes=280+m->count*28;h->stack_low=c->tib[2];h->stack_high=c->tib[1];
    if(m->status || m->count>INVENTORY_MAX || m->cursor!=m->end || !m->count ||
       m->frame!=c->frame || m->unit!=c->unit || h->stack_low>=h->stack_high) {payload_status=1;goto finish;}
    for(u32 i=0;i<m->count;i++) {
        InventoryRange *r=&m->ranges[i];
        if(r->base<=m->observer && m->observer<r->base+r->size && r->state==0x1000 &&
           r->type==0x1000000 && r->allocation!=m->main_base)h->observer=r->allocation;
    }
    if(!h->observer){payload_status=2;goto finish;}
    for(u32 i=0;i<m->count;i++)if(payload_selected(&m->ranges[i])) {
        payload_index=i;u32 size=payload_extent(&m->ranges[i]);
        if(size>PAYLOAD_CAP-h->bytes){payload_status=3;goto finish;}
        h->bytes+=size;h->count++;
    }
    if(!h->count || !payload_covered(c->unit,sizeof c->unit_data) ||
       !payload_covered(c->origin,c->table_bytes) || !payload_covered(0xcab3ac,4) ||
       !payload_covered(0xcae5fc,4) || !payload_covered(0xc06188,4)){payload_status=2;goto finish;}
    if(!payload_read(0xc06188,&h->world,4) || !payload_anchors())goto finish;
    char path[320];path_join(path,"memory-payload.bin");
    file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    if(file==INVALID_HANDLE){payload_status=9;goto finish;}
    if(!payload_write(file,h,sizeof *h) || !payload_write(file,m,h->inventory_bytes))goto finish;
    for(u32 i=0;i<m->count;i++) {
        InventoryRange *r=&m->ranges[i];if(!payload_selected(r))continue;
        payload_index=i;u32 size=payload_extent(r),record[]={i,r->base,size};
        if(!payload_mapping(r) || !payload_write(file,record,sizeof record))goto finish;
        for(u32 done=0;done<size;) {
            u32 n=size-done;if(n>PAYLOAD_CHUNK)n=PAYLOAD_CHUNK;
            if(!payload_read(r->base+done,payload_buffer,n) || !payload_chunk_anchors(r->base+done,n) ||
               !payload_write(file,payload_buffer,n))goto finish;
            done+=n;payload_copied+=n;
        }
    }
    for(u32 i=0;i<m->count;i++)if(payload_selected(&m->ranges[i])) {
        payload_index=i;if(!payload_mapping(&m->ranges[i]))goto finish;
    }
    if(!payload_anchors())goto finish;
    PayloadFooter footer={0x31444e45,1,h->count,h->bytes,GetTickCount()-payload_started,h->world,
                          (u32)sizeof c->unit_data+c->table_bytes+12,0};
    if(!payload_write(file,&footer,sizeof footer))goto finish;
finish:
    if(file!=INVALID_HANDLE && !CloseHandle(file) && !payload_status)payload_status=10;
    if(!payload_status)payload_time();
    u32 elapsed=GetTickCount()-payload_started;
    if(!payload_status && elapsed>=PAYLOAD_MS)payload_status=8;
    if(payload_status)emit(K_INFO,168,payload_status,payload_index,payload_copied,elapsed,0);
    else emit(K_INFO,167,0,h->unit,h->bytes,h->count,elapsed);
}
