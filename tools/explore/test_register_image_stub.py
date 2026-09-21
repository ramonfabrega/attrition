#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Execute the register-image adapter on authored machine states, offline."""
from pathlib import Path
import random
import re
import struct
import subprocess
import tempfile
from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE, UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE
from unicorn import x86_const as x
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_ECX, UC_X86_REG_EDX, UC_X86_REG_ESI, UC_X86_REG_EDI, UC_X86_REG_EBP, UC_X86_REG_ESP, UC_X86_REG_EIP, UC_X86_REG_EFLAGS
ROOT=Path(__file__).resolve().parents[2]


def emitted():
    source='''#include <stdio.h>
#include <string.h>
typedef unsigned int u32; typedef unsigned char u8;
#include "register_image_stub.h"
int main(void) {
    u8 bytes[256]; u32 in[9]={0}, out[9];
    copy_register_image(out,in);
    u32 n=build_register_image_stub(bytes,0x2000);
    return fwrite(bytes,1,n,stdout)==n && out[3]==4 ? 0 : 1;
}
'''
    with tempfile.TemporaryDirectory() as directory:
        path=Path(directory);(path/'build.c').write_text(source)
        subprocess.run(['cc','-Wall','-Wextra','-Werror','-I'+str(ROOT/'tools/explore'),str(path/'build.c'),'-o',str(path/'build')],check=True)
        return subprocess.check_output([str(path/'build')])


def reject_unsafe(code):
    decoded=subprocess.check_output(['/opt/homebrew/opt/llvm/bin/llvm-mc','--disassemble','--triple=i386'],
                                    input=' '.join(hex(byte) for byte in code)+'\n',text=True)
    if re.search(r'\b(?:pushal|popal|pushfl|popfl)\b',decoded):
        raise AssertionError('unsafe bulk-register/flags instruction in capture stub')


def exercise(code, seed):
    rng=random.Random(seed);uc=Uc(UC_ARCH_X86,UC_MODE_32);uc.mem_map(0x1000,0x9000)
    uc.mem_write(0x1000,bytes(code))
    stop=0x1000+len(code)
    stack=bytes(rng.randrange(256) for _ in range(256));original_esp=0x8000+4*(seed%4);uc.mem_write(original_esp,stack)
    regs={r:rng.getrandbits(32) for r in (UC_X86_REG_EAX,UC_X86_REG_EBX,UC_X86_REG_ECX,UC_X86_REG_EDX,UC_X86_REG_ESI,UC_X86_REG_EDI,UC_X86_REG_EBP)}
    regs.update({UC_X86_REG_ESP:original_esp,UC_X86_REG_EFLAGS:0x202 | sum(1<<bit for i,bit in enumerate((0,2,4,6,7,11)) if seed & (1<<i)) | (((seed >> 6) & 1)<<21)})
    for reg,value in regs.items():uc.reg_write(reg,value)
    extended={getattr(x,'UC_X86_REG_XMM'+str(i)):rng.getrandbits(128) for i in range(8)}
    extended[x.UC_X86_REG_MXCSR]=0x1f80|((seed%4)<<13)
    extended[x.UC_X86_REG_FPCW]=0x37f|((seed%4)<<10)
    extended.update({getattr(x,'UC_X86_REG_FP'+str(i)):(rng.getrandbits(63)|(1<<63),0x3fff+i)
                     for i in range(8)})
    extended[x.UC_X86_REG_FPTAG]=0
    extended[x.UC_X86_REG_FPSW]=(seed%8)<<11  # vary x87 stack TOP
    for reg,value in extended.items():uc.reg_write(reg,value)
    seen=[]
    def access(uc,kind,address,size,value,data):
        assert original_esp-640<=address and address+size<=original_esp, 'adapter exceeded private stack'
    uc.hook_add(UC_HOOK_MEM_READ|UC_HOOK_MEM_WRITE,access)
    def hook(uc,address,size,data):
        if address==0x2000:
            esp=uc.reg_read(UC_X86_REG_ESP)
            ret,pointer=struct.unpack('<2I',uc.mem_read(esp,8))
            assert (esp+4)%16==0, 'unaligned callback call site'
            image=struct.unpack('<9I',uc.mem_read(pointer,36))
            assert (image[3]-560)&~15 == uc.reg_read(x.UC_X86_REG_ESI), 'wrong context FXSAVE address'
            order=(x.UC_X86_REG_EDI,x.UC_X86_REG_ESI,x.UC_X86_REG_EBP,x.UC_X86_REG_ESP,
                   x.UC_X86_REG_EBX,x.UC_X86_REG_EDX,x.UC_X86_REG_ECX,x.UC_X86_REG_EAX)
            expected=tuple(regs[r]-(4 if r==x.UC_X86_REG_ESP else 0) for r in order)
            assert image[:8]==expected, 'wrong captured register image'
            flags=((image[8]>>8)&0xd5)|((image[8]&1)<<11)
            assert flags==regs[x.UC_X86_REG_EFLAGS]&0x8d5, 'wrong captured flags'
            seen.append('observer')
            for reg in (UC_X86_REG_EAX,UC_X86_REG_ECX,UC_X86_REG_EDX):uc.reg_write(reg,0xbad)
            uc.reg_write(UC_X86_REG_EFLAGS,uc.reg_read(UC_X86_REG_EFLAGS)&~0x8d5)
            for reg,value in extended.items():
                replacement=(1<<63,0x4000) if isinstance(value,tuple) else (
                    0x1f80 if reg==x.UC_X86_REG_MXCSR else 0x37f if reg==x.UC_X86_REG_FPCW else
                    0xffff if reg==x.UC_X86_REG_FPTAG else 0 if reg==x.UC_X86_REG_FPSW else 0xbad)
                uc.reg_write(reg,replacement)
            uc.reg_write(UC_X86_REG_ESP,esp+4);uc.reg_write(UC_X86_REG_EIP,ret)
        elif address==stop:
            assert seen==['observer']
            assert all(uc.reg_read(r)==v for r,v in regs.items()),'register/flag corruption'
            assert all(uc.reg_read(r)==v for r,v in extended.items()),'extended-state corruption'
            assert bytes(uc.mem_read(original_esp,256))==stack,'caller stack changed'
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
    bad=bytearray(code);at=bad.index(b'\x8d\x44\x24\x10');bad[at+3]=0x14
    try:exercise(bad,0)
    except AssertionError:pass
    else:raise AssertionError('wrong observer argument escaped')
    bad=bytearray(code);at=bad.index(b'\x0f\xae\x0e');bad[at:at+3]=b'\x90'*3
    try:exercise(bad,0)
    except AssertionError:pass
    else:raise AssertionError('missing extended-state restore escaped')
    print('256 emitted adapter cases passed; stack and extended-state mutants rejected')
