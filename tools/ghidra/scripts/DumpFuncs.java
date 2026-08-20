// Decompiles every function whose name contains one of the given substrings
// and writes the result to a file.
//
// Usage (headless):
//   analyzeHeadless <proj> ron -process riseofnations.exe -noanalysis \
//     -scriptPath <dir> -postScript DumpFuncs.java <outfile> <pat> [<pat>...]
//
// Output is a working note for reading, not a source to copy from: this
// project reads the original to understand it and writes its own
// implementation from a written spec. See docs/DECISIONS.md entry 7.
//@category Attrition
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.Function;

import java.io.FileWriter;
import java.io.PrintWriter;

public class DumpFuncs extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 2) {
            println("need: <outfile> <namePattern>...");
            return;
        }

        DecompInterface di = new DecompInterface();
        if (!di.openProgram(currentProgram)) {
            println("could not open program for decompilation");
            return;
        }

        PrintWriter out = new PrintWriter(new FileWriter(args[0]));
        int found = 0;
        for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
            String name = f.getName();
            boolean want = false;
            for (int i = 1; i < args.length; i++) {
                if (name.contains(args[i])) {
                    want = true;
                    break;
                }
            }
            if (!want) {
                continue;
            }
            found++;
            out.println("//======== " + f.getName(true) + "  @ " + f.getEntryPoint());
            DecompileResults r = di.decompileFunction(f, 180, monitor);
            if (r.decompileCompleted() && r.getDecompiledFunction() != null) {
                out.println(r.getDecompiledFunction().getC());
            } else {
                out.println("// decompile failed: " + r.getErrorMessage());
            }
            out.println();
        }
        out.close();
        di.dispose();
        println("dumped " + found + " function(s) to " + args[0]);
    }
}
