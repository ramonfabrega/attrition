/* Opt-in end_frame entry collector. No restore-probe identity is fabricated.
 * Other threads remain runnable. Anchors prove stability only at checked bytes. */
#include RON_STATE_PLAN
IMPORT(i32, ReadProcessMemory, (HANDLE,const void *,void *,u32,u32 *));
IMPORT(u32, VirtualQueryEx, (HANDLE,const void *,void *,u32));
IMPORT(void, GetSystemInfo, (void *));
IMPORT(u32, GetTickCount, (void));
#define FS_CAP (1024u*1024u*1024u)
#define FS_CHUNK (1024u*1024u)
#define FS_MAX 8192u
#define FS_MS 5000u
#define FS_ANCHOR_CAP 524288u
/* header: magic/version/phase/frame/trace_frame/thread/game/logger/main/observer,
 * stack low/high/begin/end/page/ranges/spans/bytes/anchors/anchor bytes,
 * inventory ms/copy ms/cap/chunk/max ranges/max inventory ms/max copy ms,
 * game slot/frame offset/log start/log end/reserved. */
static u32 fs_header[32],fs_ranges[FS_MAX][7],fs_roots[][2]=SNAP_ROOTS;
static u32 fs_anchor[16][3],fs_anchor_count,fs_anchor_bytes;
static u8 fs_expected[FS_ANCHOR_CAP],fs_buffer[FS_CHUNK];
static u32 fs_started,fs_status,fs_claimed,fs_copied,fs_pending;
static int fs_time(void){if(GetTickCount()-fs_started>=FS_MS){fs_status=8;return 0;}return 1;}
static int fs_read(u32 address,void *out,u32 n){
    u32 got=0;
    if(!n || address<0x10000 || address>0xffffffffu-n){fs_status=1;return 0;}
    if(!fs_time())return 0;
    if(!ReadProcessMemory(g_proc,(const void *)address,out,n,&got)||got!=n){fs_status=4;return 0;}
    return fs_time();
}
static int fs_write(HANDLE file,const void *p,u32 n){
    u32 got=0;if(!fs_time())return 0;
    if(!WriteFile(file,p,n,&got,0)||got!=n){fs_status=5;return 0;}return fs_time();
}
static u32 fs_extent(const u32 *r){return r[3]>fs_header[13]-r[0]?fs_header[13]-r[0]:r[3];}
static int fs_selected(const u32 *r){
    u32 p=r[5]&255;
    return r[4]==0x1000 && !(r[5]&0x100) && (p==2||p==4||p==8) &&
        r[1]!=fs_header[9] && !(r[0]<fs_header[11] && r[0]+fs_extent(r)>fs_header[10]) &&
        (r[6]==0x20000 || (r[6]==0x1000000 && r[1]==g_base));
}
static int fs_add_anchor(u32 address,u32 n){
    if(fs_anchor_count==16 || n>FS_ANCHOR_CAP-fs_anchor_bytes){fs_status=2;return 0;}
    u32 *a=fs_anchor[fs_anchor_count++];a[0]=address;a[1]=n;a[2]=fs_anchor_bytes;
    if(!fs_read(address,fs_expected+fs_anchor_bytes,n))return 0;
    fs_anchor_bytes+=n;return 1;
}
static int fs_check_anchors(void){
    for(u32 i=0;i<fs_anchor_count;i++){
        u32 *a=fs_anchor[i];if(!fs_read(a[0],fs_buffer,a[1]))return 0;
        for(u32 j=0;j<a[1];j++)if(fs_buffer[j]!=fs_expected[a[2]+j]){fs_status=7;return 0;}
    }return 1;
}
static int fs_overlap(u32 address,u32 n){
    for(u32 i=0;i<fs_anchor_count;i++){
        u32 *a=fs_anchor[i],lo=address>a[0]?address:a[0],hi=address+n<a[0]+a[1]?address+n:a[0]+a[1];
        for(u32 j=lo;j<hi;j++)if(fs_buffer[j-address]!=fs_expected[a[2]+j-a[0]]){fs_status=7;return 0;}
    }return 1;
}
static int fs_mapping(const u32 *r){
    u32 now[7];if(!fs_time())return 0;
    if(VirtualQueryEx(g_proc,(const void *)r[0],now,28)!=28){fs_status=6;return 0;}
    for(u32 i=0;i<7;i++)if(now[i]!=r[i]){fs_status=6;return 0;}return fs_time();
}
static int fs_covered(u32 address,u32 n){
    if(!n || address>0xffffffffu-n)return 0;u32 end=address+n;
    for(u32 i=0;i<fs_header[15] && address<end;i++){
        u32 *r=fs_ranges[i];if(fs_selected(r) && r[0]<=address && address<r[0]+fs_extent(r)){
            u32 next=r[0]+fs_extent(r);address=next<end?next:end;
        }
    }return address==end;
}
static void fs_capture(u32 game,u32 frame,u32 logger,u32 observer,u32 low,u32 high){
    HANDLE file=INVALID_HANDLE;u32 system[9]={0},info[7],start=GetTickCount();
    fs_status=fs_copied=fs_anchor_count=fs_anchor_bytes=0;fs_started=start;
    memset(fs_header,0,sizeof fs_header);
    u32 *h=fs_header;h[0]=0x31534652;h[1]=1;h[2]=1;h[3]=frame;h[4]=(u32)g_frame;
    h[5]=GetCurrentThreadId();h[6]=game;h[7]=logger;h[8]=g_base;h[10]=low;h[11]=high;
    h[22]=FS_CAP;h[23]=FS_CHUNK;h[24]=FS_MAX;h[25]=2000;h[26]=FS_MS;
    h[27]=SNAP_GAME_SLOT;h[28]=SNAP_FRAME_OFFSET;
    if(low>=high || logger!=SNAP_LOGGER || g_base!=0x400000){fs_status=1;goto finish;}
    if(!fs_read(SNAP_LOGGER+SNAP_START_OFFSET,h+29,4)||!fs_read(SNAP_LOGGER+SNAP_END_OFFSET,h+30,4))goto finish;
    if((i32)h[29]>=0 && (frame<h[29] || frame>=h[30])){fs_status=10;goto finish;}
    if(!fs_add_anchor(SNAP_GAME_SLOT,4)||!fs_add_anchor(game+SNAP_FRAME_OFFSET,4))goto finish;
    if(*(u32 *)fs_expected!=game || *(u32 *)(fs_expected+4)!=frame){fs_status=7;goto finish;}
    for(u32 i=0;i<sizeof fs_roots/sizeof fs_roots[0];i++)if(!fs_add_anchor(fs_roots[i][0],fs_roots[i][1]))goto finish;
    GetSystemInfo(system);h[12]=system[2];h[13]=system[3]+1;h[14]=system[1];
    if((system[0]&65535)!=0 || h[12]<65536 || h[12]>=h[13] || !h[14] || h[14]>65536 ||
       (h[14]&(h[14]-1)) || h[12]%h[14] || h[13]%h[14]){fs_status=1;goto finish;}
    for(u32 cursor=h[12];cursor<h[13];){
        if(GetTickCount()-start>=2000 || h[15]==FS_MAX){fs_status=3;goto finish;}
        u32 *r=fs_ranges[h[15]];
        if(VirtualQueryEx(g_proc,(const void *)cursor,r,28)!=28){fs_status=6;goto finish;}
        if(r[0]!=cursor || !r[3] || r[3]>0xffffffffu-r[0] || r[0]%h[14] || r[3]%h[14] ||
           (r[4]!=0x1000 && r[4]!=0x2000 && r[4]!=0x10000)){fs_status=6;goto finish;}
        if(r[0]<=observer && observer<r[0]+r[3] && r[4]==0x1000 && r[6]==0x1000000 && r[1]!=g_base)h[9]=r[1];
        h[15]++;u32 next=r[0]+r[3];cursor=next<h[13]?next:h[13];
    }
    h[20]=GetTickCount()-start;if(h[20]>=2000){fs_status=3;goto finish;}
    fs_started=GetTickCount();
    if(!h[9]){fs_status=2;goto finish;}
    for(u32 i=0;i<h[15];i++)if(fs_selected(fs_ranges[i])){
        u32 n=fs_extent(fs_ranges[i]);if(n>FS_CAP-h[17]){fs_status=2;goto finish;}h[17]+=n;h[16]++;
    }
    if(!h[16]){fs_status=2;goto finish;}
    for(u32 i=0;i<fs_anchor_count;i++)if(!fs_covered(fs_anchor[i][0],fs_anchor[i][1])){fs_status=2;goto finish;}
    h[18]=fs_anchor_count;h[19]=fs_anchor_bytes;
    if(!fs_check_anchors())goto finish;
    char path[320];path_join(path,"frame-snapshot.bin");
    file=CreateFileA(path,GENERIC_WRITE,FILE_SHARE_READ,0,1,FILE_ATTRIBUTE_NORMAL,0);
    if(file==INVALID_HANDLE){fs_status=9;goto finish;}
    if(!fs_write(file,h,sizeof fs_header)||!fs_write(file,fs_ranges,h[15]*28)||
       !fs_write(file,fs_anchor,h[18]*12)||!fs_write(file,fs_expected,h[19]))goto finish;
    for(u32 i=0;i<h[15];i++)if(fs_selected(fs_ranges[i])){
        u32 *r=fs_ranges[i],n=fs_extent(r),record[]={i,r[0],n};
        if(!fs_mapping(r)||!fs_write(file,record,12))goto finish;
        for(u32 done=0;done<n;){u32 size=n-done;if(size>FS_CHUNK)size=FS_CHUNK;
            if(!fs_read(r[0]+done,fs_buffer,size)||!fs_overlap(r[0]+done,size)||!fs_write(file,fs_buffer,size))goto finish;
            done+=size;fs_copied+=size;
        }
    }
    for(u32 i=0;i<h[15];i++)if(fs_selected(fs_ranges[i]) && !fs_mapping(fs_ranges[i]))goto finish;
    if(!fs_check_anchors())goto finish;
    info[0]=0x45465352;info[1]=1;info[2]=frame;info[3]=h[16];info[4]=fs_copied;
    info[5]=GetTickCount()-fs_started;info[6]=h[18];
    if(!fs_write(file,info,sizeof info))goto finish;
finish:
    if(file!=INVALID_HANDLE && !CloseHandle(file) && !fs_status)fs_status=11;
    if(fs_status)emit(K_INFO,181,fs_status,frame,fs_copied,GetTickCount()-start,0);
    else {fs_pending=1;emit(K_INFO,180,frame,h[16],fs_copied,GetTickCount()-fs_started,h[18]);}
}
static void __cdecl frame_snapshot_enter(u32 *regs){
    (void)regs; /* Optimized end_frame consumes the global logger, not ECX. */
    if(fs_claimed || g_frame<0 || ((u32)g_frame!=RON_STATE_FRAME && (u32)g_frame+1!=RON_STATE_FRAME))return;
    u32 game=0,frame=0;fs_started=GetTickCount();fs_status=0;
    if(!fs_read(SNAP_GAME_SLOT,&game,4)||!fs_read(game+SNAP_FRAME_OFFSET,&frame,4)){
        fs_claimed=1;emit(K_INFO,181,fs_status,0,0,0,0);return;
    }
    if(frame!=RON_STATE_FRAME)return;
    fs_claimed=1;
    fs_capture(game,frame,SNAP_LOGGER,(u32)(void *)frame_snapshot_enter,rd_fs(8),rd_fs(4));
}

/* Hooked immediately after the normal do_frame call returns from end_frame.
 * A changed root is evidence against treating the entire logger as pure. */
static void __cdecl frame_snapshot_after(u32 *regs){
    (void)regs;if(!fs_pending)return;fs_pending=0;fs_started=GetTickCount();fs_status=0;
    u32 checked=0,changed=0,total=0;
    for(u32 i=0;i<fs_anchor_count;i++){
        u32 *a=fs_anchor[i],n=0;if(!fs_read(a[0],fs_buffer,a[1]))break;
        for(u32 j=0;j<a[1];j++)if(fs_buffer[j]!=fs_expected[a[2]+j])n++;
        checked++;if(n){changed++;total+=n;emit(K_INFO,184,fs_header[3],i,a[0],a[1],n);}
    }
    emit(K_INFO,182,fs_header[3],checked,changed,total,fs_status);
}
