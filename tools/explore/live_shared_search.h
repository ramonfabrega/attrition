/* Read-only shared-search boundary witness, frames 224..225, at most 8 calls.
 * PDB PathFinderData (0x88), UnitData (0x158), typed tree headers and two
 * recycler Stack headers. No node walks, writes, allocations or extra hooks.
 * Requires RON_SEARCH_CENSUS for its ReadProcessMemory declaration. */
static struct {u32 count,active,failed,self,path,frame;} shared_search;
static i32 shared_read(u32 index,u32 address,void *out,u32 size) {
    u32 copied=0;
    i32 ok=address>=0x10000u && address<=0xffffffffu-size &&
        ReadProcessMemory(g_proc,(void *)address,out,size,&copied);
    if(!ok || copied!=size) {
        emit(K_INFO,204,shared_search.count,index,address,size,copied);
        shared_search.failed=1;return 0;
    }
    return 1;
}
static void shared_snapshot(u32 phase,u32 ret) {
    u32 data[9][86],addresses[9],sizes[9]={136,344,28,24,24,28,24,16,16};
    const u32 lengths[5]={28,24,24,28,24};
    u32 total=0;
    addresses[0]=shared_search.self+0x40u;
    if(!shared_read(0,addresses[0],data[0],sizes[0]))return;
    addresses[1]=data[0][5];
    if(addresses[1]+0xb8u!=shared_search.path || !shared_read(1,addresses[1],data[1],sizes[1])) {
        if(!shared_search.failed)emit(K_INFO,204,shared_search.count,1,addresses[1],344,0);
        shared_search.failed=1;return;
    }
    for(u32 i=0;i<5;i++) {
        addresses[i+2]=data[0][i];sizes[i+2]=addresses[i+2]?lengths[i]:0;
        if(sizes[i+2] && !shared_read(i+2,addresses[i+2],data[i+2],sizes[i+2]))return;
    }
    addresses[7]=0xc8d9b0u;addresses[8]=0xc8d820u;
    for(u32 i=7;i<9;i++)if(!shared_read(i,addresses[i],data[i],sizes[i]))return;
    emit(K_INFO,200,shared_search.count,phase,shared_search.self,shared_search.path,ret);
    for(u32 i=0;i<9;i++) {
        emit(K_INFO,201,shared_search.count,i,addresses[i],sizes[i],0);
        for(u32 j=0;j<sizes[i]/4;j+=2)
            emit(K_INFO,202,shared_search.count,i,j*4,data[i][j],j+1<sizes[i]/4?data[i][j+1]:0);
        total+=sizes[i];
    }
    emit(K_INFO,203,shared_search.count,phase,9,total,0);
}
static void shared_enter(u32 self,u32 path) {
    if(g_frame<224 || g_frame>225 || shared_search.failed)return;
    if(shared_search.active || self!=0xe85e40u) {
        emit(K_INFO,204,shared_search.count,99,self,path,0);shared_search.failed=1;return;
    }
    if(shared_search.count==8) {
        emit(K_INFO,205,8,0,0,0,0);shared_search.failed=1;return;
    }
    shared_search.count++;shared_search.active=1;
    shared_search.self=self;shared_search.path=path;shared_search.frame=(u32)g_frame;
    shared_snapshot(0,0xffffffffu);
}
static void shared_return(u32 ret) {
    if(shared_search.failed || !shared_search.active)return;
    if(shared_search.frame!=(u32)g_frame) {
        emit(K_INFO,204,shared_search.count,98,shared_search.frame,(u32)g_frame,0);
        shared_search.failed=1;return;
    }
    shared_snapshot(1,ret);shared_search.active=0;
}
