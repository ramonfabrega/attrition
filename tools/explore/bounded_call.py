"""Byte-bounded x86 calls with explicit scratch and reset between inputs.

Regions contain authored/captured bytes. Scratch may be written, but a read
must follow a write in the same call. Page padding never authorizes access.
"""
from dataclasses import dataclass
from unicorn import (Uc, UcError, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE,
                     UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE, UC_MEM_WRITE,
                     UC_PROT_READ, UC_PROT_WRITE, UC_PROT_EXEC)
from unicorn import x86_const as x
from replay_capsule import require, REG_IDS


@dataclass(frozen=True)
class Region:
    name: str
    address: int
    data: bytes
    writable: bool = False
    executable: bool = False
    scratch: bool = False


class BoundedCall:
    def __init__(self, regions, stop, budget=256):
        self.regions = tuple(regions)
        self.stop, self.budget = stop, budget
        require(budget > 0, 'invalid instruction budget')
        require(len({r.name for r in regions}) == len(regions), 'duplicate region name')
        pages = {}
        for i, r in enumerate(regions):
            require(r.data and 0 < r.address < r.address+len(r.data) <= 2**32, 'invalid region')
            require(not (r.writable and r.executable), 'writable code')
            require(not r.scratch or r.writable, 'scratch must be writable')
            require(not r.address <= stop < r.address+len(r.data), 'stop overlaps region')
            for other in regions[:i]:
                require(r.address+len(r.data) <= other.address or
                        other.address+len(other.data) <= r.address, 'overlapping regions')
            flags = UC_PROT_READ | (UC_PROT_WRITE if r.writable else 0) | (UC_PROT_EXEC if r.executable else 0)
            for page in range(r.address & ~4095, (r.address+len(r.data)+4095) & ~4095, 4096):
                pages[page] = pages.get(page, 0) | flags
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        for page, flags in pages.items():
            self.uc.mem_map(page, 4096, flags)
        self.mapped_bytes = len(pages)*4096
        self.initial_context = self.uc.context_save()
        self.uc.hook_add(UC_HOOK_CODE, self.code)
        self.uc.hook_add(UC_HOOK_MEM_READ | UC_HOOK_MEM_WRITE, self.access)

    def region(self, address, size):
        return next((r for r in self.regions if r.address <= address and
                     address+size <= r.address+len(r.data)), None)

    def access(self, uc, kind, address, size, value, _):
        r = self.region(address, size)
        require(r is not None, f'undeclared access {address:08x}+{size}')
        if kind == UC_MEM_WRITE:
            require(r.writable, f'read-only write: {r.name}')
            self.initialized.update(range(address, address+size))
            self.writes.append((address, size, value & ((1 << (8*size))-1)))
        else:
            require(not r.scratch or all(a in self.initialized for a in range(address, address+size)),
                    f'uninitialized scratch read {address:08x}+{size}')

    def code(self, uc, address, size, _):
        r = self.region(address, size)
        require(r is not None and r.executable, f'undeclared instruction {address:08x}+{size}')
        self.instructions += 1

    def run(self, entry, registers, inputs=None):
        executable = self.region(entry, 1)
        require(executable is not None and executable.executable, 'entry is not declared code')
        inputs = {} if inputs is None else inputs
        require(set(inputs) <= {r.name for r in self.regions if not r.executable and not r.scratch},
                'unknown/code/scratch input')
        require(len(registers) == len(REG_IDS), 'incomplete registers')
        self.uc.context_restore(self.initial_context)
        for r in self.regions:
            data = inputs.get(r.name, r.data)
            require(len(data) == len(r.data), f'input size: {r.name}')
            # Code is immutable; only data needs restoration on subsequent calls.
            if not r.executable or not getattr(self, 'loaded', False):
                self.uc.mem_write(r.address, data)
        self.loaded = True
        for reg, value in zip(REG_IDS, registers):
            self.uc.reg_write(reg, value)
        self.initialized, self.writes, self.instructions = set(), [], 0
        try:
            self.uc.emu_start(entry, self.stop, count=self.budget)
        except UcError as error:
            raise ValueError(f'bounded execution failed: {error}') from error
        require(self.uc.reg_read(x.UC_X86_REG_EIP) == self.stop, 'instruction budget exhausted')
        return (tuple(self.uc.reg_read(r) for r in REG_IDS),
                tuple((r.name, bytes(self.uc.mem_read(r.address, len(r.data))))
                      for r in self.regions if not r.executable), tuple(self.writes))
