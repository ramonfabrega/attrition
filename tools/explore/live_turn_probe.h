/* Field capture for a real GuyData::turn_speed call. Layout/load operands:
 * tools/explore/turn_oracle.py; actual arguments and return use CALL/RET site 8.
 * Normal squad members only; excluded crew calls carry INFO 113 instead.
 */
static void probe_turn(u32 self) {
    u8 *guy = (u8 *)self;
    i32 who = (signed char)guy[0xa1];
    i32 object = *(short *)(guy + 0x8c);
    if (who < 0 || who >= 8 || object < 0) {
        emit(K_INFO, 113, self, 1, 0, 0, 0);
        return;
    }
    u8 **slots = *(u8 ***)(g_base + 0x80aec0u + (u32)who * 0x1cu);
    u8 *unit = slots[object];
    u8 *type = *(u8 **)(unit + 0x18);
    i32 member = (signed char)guy[0xa2];
    i32 squad = *(i32 *)(type + 0x304);
    if (member < 0 || member >= squad) {
        emit(K_INFO, 113, self, 2, (u32)member, (u32)squad, 0);
        return;
    }
    u8 *constants = *(u8 **)(g_base + 0x8061e4u);
    emit(K_INFO, 110, self, *(u32 *)(type + 0x2c4), (u32)squad,
         *(u32 *)(guy + 0x80), *(u32 *)(guy + 0x84));
    emit(K_INFO, 111, self, *(u16 *)(guy + 0x9a), *(u32 *)(unit + 0x68),
         *(u32 *)(constants + 8), *(u32 *)(constants + 12));
}
