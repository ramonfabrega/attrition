#include "restore_packet_path.h"
/* One bounded structural snapshot, after the first natural suspension.
 * Authored TLV protocol; original-derived bytes stay beside the trace. */
#define GRAPH_BYTES (256u*1024u)
#define GRAPH_NODES 2048u
#define GRAPH_PAYLOADS 1024u
static u32 graph_storage[GRAPH_BYTES/4], graph_used, graph_records;
static u32 graph_nodes[GRAPH_NODES], graph_payloads[GRAPH_PAYLOADS];
static u32 graph_node_count, graph_payload_count;
static int graph_failed;
static u32 graph_failure_tag=151;
#ifdef RON_RESTORE_POSTGRAPH
IMPORT(u32, GetTickCount, (void));
static int graph_timed;
static u32 graph_started;
#endif
static int graph_fail(u32 reason, u32 value) {
    emit(K_INFO, graph_failure_tag, reason, value, graph_records, graph_used, 0);
    graph_failed = 1; return 0;
}
static int graph_read(u32 address, void *out, u32 size) {
#ifdef RON_RESTORE_POSTGRAPH
    if (graph_timed && GetTickCount()-graph_started>=2000u) return graph_fail(11,address);
#endif
    u32 copied = 0;
    if (address < 0x10000u || address > 0xffffffffu-size ||
        !ReadProcessMemory(g_proc, (void *)address, out, size, &copied) || copied != size)
        return graph_fail(1, address);
    return 1;
}
static u32 *graph_record(u32 kind, u32 owner, u32 address, u32 size) {
    if (graph_records>=4096u || !size || (size&3) || size > GRAPH_BYTES-16 || graph_used > GRAPH_BYTES-16-size) {
        graph_fail(2, size); return 0;
    }
    u32 *p = graph_storage+graph_used/4;
    p[0]=kind; p[1]=owner; p[2]=address; p[3]=size;
    if (!graph_read(address,p+4,size)) return 0;
    graph_used += 16+size; graph_records++; return p+4;
}
static int graph_payload(u32 address) {
    if (!address) return graph_fail(3, address);
    for (u32 i=0;i<graph_payload_count;i++) if (graph_payloads[i]==address) return 1;
    if (graph_payload_count==GRAPH_PAYLOADS) return graph_fail(4, address);
    graph_payloads[graph_payload_count++]=address; return 1;
}
static int graph_tree(u32 owner, u32 address) {
    u32 *h = graph_record(2,owner,address,(owner==0 || owner==4)?28:24);
    if (!h) return 0;
    if (h[2]>GRAPH_NODES || h[2]>GRAPH_NODES-graph_node_count) return graph_fail(5,h[2]);
    u32 next=graph_node_count, active=0;
    if (h[3]) {
        if (graph_node_count==GRAPH_NODES) return graph_fail(7,graph_node_count);
        for (u32 i=0;i<graph_node_count;i++)
            if (graph_nodes[i]==h[3]) return graph_fail(6,h[3]);
        graph_nodes[graph_node_count++]=h[3];
    }
    while (next<graph_node_count) {
        u32 address_node=graph_nodes[next++];
        u32 *node=graph_record(3,owner,address_node,(owner==0 || owner==4)?20:24);
        if (!node) return 0;
        for (u32 child=0;child<2;child++) if (node[child]) {
            for (u32 i=0;i<graph_node_count;i++)
                if (graph_nodes[i]==node[child]) return graph_fail(6,node[child]);
            if (graph_node_count==GRAPH_NODES) return graph_fail(7,graph_node_count);
            graph_nodes[graph_node_count++]=node[child];
        }
        int live=(owner==0 || owner==4 || !((u8 *)node)[0x15]);
        if (live) active++;
        if ((owner==0 || (owner==2 && live)) && !graph_payload(node[3])) return 0;
        if (owner==4 && !graph_record(5,owner,node[3],108)) return 0;
    }
    if (active != h[2]) return graph_fail(8,owner);
    return 1;
}
static int collect_search_graph(u32 unit) {
    graph_used=32; graph_records=graph_node_count=graph_payload_count=0; graph_failed=0;
    u32 *u=graph_record(1,0,unit+0x104,0x48);
    if (!u) return 0;
    for (u32 i=0;i<5;i++) if (!u[i] || !graph_tree(i,u[i])) return 0;
    /* PathNode parent closure; fixed capacity bounds even cyclic payload links.
     * The offline validator rejects cycles and ownership inconsistencies. */
    for (u32 i=0;i<graph_payload_count;i++) {
        u32 *p=graph_record(4,0,graph_payloads[i],36);
        if (!p || (p[8] && !graph_payload(p[8]))) return 0;
    }
    const u32 pools[]={0xc8d810,0xc8d950,0xc8d860,0xc8d880,0xc8d9a0,0xc8d9b0,0xc8da70};
    for (u32 i=0;i<7;i++) {
        u32 *p=graph_record(6,i,pools[i],16);
        if (!p) return 0;
        if (p[1]>4096 || p[2]>p[1] || (p[1] && !p[0])) { graph_fail(9,i); return 0; }
        if (p[1] && !graph_record(7,i,p[0],p[1]*4)) return 0;
    }
    graph_storage[0]=0x31475352; graph_storage[1]=1; graph_storage[2]=(u32)g_frame;
    graph_storage[3]=unit; graph_storage[4]=graph_records; graph_storage[5]=graph_used;
    graph_storage[6]=graph_node_count; graph_storage[7]=graph_payload_count;
    return 1;
}
static void capture_search_graph(u32 unit) {
    if (!collect_search_graph(unit)) return;
    char path[320]; restore_packet_path(path,"search-graph.bin");
    HANDLE file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    u32 written=0;
    i32 ok=file!=INVALID_HANDLE && WriteFile(file,graph_storage,graph_used,&written,0);
    if (file!=INVALID_HANDLE) CloseHandle(file);
    if (!ok || written!=graph_used) { graph_fail(10,written); return; }
    emit(K_INFO,150,0,unit,graph_node_count,graph_payload_count,graph_used);
    flush();
}
