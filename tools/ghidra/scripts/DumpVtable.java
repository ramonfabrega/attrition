// Prints a C++ vtable as slot offset -> function name, so that a decompiled
// indirect call like (*(code **)(*this + 0xcc))() can be read as the method it
// actually is. Ghidra resolves those to offsets, never to names.
//
//   DumpVtable.java <vftableSymbol> [slotCount]
//
// The symbol is the PDB's own, e.g. "Unit::vftable". With no count, walks
// until an entry does not point at a function.
//@category Attrition
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.Symbol;

public class DumpVtable extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 1) {
            println("need: <vftableSymbol> [slotCount]");
            return;
        }
        int want = args.length > 1 ? Integer.parseInt(args[1]) : 0;

        // Matched loosely: MSVC's multiple-inheritance vtables are named
        // "vftable{for `Type'}", which Ghidra renders as vftable_for_Type_
        // in decompiled code; comparing only letters and digits lets either
        // spelling find it.
        String want_ = args[0].replaceAll("[^A-Za-z0-9]", "").toLowerCase();
        Address base = null;
        for (Symbol s : currentProgram.getSymbolTable().getAllSymbols(true)) {
            String full = s.getName(true).replaceAll("[^A-Za-z0-9]", "").toLowerCase();
            if (full.equals(want_)) {
                base = s.getAddress();
                println("matched symbol " + s.getName(true));
                break;
            }
        }
        if (base == null) {
            println("no symbol named " + args[0]);
            return;
        }
        println("vtable " + args[0] + " @ " + base);
        for (int i = 0; want == 0 || i < want; i++) {
            Address slot = base.add((long) i * 4);
            long target;
            try {
                target = currentProgram.getMemory().getInt(slot) & 0xffffffffL;
            } catch (Exception e) {
                break;
            }
            Address fa = base.getAddressSpace().getAddress(target);
            Function f = getFunctionAt(fa);
            if (f == null) {
                if (want == 0) {
                    break;
                }
                println(String.format("  +0x%x  %s  (not a function)", i * 4, fa));
                continue;
            }
            println(String.format("  +0x%x  %s", i * 4, f.getName(true)));
        }
    }
}
