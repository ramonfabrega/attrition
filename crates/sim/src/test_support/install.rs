/// The install's root: `$RON_INSTALL`, else the checkout's `game/`, else
/// — from a worktree under `.claude/worktrees/<name>/` — the main
/// checkout's `game/`, three directories up. The last is what stops
/// every worktree session typing the variable by hand (2,147 times in
/// 120 sessions, by the transcripts, before 2026-09-01).
pub(crate) fn install_root() -> Option<String> {
    if let Ok(r) = std::env::var("RON_INSTALL") {
        return Some(r);
    }
    let here = env!("CARGO_MANIFEST_DIR");
    [
        format!("{here}/../../game"),
        format!("{here}/../../../../../game"),
    ]
    .into_iter()
    .find(|g| std::path::Path::new(g).join("riseofnations.exe").is_file())
}
