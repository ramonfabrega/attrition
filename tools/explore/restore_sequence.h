/* Bounded two-packet selection; no memory access or intervention in this model. */
#ifndef RESTORE_SEQUENCE_H
#define RESTORE_SEQUENCE_H
/* 0 waiting first, 1 first active, 2 waiting next matching unit,
 * 3 second active, 4 finished, 5 refused. */
typedef struct {u32 stage,owner,id,unit,first_frame;} RestoreSequence;
static int restore_sequence_select(RestoreSequence *s,u32 owner,u32 id,u32 unit,u32 frame) {
    if(s->stage>=4)return 0;
    if(s->stage==2 && (owner!=s->owner || id!=s->id))return 0;
    if((s->stage!=0 && s->stage!=2) || owner>=8 || id>=512 ||
       unit<0x10000u || unit>0xffffffffu-344u || unit%4u) {s->stage=5;return -1;}
    if(s->stage==0) {
        s->owner=owner;s->id=id;s->unit=unit;s->first_frame=frame;s->stage=1;
    } else {
        if(unit!=s->unit || frame<s->first_frame){s->stage=5;return -1;}
        s->stage=3;
    }
    return 1;
}
static int restore_sequence_return(RestoreSequence *s,u32 unit,u32 frame,u32 result,int suspended) {
    if((s->stage!=1 && s->stage!=3) || unit!=s->unit || frame<s->first_frame ||
       (s->stage==1 && (result!=0xffffffffu || !suspended))) {s->stage=5;return 0;}
    s->stage=s->stage==1?2:4;return 1;
}
#endif
