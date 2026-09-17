#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Execute bytes emitted by the production RNG/frame stub builder."""
from pathlib import Path
import random
import re
import struct
import subprocess
import tempfile
from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_ECX, UC_X86_REG_EDX, UC_X86_REG_ESI, UC_X86_REG_EDI, UC_X86_REG_EBP, UC_X86_REG_ESP, UC_X86_REG_EIP, UC_X86_REG_EFLAGS
ROOT=Path(__file__).resolve().parents[2]


def emitted():
    source='''#include <stdio.h>
#include <string.h>
typedef unsigned int u32; typedef unsigned char u8;
#include "hook_stub.h"
int main(void) {
    u8 bytes[128], authored[5]={0x90,0x90,0x90,0x90,0x90};
    u32 n=build_hook_stub(bytes,0x1000,0x3000,3,0x2000,authored,5);
    return fwrite(bytes,1,n,stdout)==n ? 0 : 1;
}
'''
    with tempfile.TemporaryDirectory() as directory:
        path=Path(directory);(path/'build.c').write_text(source)
        subprocess.run(['cc','-Wall','-Wextra','-Werror','-I'+str(ROOT/'tools/trace'),str(path/'build.c'),'-o',str(path/'build')],check=True)
        return subprocess.check_output([str(path/'build')])


def reject_unsafe(code):
    decoded=subprocess.check_output(['/opt/homebrew/opt/llvm/bin/llvm-mc','--disassemble','--triple=i386'],
                                    input=' '.join(hex(byte) for byte in code)+'\n',text=True)
    if re.search(r'\b(?:pushal|popal|pushfl|popfl)\b',decoded):
        raise AssertionError('unsafe bulk-register/flags instruction in capture stub')


def exercise(code, seed):
    rng=random.Random(seed);uc=Uc(UC_ARCH_X86,UC_MODE_32);uc.mem_map(0x1000,0x9000)
    uc.mem_write(0x1000,bytes(code))
    stack=bytes(rng.randrange(256) for _ in range(256));uc.mem_write(0x8000,stack)
    regs={r:rng.getrandbits(32) for r in (UC_X86_REG_EAX,UC_X86_REG_EBX,UC_X86_REG_ECX,UC_X86_REG_EDX,UC_X86_REG_ESI,UC_X86_REG_EDI,UC_X86_REG_EBP)}
    regs.update({UC_X86_REG_ESP:0x8000,UC_X86_REG_EFLAGS:0x202 | sum(1<<bit for i,bit in enumerate((0,2,4,6,7,11)) if seed & (1<<i)) | (((seed >> 6) & 1)<<21)})
    for reg,value in regs.items():uc.reg_write(reg,value)
    seen=[]
    def hook(uc,address,size,data):
        if address==0x2000:
            esp=uc.reg_read(UC_X86_REG_ESP)
            ret,kind,self,ebp,caller,arg0=struct.unpack('<6I',uc.mem_read(esp,24))
            assert (kind,self,ebp,caller,arg0)==(3,regs[UC_X86_REG_ECX],regs[UC_X86_REG_EBP],*struct.unpack_from('<II',stack))
            seen.append('observer')
            for reg in (UC_X86_REG_EAX,UC_X86_REG_ECX,UC_X86_REG_EDX):uc.reg_write(reg,0xbad)
            uc.reg_write(UC_X86_REG_EFLAGS,uc.reg_read(UC_X86_REG_EFLAGS)&~0x8d5)
            uc.reg_write(UC_X86_REG_ESP,esp+4);uc.reg_write(UC_X86_REG_EIP,ret)
        elif address==0x3005:
            assert seen==['observer']
            assert all(uc.reg_read(r)==v for r,v in regs.items()),'register/flag corruption'
            assert bytes(uc.mem_read(0x8000,256))==stack,'caller stack changed'
            seen.append('native');uc.emu_stop()
    uc.hook_add(UC_HOOK_CODE,hook);uc.emu_start(0x1000,0x9000,count=100)
    assert seen==['observer','native']


if __name__=='__main__':
    code=emitted()
    reject_unsafe(code)
    for opcode in (0x60,0x61,0x9c,0x9d):
        try:reject_unsafe(bytes([opcode]))
        except AssertionError:pass
        else:raise AssertionError('instruction guard did not reject unsafe opcode')
    for seed in range(256):exercise(code,seed)
    bad=bytearray(code);at=bad.index(b'\x8b\x44\x24\x14');bad[at+3]=0x18
    try:exercise(bad,0)
    except AssertionError:pass
    else:raise AssertionError('wrong observer argument escaped')
    print('256 production-emitter cases passed; wrong stack operand rejected')
