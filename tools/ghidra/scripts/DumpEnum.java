// Prints the names a PDB enum gives to a set of values, and decompiles the
// callers of named functions. Two questions that both come down to "what does
// the original call this?".
//
//   DumpEnum.java <outfile> TypeIndex 61,62,400 [callersOf=Unit::suffer_attrition]
//@category Attrition
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.data.DataType;
import ghidra.program.model.data.DataTypeManager;
import ghidra.program.model.data.Enum;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.Reference;
import ghidra.program.model.symbol.Symbol;

import java.io.FileWriter;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

public class DumpEnum extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        String out = args[0];
        String enumName = args[1];
        List<Long> wanted = new ArrayList<>();
        for (String s : args[2].split(",")) {
            wanted.add(Long.parseLong(s.trim()));
        }

        DataTypeManager dtm = currentProgram.getDataTypeManager();
        Iterator<DataType> it = dtm.getAllDataTypes();
        while (it.hasNext()) {
            DataType dt = it.next();
            if (!(dt instanceof Enum e) || !dt.getName().equals(enumName)) {
                continue;
            }
            println("enum " + dt.getPathName());
            for (long v : wanted) {
                String n = e.getName(v);
                println("  " + v + " = " + (n == null ? "(no name)" : n));
            }
            break;
        }

        if (args.length < 4) {
            return;
        }
        Set<Function> callers = new LinkedHashSet<>();
        for (int i = 3; i < args.length; i++) {
            for (Symbol s : currentProgram.getSymbolTable().getGlobalSymbols(args[i])) {
                for (Reference r : currentProgram.getReferenceManager()
                        .getReferencesTo(s.getAddress())) {
                    Function f = getFunctionContaining(r.getFromAddress());
                    if (f != null && !f.getName().equals(args[i])) {
                        callers.add(f);
                    }
                }
            }
        }
        DecompInterface di = new DecompInterface();
        di.openProgram(currentProgram);
        PrintWriter w = new PrintWriter(new FileWriter(out));
        for (Function f : callers) {
            w.println("//======== " + f.getName(true) + "  @ " + f.getEntryPoint());
            DecompileResults r = di.decompileFunction(f, 180, monitor);
            w.println(r.decompileCompleted() && r.getDecompiledFunction() != null
                    ? r.getDecompiledFunction().getC()
                    : "// decompile failed: " + r.getErrorMessage());
            w.println();
        }
        w.close();
        di.dispose();
        println("dumped " + callers.size() + " caller(s) to " + out);
    }
}
