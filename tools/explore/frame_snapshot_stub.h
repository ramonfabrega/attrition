/* The existing tested extended-state adapter, followed by displaced prologue. */
static u32 build_frame_snapshot_stub(u8 *s,u32 address,u32 target,u32 callback,
                                     const u8 *displaced,u32 length){
    u32 n=build_register_image_stub(s,callback);
    memcpy(s+n,displaced,length);n+=length;
    s[n++]=0xe9;*(u32 *)(s+n)=target+length-(address+n+4);
    return n+4;
}
