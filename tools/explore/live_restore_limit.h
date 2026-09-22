/* One opt-in continuation-budget experiment. Only the fixed limit word is
 * written, after a successful pre-payload and restored before post observation.
 * No game pointer, route, allocator or return value is substituted. */
static struct {
    u32 version,address,before,after,saving,at_return,restored,flags;
} restore_limit;
_Static_assert(sizeof restore_limit==32,"limit provenance layout");
static int restore_limit_active;
#ifndef RESTORE_LIMIT_WORD
#define RESTORE_LIMIT_WORD (*(volatile u32 *)0xe85ec0u)
#endif
static int restore_limit_apply(void) {
    u32 modes[2];
    if(restore_limit_active || payload_status || !payload_header.count ||
       payload_copied!=payload_header.bytes || payload_header.frame!=restore_probe.frame ||
       payload_header.unit!=restore_probe.unit || (u32)g_frame!=restore_probe.frame ||
       restore_probe.after_modes[0]!=300 || restore_probe.after_modes[1]!=1 ||
       !restore_read(0xe85ec0,modes,8) || modes[0]!=300 || modes[1]!=1) goto failed;
    restore_limit.version=1;restore_limit.address=0xe85ec0;
    restore_limit.before=300;restore_limit.after=95;restore_limit.saving=1;
    restore_limit.flags=1;restore_limit_active=1;
    RESTORE_LIMIT_WORD=95;
    if(RESTORE_LIMIT_WORD!=95) {
        RESTORE_LIMIT_WORD=300;restore_limit_active=0;goto failed;
    }
    emit(K_INFO,183,restore_probe.unit,0xe85ec0,300,95,1);flush();return 1;
failed:
    emit(K_INFO,185,1,restore_probe.unit,0,0,0);flush();return 0;
}
static int restore_limit_finish(void) {
    if(!restore_limit_active) return 0;
    restore_limit_active=0;
    restore_limit.at_return=RESTORE_LIMIT_WORD;
    /* Restore even if the game or observer has changed unexpectedly. */
    RESTORE_LIMIT_WORD=restore_limit.before;
    restore_limit.restored=RESTORE_LIMIT_WORD;
    u32 modes[2];
    int ok=restore_read(0xe85ec0,modes,8) && modes[0]==300 && modes[1]==1 &&
           restore_limit.at_return==95 && restore_limit.restored==300;
    restore_limit.flags=ok?3:1;
    emit(K_INFO,184,restore_probe.unit,0xe85ec0,restore_limit.at_return,restore_limit.restored,1);
    if(!ok)emit(K_INFO,185,2,restore_probe.unit,0,0,0);
    flush();return ok;
}
