"""Explicit cdecl memset model over the allocator runner's byte guards.

This authored x86 implementation is not DLL code. Every destination store and
stack load executes in Unicorn and passes normal access/initialization hooks.
Only the byte-fill contract is modeled; native timing and clobbers are not.
"""
from unicorn import x86_const as x
from bounded_call import Region
from modeled_allocator import AllocatorCall
from replay_capsule import require

# push edi; mov edi,[esp+8]; mov eax,[esp+12]; mov ecx,[esp+16];
# rep stosb; mov eax,[esp+8]; pop edi; ret
FILL_CODE = bytes.fromhex('57 8b7c2408 8b44240c 8b4c2410 f3aa 8b442408 5f c3')


class MemoryFillCall(AllocatorCall):
    def __init__(self, regions, stop, *, memset_target, max_fill_bytes=1024*1024,
                 max_fills=1024, **kwargs):
        require(0 < max_fill_bytes <= 16*1024*1024, 'invalid fill byte cap')
        require(max_fills > 0, 'invalid fill call cap')
        self.memset_target = memset_target
        self.max_fill_bytes, self.max_fills = max_fill_bytes, max_fills
        self.fills, self.fill_active = [], False
        super().__init__([*regions, Region('_memset_service', memset_target,
                                         FILL_CODE, executable=True)], stop, **kwargs)

    def run(self, entry, registers, inputs=None):
        self.fills, self.fill_active = [], False
        return super().run(entry, registers, inputs)

    def code(self, uc, address, size, opaque):
        super().code(uc, address, size, opaque)
        if not self.memset_target <= address < self.memset_target+len(FILL_CODE):
            return
        if address == self.memset_target:
            require(not self.fill_active, 'nested modeled fill')
            esp = uc.reg_read(x.UC_X86_REG_ESP)
            ret, destination, value, count = (self.checked_word(esp+4*i) for i in range(4))
            target = self.region(ret, 1)
            require(ret == self.stop or (target is not None and target.executable),
                    'memset return target is not declared code')
            require(not uc.reg_read(x.UC_X86_REG_EFLAGS) & 0x400,
                    'memset requires clear direction flag')
            require(count <= self.max_fill_bytes, 'fill byte cap exceeded')
            require(len(self.fills) < self.max_fills, 'fill call cap exceeded')
            # Zero count makes no destination access. Nonzero operations may
            # partially write before a byte guard refuses; failed runs are
            # discarded, and reset clears both guest bytes and initialization.
            self.fills.append(dict(destination=destination, value=value & 255,
                                   count=count, return_address=ret))
            self.fill_active = True
        require(self.fill_active, 'entry inside modeled fill')
        if address == self.memset_target+len(FILL_CODE)-1:
            self.fill_active = False
