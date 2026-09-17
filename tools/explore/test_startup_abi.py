#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Compile the real wrappers; verify authored stdcall arguments and returns."""
from pathlib import Path
import random
import re
import struct
import subprocess
import tempfile
from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_ECX, UC_X86_REG_EDX, UC_X86_REG_ESI, UC_X86_REG_EDI, UC_X86_REG_EBP, UC_X86_REG_ESP, UC_X86_REG_EIP
ROOT = Path(__file__).resolve().parents[2]
LLVM = Path('/opt/homebrew/opt/llvm/bin')


def compiled():
    prefix = '''typedef unsigned int u32; typedef unsigned char u8; typedef int i32;
#define WINAPI __attribute__((stdcall))
#define g_base 0x400000u
#define K_INFO 5
extern void emit(u32,u32,u32,u32,u32,u32,u32);
extern void flush(void);
extern i32 auto_different(const void *,const void *,u32);
extern void auto_branch(u32,u32,u8);
#include "live_startup_probe.h"
void *get_startup(void) {return (void *)startup_mf;}
void *get_shutdown(void) {return (void *)shutdown_mf;}
'''
    with tempfile.TemporaryDirectory() as directory:
        source, obj = Path(directory)/'probe.c', Path(directory)/'probe.obj'
        source.write_text(prefix)
        subprocess.run([str(LLVM/'clang'), '@tools/trace/compile_flags.txt', '-O2', '-Wno-unused-function',
                        '-I'+str(ROOT/'tools/explore'), '-c',str(source),'-o',str(obj)],cwd=ROOT,check=True)
        listing = subprocess.check_output([str(LLVM/'llvm-objdump'),'-dr',str(obj)],text=True)
    code, entries = bytearray(4096), {}
    for line in listing.splitlines():
        m = re.match(r'([0-9a-f]+) <_(startup_mf@8|shutdown_mf@0)>:',line)
        if m: entries[m[2]] = int(m[1],16)
        m = re.match(r'\s*([0-9a-f]+):\s+((?:[0-9a-f]{2} )*[0-9a-f]{2})\s',line)
        if m:
            at, data = int(m[1],16), bytes.fromhex(m[2]); code[at:at+len(data)] = data
        m = re.search(r'([0-9a-f]+):\s+IMAGE_REL_I386_REL32\s+(_emit|_flush)',line)
        if m:
            at = int(m[1],16); target = {'_emit':0x2000,'_flush':0x3000}[m[2]]
            struct.pack_into('<i',code,at,target-(0x1000+at+4))
    assert len(entries)==2
    return code, entries


def exercise(code, entry, startup, seed):
    rng = random.Random(seed)
    uc = Uc(UC_ARCH_X86,UC_MODE_32); uc.mem_map(0x1000,0x9000); uc.mem_map(0xca9000,4096)
    uc.mem_write(0x1000,bytes(code)); uc.mem_write(0xca9f2c,struct.pack('<II',0x4000,0x4000))
    args = (rng.getrandbits(32),rng.getrandbits(32)) if startup else ()
    result = rng.getrandbits(32)
    stack = struct.pack('<I',0x5000)+b''.join(struct.pack('<I',a) for a in args)+b'guard'*16
    uc.mem_write(0x8000,stack); uc.reg_write(UC_X86_REG_ESP,0x8000)
    saved = {r:rng.getrandbits(32) for r in (UC_X86_REG_EBX,UC_X86_REG_ESI,UC_X86_REG_EDI,UC_X86_REG_EBP)}
    for reg,value in saved.items(): uc.reg_write(reg,value)
    events=[]
    def hook(uc,address,size,data):
        if address not in (0x2000,0x3000,0x4000,0x5000): return
        esp=uc.reg_read(UC_X86_REG_ESP)
        if address==0x5000:
            assert events==['emit','flush','native','emit','flush']
            assert uc.reg_read(UC_X86_REG_EAX)==result
            assert uc.reg_read(UC_X86_REG_ESP)==0x8004+4*len(args), 'stack cleanup'
            assert all(uc.reg_read(r)==v for r,v in saved.items())
            assert bytes(uc.mem_read(0x8000,len(stack)))==stack
            uc.emu_stop(); return
        ret=struct.unpack('<I',uc.mem_read(esp,4))[0]
        cleanup=0
        if address==0x2000:
            row=struct.unpack('<7I',uc.mem_read(esp+4,28)); phase=int('native' in events)
            assert row==(5,180,1 if startup else 2,phase,*(args if startup else (0,0)),result if phase else 0)
            events.append('emit')
        elif address==0x3000: events.append('flush')
        else:
            assert tuple(struct.unpack('<'+'I'*len(args),uc.mem_read(esp+4,4*len(args))))==args
            events.append('native');cleanup=4*len(args)
        for reg in (UC_X86_REG_EAX,UC_X86_REG_ECX,UC_X86_REG_EDX):uc.reg_write(reg,0xbad)
        if address==0x4000:uc.reg_write(UC_X86_REG_EAX,result)
        uc.reg_write(UC_X86_REG_ESP,esp+4+cleanup);uc.reg_write(UC_X86_REG_EIP,ret)
    uc.hook_add(UC_HOOK_CODE,hook);uc.emu_start(0x1000+entry,0x9000,count=200)
    assert uc.reg_read(UC_X86_REG_EIP)==0x5000


if __name__=='__main__':
    code, entries=compiled()
    for seed in range(256):
        for name,entry in entries.items():exercise(code,entry,name=='startup_mf@8',seed)
    bad=bytearray(code);at=bad.index(b'\xc2\x08\x00');bad[at+1]=4
    try:exercise(bad,entries['startup_mf@8'],True,0)
    except AssertionError:pass
    else:raise AssertionError('wrong callee cleanup escaped')
    print('512 compiled wrapper cases passed; wrong stack cleanup rejected')
