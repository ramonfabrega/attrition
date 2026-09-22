"""One bounded re-entry into the saved-search callee, preserving model state.

The caller supplies a previously returned runner and explicit argument/mode/GPR
boundary. Heap/global bytes, lifetime history, extended CPU state and cumulative
service budgets persist. Only stack definedness and per-call diagnostics reset.
This is a frozen-environment experiment, not a native next-frame replay. The
ordinary runner.run reset semantics remain unchanged.
"""
from unicorn import UcError, x86_const as x
from replay_capsule import require, REG_IDS
from payload_replay import fingerprint
from resume_frontier import ENTRY


def continue_once(runner,regs,boundary):
    require(runner.uc.reg_read(x.UC_X86_REG_EIP)==runner.stop,'prior call did not return')
    require(not runner.copy_active and not runner.fill_active,'modeled service still active')
    require(set(boundary)=={'callee_arguments','modes'},'explicit argument and mode boundary required')
    require(len(regs)==len(REG_IDS) and all(type(v) is int and 0<=v<=0xffffffff for v in regs),
            'complete unsigned 32-bit register boundary required')
    patches=[]
    for name,data in boundary.items():
        matches=[r for r in runner.regions if r.name==name]
        require(len(matches)==1,'unique boundary region required')
        region=matches[0]
        require(isinstance(data,bytes),'immutable boundary bytes required')
        require(region.writable and not region.executable and not region.scratch and len(data)==len(region.data),'invalid boundary extent')
        patches.append((region,data))
    # Rearm this invocation's frame. Heap/global initializedness and lifetime
    # history persist. Backing bytes do not authorize future scratch reads.
    stacks=[r for r in runner.regions if r.name=='scratch']
    require(len(stacks)==1 and stacks[0].scratch and stacks[0].writable,'unique scratch stack required')
    stack=stacks[0]
    runner.initialized.difference_update(range(stack.address,stack.address+len(stack.data)))
    for region,data in patches:
        runner.uc.mem_write(region.address,data)
    for reg,value in zip(REG_IDS,regs):runner.uc.reg_write(reg,value)
    runner.instructions=0;runner.writes=[]
    error=None
    try:
        runner.uc.emu_start(ENTRY,runner.stop,count=runner.budget)
        require(runner.uc.reg_read(x.UC_X86_REG_EIP)==runner.stop,'instruction budget exhausted')
    except (ValueError,UcError) as exc:error=str(exc)
    return dict(returned=error is None,reason=error,instructions=runner.instructions,
                eax=runner.uc.reg_read(x.UC_X86_REG_EAX),fingerprint=fingerprint(runner,error))

