// Dumps the first N entries of a named data symbol, both ways.
//
// A reading tool: used to confirm what a table actually contains rather than
// inferring it from its name. See docs/DECISIONS.md entry 7.
//@category Attrition
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.symbol.Symbol;

public class DumpData extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        int n = args.length > 1 ? Integer.parseInt(args[1]) : 16;
        for (Symbol s : currentProgram.getSymbolTable().getGlobalSymbols(args[0])) {
            Address a = s.getAddress();
            println("symbol " + s.getName() + " @ " + a);
            StringBuilder sb = new StringBuilder();
            for (int i = 0; i < n; i++) {
                sb.append(getInt(a.add(4L * i))).append(' ');
            }
            println("as i32: " + sb);
            sb.setLength(0);
            for (int i = 0; i < n; i++) {
                sb.append(getByte(a.add(i)) & 0xff).append(' ');
            }
            println("as u8:  " + sb);
        }
    }
}
