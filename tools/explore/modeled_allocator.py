"""Opt-in malloc-success experiment; no claim to reproduce the native heap.

A caller supplies a free arena and an independently verified malloc target.
The authored RET replaces that one service only. Requested bytes become
addressable but remain uninitialized. Alignment gaps and unused capacity are
never addressable; zero-size requests and exhausted quotas refuse execution.
"""
import struct
from unicorn import UC_MEM_READ, x86_const as x
from bounded_call import Region
from replay_capsule import require
from thread_context_probe import ThreadCall


class AllocatorCall(ThreadCall):
    def __init__(self, regions, stop, *, malloc_target, arena_address,
                 arena_size, max_allocations=64, **kwargs):
        require(arena_size > 0 and arena_size <= 16*1024*1024, 'invalid arena size')
        require(arena_address % 16 == 0, 'arena must be 16-byte aligned')
        require(max_allocations > 0, 'invalid allocation quota')
        self.malloc_target = malloc_target
        self.arena_address, self.arena_size = arena_address, arena_size
        self.max_allocations = max_allocations
        self.allocations, self.cursor = [], 0
        super().__init__([*regions,
                          Region('_malloc_service', malloc_target, b'\xc3', executable=True),
                          Region('_malloc_arena', arena_address, bytes(arena_size),
                                 writable=True, scratch=True)], stop, **kwargs)

    def run(self, entry, registers, inputs=None):
        self.allocations, self.cursor = [], 0
        return super().run(entry, registers, inputs)

    def access(self, uc, kind, address, size, value, opaque):
        lo, hi = self.arena_address, self.arena_address+self.arena_size
        if address < hi and address+size > lo:
            require(any(a['address'] <= address and address+size <= a['address']+a['size']
                        for a in self.allocations),
                    f'outside modeled allocation {address:08x}+{size}')
        return super().access(uc, kind, address, size, value, opaque)

    def checked_word(self, address):
        # Host mem_read bypasses Unicorn access hooks, so explicitly run the
        # same byte/initialization checks used for guest loads.
        self.access(self.uc, UC_MEM_READ, address, 4, 0, None)
        return struct.unpack('<I', self.uc.mem_read(address, 4))[0]

    def code(self, uc, address, size, opaque):
        super().code(uc, address, size, opaque)
        if address != self.malloc_target:
            return
        esp = uc.reg_read(x.UC_X86_REG_ESP)
        ret, requested = self.checked_word(esp), self.checked_word(esp+4)
        target = self.region(ret, 1)
        require(ret == self.stop or (target is not None and target.executable),
                'malloc return target is not declared code')
        require(requested > 0, 'zero-size malloc is unmodeled')
        require(len(self.allocations) < self.max_allocations, 'allocation quota exhausted')
        offset = (self.cursor+15) & ~15
        require(offset+requested <= self.arena_size, 'allocation arena exhausted')
        result = self.arena_address+offset
        self.allocations.append(dict(address=result, size=requested, return_address=ret))
        self.cursor = offset+requested
        uc.reg_write(x.UC_X86_REG_EAX, result)
        # RET pops only the return address. The cdecl caller owns the argument.
        # Other registers and flags are preserved by this chosen model; native
        # caller-saved clobbers and native pointer identities are not modeled.
