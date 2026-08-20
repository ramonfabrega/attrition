// Prints a PDB struct's components, optionally only those in an offset range.
//
//   DumpStruct.java <TypeName> [startOffset] [endOffset]
//
// The whole point of having private symbols is that field names do not have to
// be guessed. This is how you stop writing `field_0xa`.
//@category Attrition
import ghidra.app.script.GhidraScript;
import ghidra.program.model.data.DataType;
import ghidra.program.model.data.DataTypeComponent;
import ghidra.program.model.data.DataTypeManager;
import ghidra.program.model.data.Structure;

import java.util.Iterator;

public class DumpStruct extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        int start = args.length > 1 ? Integer.decode(args[1]) : 0;
        int end = args.length > 2 ? Integer.decode(args[2]) : Integer.MAX_VALUE;

        DataTypeManager dtm = currentProgram.getDataTypeManager();
        Iterator<DataType> it = dtm.getAllDataTypes();
        while (it.hasNext()) {
            DataType dt = it.next();
            if (!(dt instanceof Structure s) || !dt.getName().equals(args[0])) {
                continue;
            }
            println("struct " + dt.getPathName() + "  (" + s.getLength() + " bytes)");
            for (DataTypeComponent c : s.getDefinedComponents()) {
                int o = c.getOffset();
                if (o < start || o > end) {
                    continue;
                }
                println(String.format("  +%-5d 0x%-4x %-24s %s",
                        o, o, c.getDataType().getName(),
                        c.getFieldName() == null ? "(unnamed)" : c.getFieldName()));
            }
        }
    }
}
