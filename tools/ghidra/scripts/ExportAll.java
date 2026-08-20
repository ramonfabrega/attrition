// Decompiles every function in the program, once, into a directory tree —
// one file per function, plus an index and a dump of every structure the
// PDB loaded — so that later reading is grep over files rather than a
// two-minute Ghidra pass per question.
//
// Before decompiling, any method `Class::m` whose first parameter is not a
// pointer to a named structure gets `this` retyped to `Class *` when the
// data type manager holds a structure of that name. That is what turns
// `field_0xNN` into a field name. The change is saved with the project.
//
// Usage (headless):
//   analyzeHeadless <proj> ron -process riseofnations.exe -noanalysis \
//     -scriptPath <dir> -postScript ExportAll.java <outdir>
//
// Output is a working note for reading, not a source to copy from; see
// docs/DECISIONS.md entry 7. Nothing written here may enter the repo.
//@category Attrition
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.decompiler.parallel.DecompileConfigurer;
import ghidra.app.decompiler.parallel.DecompilerCallback;
import ghidra.app.decompiler.parallel.ParallelDecompiler;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.data.DataType;
import ghidra.program.model.data.DataTypeComponent;
import ghidra.program.model.data.DataTypeManager;
import ghidra.program.model.data.Pointer;
import ghidra.program.model.data.PointerDataType;
import ghidra.program.model.data.Structure;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Parameter;
import ghidra.program.model.symbol.SourceType;
import ghidra.util.task.TaskMonitor;

import java.io.File;
import java.io.FileWriter;
import java.io.PrintWriter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentLinkedQueue;

public class ExportAll extends GhidraScript {
    private File root;
    private final ConcurrentLinkedQueue<String> index = new ConcurrentLinkedQueue<>();

    private static String safe(String s) {
        return s.replaceAll("[^A-Za-z0-9_.\\-]", "_");
    }

    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        if (args.length < 1) {
            println("need: <outdir>");
            return;
        }
        root = new File(args[0]);
        new File(root, "funcs").mkdirs();

        DataTypeManager dtm = currentProgram.getDataTypeManager();
        Map<String, Structure> structs = new HashMap<>();
        Iterator<Structure> it = dtm.getAllStructures();
        while (it.hasNext()) {
            Structure s = it.next();
            structs.putIfAbsent(s.getName(), s);
        }
        println("structures: " + structs.size());

