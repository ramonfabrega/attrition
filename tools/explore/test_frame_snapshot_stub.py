"""Execute the exact end-frame adapter plus displaced-prologue continuation."""
import subprocess,tempfile,unittest
from pathlib import Path
from test_register_image_stub import exercise,reject_unsafe
ROOT=Path(__file__).resolve().parent

class StubTests(unittest.TestCase):
    def test_emitter_preserves_state_and_continues(self):
        source='''#include <stdio.h>
#include <string.h>
typedef unsigned int u32;typedef unsigned char u8;
#include "register_image_stub.h"
#include "frame_snapshot_stub.h"
int main(void){u8 bytes[256],nops[6]={0x90,0x90,0x90,0x90,0x90,0x90};
u32 in[9]={0},out[9];copy_register_image(out,in);
u32 a=build_register_image_stub(bytes,0x2000);
u32 n=build_frame_snapshot_stub(bytes,0x1000,0x1000+a+5,0x2000,nops,6);
return fwrite(bytes,1,n,stdout)==n && out[3]==4?0:1;}
'''
        with tempfile.TemporaryDirectory() as folder:
            p=Path(folder);(p/'test.c').write_text(source)
            subprocess.run(['cc','-Wall','-Wextra','-Werror','-I'+str(ROOT),str(p/'test.c'),'-o',str(p/'test')],check=True)
            code=subprocess.check_output([str(p/'test')])
        reject_unsafe(code)
        for seed in range(256):exercise(code,seed)
        bad=bytearray(code);at=bad.index(b'\x0f\xae\x0e');bad[at:at+3]=b'\x90'*3
        with self.assertRaisesRegex(AssertionError,'extended-state'):exercise(bad,0)

if __name__=='__main__':unittest.main()
