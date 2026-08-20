// Prints every value of a PDB enum, sorted, to a file — the whole TypeIndex
// space in one pass rather than a handful of values per question.
//
//   DumpEnumAll.java <outfile> TypeIndex [OtherEnum ...]
//@category Attrition
import ghidra.app.script.GhidraScript;
import ghidra.program.model.data.DataType;
import ghidra.program.model.data.DataTypeManager;
import ghidra.program.model.data.Enum;

import java.io.FileWriter;
import java.io.PrintWriter;
import java.util.Arrays;
import java.util.Iterator;

public class DumpEnumAll extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        PrintWriter w = new PrintWriter(new FileWriter(args[0]));
        DataTypeManager dtm = currentProgram.getDataTypeManager();
        for (int a = 1; a < args.length; a++) {
            Iterator<DataType> it = dtm.getAllDataTypes();
            while (it.hasNext()) {
                DataType dt = it.next();
                if (!(dt instanceof Enum e) || !dt.getName().equals(args[a])) {
                    continue;
                }
                w.println("enum " + dt.getPathName() + "  // " + e.getCount() + " values");
                long[] vals = e.getValues();
                Arrays.sort(vals);
                for (long v : vals) {
                    for (String n : e.getNames(v)) {
                        w.println("  " + v + "\t0x" + Long.toHexString(v) + "\t" + n);
                    }
                }
                println("dumped " + dt.getPathName() + ": " + e.getCount() + " values");
                break;
            }
        }
        w.close();
    }
}