        // Pass 1: type `this` where the PDB did not.
        int retyped = 0;
        List<Function> funcs = new ArrayList<>();
        for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
            if (f.isThunk()) continue;
            funcs.add(f);
            String full = f.getName(true);
            int sep = full.lastIndexOf("::");
            if (sep < 0) continue;
            String cls = full.substring(0, sep);
            int sep2 = cls.lastIndexOf("::");
            if (sep2 >= 0) cls = cls.substring(sep2 + 2);
            Structure s = structs.get(cls);
            if (s == null) continue;
            if (f.getParameterCount() == 0) continue;
            Parameter p = f.getParameter(0);
            DataType dt = p.getDataType();
            boolean named = dt instanceof Pointer
                && ((Pointer) dt).getDataType() instanceof Structure;
            if (named) continue;
            try {
                p.setDataType(new PointerDataType(s), SourceType.USER_DEFINED);
                retyped++;
            } catch (Exception e) {
                // leave it
            }
        }
        println("functions: " + funcs.size() + "  retyped this: " + retyped);

        // Pass 2: every structure, one file.
        try (PrintWriter out = new PrintWriter(new FileWriter(new File(root, "types.txt")))) {
            Iterator<Structure> it2 = dtm.getAllStructures();
            while (it2.hasNext()) {
                Structure s = it2.next();
                out.println("struct " + s.getPathName() + "  // size 0x" + Integer.toHexString(s.getLength()));
                for (DataTypeComponent c : s.getDefinedComponents()) {
                    String n = c.getFieldName();
                    if (n == null) n = "";
                    out.println(String.format("  +0x%-5x %-40s %s", c.getOffset(), c.getDataType().getName(), n));
                }
                out.println();
            }
        }

        // Pass 2b: every vtable, slot offset -> method. The decompiler renders
        // a virtual call as an offset and never as a name; this is the ring
        // that decodes it.
        int vtables = 0;
        try (PrintWriter out = new PrintWriter(new FileWriter(new File(root, "vtables.txt")))) {
            for (ghidra.program.model.symbol.Symbol sym :
                    currentProgram.getSymbolTable().getAllSymbols(true)) {
                // "vftable", and "vftable_for_<Base>_" where a class inherits
                // from two bases — UnitTypeData::vftable_for_Type_ is the one
                // every type_avail call goes through.
                if (!sym.getName().startsWith("vftable")) continue;
                ghidra.program.model.address.Address base = sym.getAddress();
                out.println("vtable " + sym.getName(true) + "  @ " + base);
                vtables++;
                for (int i = 0; ; i++) {
                    ghidra.program.model.address.Address slot = base.add((long) i * 4);
                    long target;
                    try {
                        target = currentProgram.getMemory().getInt(slot) & 0xffffffffL;
                    } catch (Exception e) {
                        break;
                    }
                    Function f = getFunctionAt(base.getAddressSpace().getAddress(target));
                    if (f == null) break;
                    out.println(String.format("  +0x%-4x %s", i * 4, f.getName(true)));
                }
                out.println();
            }
        }
        println("vtables: " + vtables);

        // Pass 3: decompile everything, in parallel.
        DecompilerCallback<Void> cb = new DecompilerCallback<Void>(currentProgram,
            new DecompileConfigurer() {
                @Override
                public void configure(DecompInterface d) {
                    d.toggleCCode(true);
                    d.toggleSyntaxTree(false);
                    d.setSimplificationStyle("decompile");
                }
            }) {
            @Override
            public Void process(DecompileResults r, TaskMonitor m) throws Exception {
                Function f = r.getFunction();
                String full = f.getName(true);
                String addr = f.getEntryPoint().toString();
                // Directory is the innermost namespace (the class); file is the
                // method. Both truncated — template instantiations produce names
                // longer than a path may be — and the full name is in the index.
                String ns = full.contains("::") ? full.substring(0, full.lastIndexOf("::")) : "_global";
                if (ns.contains("::")) ns = ns.substring(ns.lastIndexOf("::") + 2);
                String name = full.contains("::") ? full.substring(full.lastIndexOf("::") + 2) : full;
                ns = safe(ns);
                if (ns.length() > 60) ns = ns.substring(0, 60);
                name = safe(name);
                if (name.length() > 80) name = name.substring(0, 80);
                File dir = new File(new File(root, "funcs"), ns);
                dir.mkdirs();
                File out = new File(dir, name + "@" + addr + ".c");
                try (PrintWriter w = new PrintWriter(new FileWriter(out))) {
                    w.println("//======== " + full + "  @ " + addr);
                    if (r.decompileCompleted() && r.getDecompiledFunction() != null) {
                        w.println(r.getDecompiledFunction().getC());
                    } else {
                        w.println("// decompile failed: " + r.getErrorMessage());
                    }
                }
                index.add(addr + "\t" + full + "\t" + root.toPath().relativize(out.toPath()));
                return null;
            }
        };
        cb.setTimeout(120);
        ParallelDecompiler.decompileFunctions(cb, funcs, monitor);
        cb.dispose();

        List<String> lines = new ArrayList<>(index);
        java.util.Collections.sort(lines);
        try (PrintWriter out = new PrintWriter(new FileWriter(new File(root, "INDEX.tsv")))) {
            for (String l : lines) out.println(l);
        }
        println("exported " + lines.size() + " function(s) to " + root);
    }
}
