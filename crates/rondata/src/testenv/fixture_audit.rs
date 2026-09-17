//! Opt-in fixture-request evidence for the local release gate. No game contents.
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

static WRITER: Mutex<()> = Mutex::new(());

fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn append(path: &Path, name: &str, present: bool, test: &str) -> io::Result<()> {
    let line = format!("{}\t{}\t{}\n", u8::from(present), hex(name), hex(test));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())
}

pub(super) fn record(name: &str, present: bool) {
    let Some(directory) = std::env::var_os("RON_FIXTURE_AUDIT_DIR") else {
        return;
    };
    let _guard = WRITER.lock().expect("fixture audit writer poisoned");
    let path = Path::new(&directory).join(format!("{}.tsv", std::process::id()));
    append(
        &path,
        name,
        present,
        std::thread::current().name().unwrap_or("unnamed"),
    )
    .expect("cannot persist fixture-request audit");
}

#[test]
fn audit_rows_escape_names_and_record_both_outcomes() {
    let path = std::env::temp_dir().join(format!(
        "attrition-fixture-audit-{}.tsv",
        std::process::id()
    ));
    std::fs::write(&path, "").unwrap();
    append(&path, "a\tb\n", false, "test").unwrap();
    append(&path, "ok", true, "test").unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "0\t6109620a\t74657374\n1\t6f6b\t74657374\n"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn audit_write_errors_are_not_successes() {
    assert!(append(&std::env::temp_dir(), "fixture", false, "test").is_err());
}
