"""Non-reusing free model for explicit borrowed objects and modeled allocations.

Borrowed extents are caller-supplied logical object bounds, not decoded CRT heap
chunks. They must be exact writable input regions. Retiring an object forbids
all later reads/writes through aliases; unknown/interior/double frees refuse.
"""
from unicorn import x86_const as x
from bounded_call import Region
from modeled_memory_copy import MemoryCopyCall
from replay_capsule import require


class LifetimeCall(MemoryCopyCall):
    def __init__(self, regions, stop, *, free_target, borrowed=(), max_frees=1024, **kwargs):
        require(max_frees > 0, 'invalid free call cap')
        self.borrowed = tuple(borrowed)
        for i, (address, size) in enumerate(self.borrowed):
            require(any(r.address == address and len(r.data) == size and r.writable and
                        not r.executable and not r.scratch for r in regions),
                    'borrowed object needs an exact writable input region')
            require(all(address+size <= a or a+n <= address for a, n in self.borrowed[:i]),
                    'overlapping borrowed objects')
        self.free_target, self.max_frees = free_target, max_frees
        self.frees, self.retired = [], []
        super().__init__([*regions, Region('_free_service', free_target, b'\xc3', executable=True)],
                         stop, **kwargs)

    def run(self, entry, registers, inputs=None):
        self.frees, self.retired = [], []
        return super().run(entry, registers, inputs)

    def access(self, uc, kind, address, size, value, opaque):
        require(not any(address < base+count and address+size > base for base, count in self.retired),
                f'retired object access {address:08x}+{size}')
        return super().access(uc, kind, address, size, value, opaque)

    def code(self, uc, address, size, opaque):
        super().code(uc, address, size, opaque)
        if address != self.free_target:
            return
        esp = uc.reg_read(x.UC_X86_REG_ESP)
        ret, pointer = self.checked_word(esp), self.checked_word(esp+4)
        target = self.region(ret, 1)
        require(ret == self.stop or (target is not None and target.executable),
                'free return target is not declared code')
        require(len(self.frees) < self.max_frees, 'free call cap exceeded')
        if pointer == 0:
            self.frees.append(dict(pointer=0, size=0, source='null', return_address=ret))
            return
        require(not any(pointer == a for a, _ in self.retired), 'double free')
        owned = next((a['size'] for a in self.allocations if a['address'] == pointer), None)
        borrowed = next((n for a, n in self.borrowed if a == pointer), None)
        extent = owned if owned is not None else borrowed
        require(extent is not None, f'unknown free pointer {pointer:08x}')
        self.retired.append((pointer, extent))
        self.frees.append(dict(pointer=pointer, size=extent,
                               source='modeled' if owned is not None else 'borrowed', return_address=ret))
        # No reuse and no writes to physical heap metadata. EAX/flags and
        # caller-saved registers are preserved by this chosen void-return model.
