// Decompiles every function whose entry point falls in an address range.
//
// The PDB groups a translation unit's functions together in the image, so a
// range around a known function is a cheap way to read its neighbours — which
// is where the code that drives it usually lives.
//
//   DumpRange.java <outfile> <startHex> <endHex>
//@category Attrition
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;

import java.io.FileWriter;
import java.io.PrintWriter;

public class DumpRange extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        Address start = currentProgram.getAddressFactory().getAddress(args[1]);
        Address end = currentProgram.getAddressFactory().getAddress(args[2]);

        DecompInterface di = new DecompInterface();
        if (!di.openProgram(currentProgram)) {
            println("could not open program for decompilation");
            return;
        }
        PrintWriter out = new PrintWriter(new FileWriter(args[0]));
        int found = 0;
        for (Function f : currentProgram.getFunctionManager().getFunctions(start, true)) {
            if (f.getEntryPoint().compareTo(end) > 0) {
                break;
            }
            found++;
            out.println("//======== " + f.getName(true) + "  @ " + f.getEntryPoint());
            DecompileResults r = di.decompileFunction(f, 120, monitor);
            out.println(r.decompileCompleted() && r.getDecompiledFunction() != null
                    ? r.getDecompiledFunction().getC()
                    : "// decompile failed: " + r.getErrorMessage());
            out.println();
        }
        out.close();
        di.dispose();
        println("dumped " + found + " function(s) to " + args[0]);
    }
}
