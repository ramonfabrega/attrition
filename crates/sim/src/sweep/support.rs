/// The rows `tools/emu/<script>` prints for the install's executable, or
/// `None` (said on stderr) where the install or `uv` is absent.
pub(crate) fn emu_rows(script: &str) -> Option<String> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Some(install) = crate::testenv::install_root() else {
        eprintln!("skipping: set RON_INSTALL (the sweep runs the install's executable)");
        return None;
    };
    let out = std::process::Command::new("uv")
        .args(["run", &format!("{root}/tools/emu/{script}")])
        .arg(format!("{install}/riseofnations.exe"))
        .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8(o.stdout).expect("utf-8 rows")),
        Ok(o) => panic!("{script} failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            None
        }
    }
}

/// One row, `<args…> -> <result>`, split into its integers.
pub(crate) fn row(line: &str) -> (Vec<i64>, i64) {
    let (lhs, rhs) = line
        .split_once(" -> ")
        .expect("a row is `<args…> -> <result>`");
    let args = lhs
        .split_whitespace()
        .map(|s| s.parse().expect("an integer argument"))
        .collect();
    (args, rhs.trim().parse().expect("an integer result"))
}

/// One row with a tail, `<args…> -> <result> | <tail> | …` (item 2061): its
/// integers and the tail's fields, each trimmed, in order.
#[allow(dead_code)] // Some standalone partitions use only the scalar row helper.
pub(crate) fn tail_row(line: &str) -> (Vec<i64>, i64, Vec<&str>) {
    let (head, tail) = line.split_once(" | ").unwrap_or((line, ""));
    let (args, eax) = row(head);
    let tail = if tail.is_empty() && !line.contains(" | ") {
        Vec::new()
    } else {
        tail.split(" | ").map(str::trim).collect()
    };
    (args, eax, tail)
}
