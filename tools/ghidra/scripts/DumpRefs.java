// Decompiles every function that references a named data symbol.
//
// Used to find where a runtime-built table is filled, so its contents are
// established from the code that writes it rather than from its name.
//@category Attrition
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.Reference;
import ghidra.program.model.symbol.Symbol;

import java.io.FileWriter;
import java.io.PrintWriter;
import java.util.LinkedHashSet;
import java.util.Set;

public class DumpRefs extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 2) {
            println("need: <outfile> <dataSymbol>");
            return;
        }

        Set<Function> callers = new LinkedHashSet<>();
        for (Symbol s : currentProgram.getSymbolTable().getGlobalSymbols(args[1])) {
            println("symbol " + s.getName() + " @ " + s.getAddress());
            for (Reference r : currentProgram.getReferenceManager()
                    .getReferencesTo(s.getAddress())) {
                Address from = r.getFromAddress();
                Function f = getFunctionContaining(from);
                if (f != null) {
                    println("  " + r.getReferenceType() + " from " + from + " in " + f.getName());
                    if (r.getReferenceType().isWrite()) {
                        callers.add(f);
                    }
                } else {
                    println("  " + r.getReferenceType() + " from " + from + " (no function)");
                }
            }
        }

        DecompInterface di = new DecompInterface();
        di.openProgram(currentProgram);
        PrintWriter out = new PrintWriter(new FileWriter(args[0]));
        for (Function f : callers) {
            out.println("//======== " + f.getName(true) + "  @ " + f.getEntryPoint());
            DecompileResults r = di.decompileFunction(f, 180, monitor);
            out.println(r.decompileCompleted() && r.getDecompiledFunction() != null
                    ? r.getDecompiledFunction().getC()
                    : "// decompile failed: " + r.getErrorMessage());
            out.println();
        }
        out.close();
        di.dispose();
        println("dumped " + callers.size() + " writing function(s) to " + args[0]);
    }
}
