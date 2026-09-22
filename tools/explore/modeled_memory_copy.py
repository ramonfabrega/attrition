"""Opt-in, nonoverlapping memcpy contract implemented by guarded guest loads/stores."""
from unicorn import x86_const as x
from bounded_call import Region
from modeled_memory_fill import MemoryFillCall
from replay_capsule import require

# push esi; push edi; mov edi,[esp+12]; mov esi,[esp+16]; mov ecx,[esp+20];
# rep movsb; mov eax,[esp+12]; pop edi; pop esi; ret
COPY_CODE = bytes.fromhex('56 57 8b7c240c 8b742410 8b4c2414 f3a4 8b44240c 5f 5e c3')


class MemoryCopyCall(MemoryFillCall):
    def __init__(self, regions, stop, *, memcpy_target, max_copy_bytes=1024*1024,
                 max_copies=1024, **kwargs):
        require(0 < max_copy_bytes <= 16*1024*1024, 'invalid copy byte cap')
        require(max_copies > 0, 'invalid copy call cap')
        self.memcpy_target = memcpy_target
        self.max_copy_bytes, self.max_copies = max_copy_bytes, max_copies
        self.copies, self.copy_active = [], False
        super().__init__([*regions, Region('_memcpy_service', memcpy_target,
                                         COPY_CODE, executable=True)], stop, **kwargs)

    def run(self, entry, registers, inputs=None):
        self.copies, self.copy_active = [], False
        return super().run(entry, registers, inputs)

    def code(self, uc, address, size, opaque):
        super().code(uc, address, size, opaque)
        if not self.memcpy_target <= address < self.memcpy_target+len(COPY_CODE):
            return
        if address == self.memcpy_target:
            require(not self.copy_active, 'nested modeled copy')
            esp = uc.reg_read(x.UC_X86_REG_ESP)
            ret, destination, source, count = (self.checked_word(esp+4*i) for i in range(4))
            target = self.region(ret, 1)
            require(ret == self.stop or (target is not None and target.executable),
                    'memcpy return target is not declared code')
            require(not uc.reg_read(x.UC_X86_REG_EFLAGS) & 0x400,
                    'memcpy requires clear direction flag')
            require(count <= self.max_copy_bytes, 'copy byte cap exceeded')
            require(len(self.copies) < self.max_copies, 'copy call cap exceeded')
            require(source+count <= 2**32 and destination+count <= 2**32, 'copy address overflow')
            require(not count or destination+count <= source or source+count <= destination,
                    'overlapping memcpy is unmodeled')
            self.copies.append(dict(destination=destination, source=source, count=count,
                                    return_address=ret))
            self.copy_active = True
        require(self.copy_active, 'entry inside modeled copy')
        if address == self.memcpy_target+len(COPY_CODE)-1:
            self.copy_active = False
