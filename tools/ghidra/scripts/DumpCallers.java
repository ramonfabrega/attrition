// Lists every reference to named symbols and decompiles the functions that
// make them. References that land in data rather than code — a vtable slot —
// are reported with the data address, since that is the thread to pull for a
// virtual call.
//
//   DumpCallers.java <outfile> <symbol>...
//@category Attrition
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.Reference;


import java.io.FileWriter;
import java.io.PrintWriter;
import java.util.LinkedHashSet;
import java.util.Set;

public class DumpCallers extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        // Match on the *qualified* name. getGlobalSymbols only sees the global
        // namespace, so it silently misses every C++ method — which reads as
        // "no callers" rather than as "wrong lookup".
        Set<Function> callers = new LinkedHashSet<>();
        for (int i = 1; i < args.length; i++) {
            boolean any = false;
            for (Function target : currentProgram.getFunctionManager().getFunctions(true)) {
                if (!target.getName(true).contains(args[i])) {
                    continue;
                }
                any = true;
                println("target " + target.getName(true) + " @ " + target.getEntryPoint());
                int n = 0;
                for (Reference r : currentProgram.getReferenceManager()
                        .getReferencesTo(target.getEntryPoint())) {
                    n++;
                    Function f = getFunctionContaining(r.getFromAddress());
                    if (f == null) {
                        println("  " + r.getReferenceType() + " from data @ "
                                + r.getFromAddress());
                    } else if (!f.getEntryPoint().equals(target.getEntryPoint())) {
                        println("  " + r.getReferenceType() + " from " + f.getName(true));
                        callers.add(f);
                    }
                }
                println("  (" + n + " reference(s))");
            }
            if (!any) {
                println("no function matched " + args[i]);
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
        println("dumped " + callers.size() + " caller(s) to " + args[0]);
    }
}
