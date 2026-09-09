#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Exercise the actual compiled modal trampoline with authored x86 state.

No game files needed. Mutating its callback argument offset must fail.
"""
from pathlib import Path
import random
import re
import struct
import subprocess
import tempfile
from test_hook_stub import reject_unsafe
from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE
from unicorn.x86_const import (UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_ECX,
    UC_X86_REG_EDX, UC_X86_REG_ESI, UC_X86_REG_EDI, UC_X86_REG_EBP,
    UC_X86_REG_ESP, UC_X86_REG_EIP, UC_X86_REG_EFLAGS)

ROOT=Path(__file__).resolve().parents[2]
LLVM=Path('/opt/homebrew/opt/llvm/bin')


def compiled():
    with tempfile.TemporaryDirectory() as tmp:
        obj=Path(tmp)/'tracer.obj'
        subprocess.run([str(LLVM/'clang'),'@tools/trace/compile_flags.txt','-O2','-DRON_AUTOSTART',
                        '-c','tools/trace/tracer.c','-o',str(obj)],cwd=ROOT,check=True)
        listing=subprocess.check_output([str(LLVM/'llvm-objdump'),'-d','--disassemble-symbols=_auto_modal',str(obj)],text=True)
    code=bytearray()
    for line in listing.splitlines():
        m=re.match(r'\s*[0-9a-f]+:\s+((?:[0-9a-f]{2} )*[0-9a-f]{2})\s',line)
        if m: code.extend(bytes.fromhex(m[1]))
    if code.count(b'\xe8\0\0\0\0') != 1: raise AssertionError('unexpected callback relocation')
    at=code.index(b'\xe8\0\0\0\0')
    struct.pack_into('<i',code,at+1,0x2000-(0x1000+at+5))
    return code


def exercise(code, seed):
    rand=random.Random(seed)
    uc=Uc(UC_ARCH_X86,UC_MODE_32)
    uc.mem_map(0x1000,0x9000)
    uc.mem_write(0x1000,bytes(code))
    uc.mem_write(0x43c0,struct.pack('<I',0x3000))
    stack=bytes(rand.randrange(256) for _ in range(256))
    uc.mem_write(0x8000,stack)
    regs={r:rand.getrandbits(32) for r in (UC_X86_REG_EBX,UC_X86_REG_ECX,UC_X86_REG_EDX,
                                          UC_X86_REG_ESI,UC_X86_REG_EDI,UC_X86_REG_EBP)}
    regs.update({UC_X86_REG_EAX:0x4000,UC_X86_REG_ESP:0x8000,UC_X86_REG_EFLAGS:0x202 | sum(1<<bit for i,bit in enumerate((0,2,4,6,7,11)) if seed & (1<<i)) | (((seed >> 6) & 1)<<21)})
    for reg,value in regs.items(): uc.reg_write(reg,value)
    seen=[]
    def hook(uc,address,size,data):
        if address==0x2000:
            esp=uc.reg_read(UC_X86_REG_ESP)
            ret,self,mode=struct.unpack('<III',uc.mem_read(esp,12))
            assert self==regs[UC_X86_REG_ECX], 'wrong this'
            assert mode==struct.unpack_from('<I',stack,4)[0], 'wrong mode'
            seen.append('callback')
            for reg in (UC_X86_REG_EAX,UC_X86_REG_ECX,UC_X86_REG_EDX): uc.reg_write(reg,0xbad)
            # A cdecl callback may change arithmetic flags, not IF/DF/ID.
            uc.reg_write(UC_X86_REG_EFLAGS,uc.reg_read(UC_X86_REG_EFLAGS) & ~0x8d5)
            uc.reg_write(UC_X86_REG_ESP,esp+4)
            uc.reg_write(UC_X86_REG_EIP,ret)
        elif address==0x3000:
            assert seen==['callback']
            assert all(uc.reg_read(reg)==value for reg,value in regs.items()), 'register corruption'
            assert bytes(uc.mem_read(0x8000,256))==stack, 'native arguments changed'
            seen.append('native')
            uc.emu_stop()
    uc.hook_add(UC_HOOK_CODE,hook)
    uc.emu_start(0x1000,0x9000,count=100)
    assert seen==['callback','native'], 'did not delegate'


if __name__=='__main__':
    code=compiled()
    reject_unsafe(code)
    for seed in range(256): exercise(code,seed)
    bad=bytearray(code)
    at=bad.index(b'\x8b\x44\x24\x14');bad[at+3]=0x18
    try: exercise(bad,0)
    except AssertionError: pass
    else: raise AssertionError('wrong argument offset escaped the regression')
    print('256 compiled-trampoline cases passed; mutated argument offset rejected')
