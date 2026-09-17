/* RNG/frame entry stub. Save volatile registers individually and arithmetic
 * flags through LAHF/SETO/SAHF. Avoid the cross-thread WoW64 failure reproduced
 * by wow64bop.c's POPAD/POPFD cases; see ORACLE, "Coverage is back".
 *
 * The cdecl observer preserves EBX/ESI/EDI/EBP and the direction flag. The
 * original stack and all integer registers are restored before displacement.
 * Explicit addresses make the production emitter testable on a native host.
 */
static u32 build_hook_stub(u8 *s, u32 address, u32 target, u32 kind, u32 callback,
                           const u8 *displaced, u32 len) {
    static const u8 head[] = {
        0x50, 0x9f, 0x0f, 0x90, 0xc0, /* push eax; lahf; seto al */
        0x51, 0x52, 0x50,             /* ecx, edx, packed flags */
        0x8b,0x44,0x24,0x14,0x50,    /* original arg0 */
        0x8b,0x44,0x24,0x14,0x50,    /* original return address */
        0x55,0x51,0x68               /* ebp, ecx, kind */
    };
    u32 n=sizeof head;
    memcpy(s,head,n);
    *(u32 *)(s+n)=kind; n+=4;
    s[n++]=0xb8; *(u32 *)(s+n)=callback; n+=4;
    static const u8 tail[] = {
        0xff,0xd0,0x83,0xc4,0x14,    /* call observer; remove five args */
        0x58,0x5a,0x59,              /* packed flags, edx, ecx */
        0x04,0x7f,0x9e,0x58         /* restore OF, other arithmetic flags, eax */
    };
    memcpy(s+n,tail,sizeof tail); n+=sizeof tail;
    memcpy(s+n,displaced,len); n+=len;
    s[n++]=0xe9;
    *(u32 *)(s+n)=target+len-(address+n+4); n+=4;
    return n;
}
