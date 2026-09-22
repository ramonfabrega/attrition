/* Explicit namespacing for the second packet only; never changes log paths. */
#ifndef RESTORE_PACKET_PATH_H
#define RESTORE_PACKET_PATH_H
#ifdef RON_RESTORE_SECOND
static u32 restore_packet_index;
static void restore_packet_path(char *out,const char *name) {
    if(!restore_packet_index){path_join(out,name);return;}
    char leaf[80];u32 i=0;
    const char *prefix="second-";
    while(*prefix)leaf[i++]=*prefix++;
    while(*name && i<sizeof leaf-1)leaf[i++]=*name++;
    leaf[i]=0;path_join(out,leaf);
}
#else
#define restore_packet_path path_join
#endif
#endif
