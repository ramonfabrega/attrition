/* Observer adapter for 32-bit x86. Callback is cdecl and must preserve control
 * flags (including DF) and nonvolatile GPRs. It may clobber x87/SSE state.
 * Image: EDI, ESI, EBP, original ESP minus 4, EBX, EDX, ECX, EAX,
 * packed LAHF/SETO flags. The last word is NOT a complete EFLAGS snapshot.
 * No PUSHAD/POPAD/PUSHFD/POPFD. Requires FXSAVE support, as does this target's
 * SSE environment. All stack storage is private to this invocation.
 */
static u32 build_register_image_stub(u8 *s, u32 callback) {
    static const u8 head[] = {
        0x50,0x9f,0x0f,0x90,0xc0, /* save EAX; LAHF; SETO AL */
        0x87,0x04,0x24,0x50,      /* exchange EAX with flags word; save EAX */
        0x51,0x52,0x53,           /* ECX, EDX, EBX */
        0x8d,0x44,0x24,0x10,0x50, /* original ESP-4 */
        0x55,0x56,0x57,           /* EBP, ESI, EDI */
        0x89,0xe3,                /* EBX = image */
        0x81,0xec,0x10,0x02,0,0, /* reserve 528 bytes */
        0x83,0xe4,0xf0,           /* align save area to 16 */
        0x0f,0xae,0x04,0x24,      /* FXSAVE [ESP] */
        0x89,0xe6,                /* ESI = save area */
        0x83,0xec,0x0c,0x53,      /* argument, aligned pre-CALL stack */
        0xb8                     /* MOV EAX, callback */
    };
    static const u8 tail[] = {
        0xff,0xd0,0x83,0xc4,0x10, /* CALL EAX; remove argument/padding */
        0x0f,0xae,0x0e,           /* FXRSTOR [ESI] */
        0x89,0xdc,                /* ESP = image */
        0x5f,0x5e,0x5d,0x83,0xc4,0x04, /* EDI, ESI, EBP; skip saved ESP */
        0x5b,0x5a,0x59,           /* EBX, EDX, ECX */
        0x8b,0x44,0x24,0x04,      /* packed flags */
        0x04,0x7f,0x9e,0x58,      /* restore OF, arithmetic flags, EAX */
        0x8d,0x64,0x24,0x04       /* discard flags without changing them */
    };
    u32 n=sizeof head;
    memcpy(s,head,n);memcpy(s+n,&callback,4);n+=4;
    memcpy(s+n,tail,sizeof tail);return n+sizeof tail;
}

static void copy_register_image(u32 *out,const u32 *in) {
    memcpy(out,in,32);out[3]+=4;
    out[8]=((in[8]>>8)&0xd5u)|((in[8]&1u)<<11);
}
