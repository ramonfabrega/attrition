// Lists every function that references an absolute address, and says whether
// it reads or writes. For finding what fills a .bss table at startup.
//
//   DumpAddrRefs.java <hexAddress>
//
// LIMITATION, learned the hard way: this only sees references the disassembler
// could resolve statically — direct absolute addressing such as
// `mov ecx, [table + eax*4]`. A field reached through a pointer held in a
// register (`constantsc->supply_radius`) produces NO reference to its address,
// so a "no references" answer is evidence of nothing unless a known reader
// shows up as a positive control. Always include one.
//@category Attrition
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.Reference;

public class DumpAddrRefs extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 1) {
            println("need: <hexAddress>");
            return;
        }
        String hex = args[0].startsWith("0x") ? args[0].substring(2) : args[0];
        Address base = currentProgram.getAddressFactory()
                .getDefaultAddressSpace()
                .getAddress(Long.parseLong(hex, 16));
        println("address " + base);

        // A table is indexed, so references land across its whole extent, not
        // only on its first byte. Sweep a window.
        int span = args.length > 1 ? Integer.decode(args[1]) : 0x400;
        int found = 0;
        for (int i = 0; i < span; i += 4) {
            Address a = base.add(i);
            for (Reference r : currentProgram.getReferenceManager().getReferencesTo(a)) {
                Function f = getFunctionContaining(r.getFromAddress());
                println("  +" + i + " <- " + (f == null ? "(no function)" : f.getName(true))
                        + " @ " + r.getFromAddress() + " " + r.getReferenceType());
                found++;
            }
        }
        println("total " + found + " reference(s)");
    }
}
