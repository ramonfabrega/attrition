/* RonDriver — a fixed-path app bundle that owns this machine's macOS TCC grants.
 *
 * Why this exists: macOS attributes Accessibility, Screen Recording and
 * Automation to the *responsible process*, and for a native Claude Code
 * install that is the bare versioned binary
 * `~/.local/share/claude/versions/<VERSION>` — a path that carries the version
 * number, so every update writes a new one and silently revokes all three
 * grants (2026-09-03, read off `tccd`'s own AUTHREQ_SUBJECT lines; docs/ORACLE.md,
 * "The permissions belong to a path"). A capture driven from under Claude Code
 * therefore breaks on a schedule nobody here controls, and it breaks *quietly*:
 * `cliclick` exits 0 and prints a plausible cursor position while posting no
 * events at all.
 *
 * Launched through LaunchServices (`open -a`), this bundle is its own
 * responsible process, and every child it spawns inherits that. Its path never
 * changes and its bytes never change, so the grants are given once and kept.
 *
 * It must **spawn and wait**, never `exec`: an exec would replace this image
 * with /bin/zsh and hand the attribution straight back to the interpreter,
 * which is the bug this whole bundle exists to escape.
 *
 *   RonDriver <cwd> <logfile> <program> [args...]
 *
 * stdout and stderr of the child are appended to <logfile>; the exit status is
 * the child's. Keep this file frozen — rebuilding changes the cdhash, and a
 * changed cdhash costs a re-grant, which is the one thing it is here to avoid.
 */
#include <errno.h>
#include <fcntl.h>
#include <spawn.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

extern char **environ;

int main(int argc, char *argv[]) {
    if (argc < 4) {
        fprintf(stderr, "usage: RonDriver <cwd> <logfile> <program> [args...]\n");
        return 64;
    }
    const char *cwd = argv[1], *logfile = argv[2];

    if (chdir(cwd) != 0) {
        fprintf(stderr, "RonDriver: chdir(%s): %s\n", cwd, strerror(errno));
        return 66;
    }

    int fd = open(logfile, O_WRONLY | O_CREAT | O_APPEND, 0644);
    if (fd < 0) {
        fprintf(stderr, "RonDriver: open(%s): %s\n", logfile, strerror(errno));
        return 73;
    }

    posix_spawn_file_actions_t fa;
    posix_spawn_file_actions_init(&fa);
    posix_spawn_file_actions_adddup2(&fa, fd, STDOUT_FILENO);
    posix_spawn_file_actions_adddup2(&fa, fd, STDERR_FILENO);

    pid_t child;
    int rc = posix_spawn(&child, argv[3], &fa, NULL, &argv[3], environ);
    posix_spawn_file_actions_destroy(&fa);
    close(fd);
    if (rc != 0) {
        fprintf(stderr, "RonDriver: spawn(%s): %s\n", argv[3], strerror(rc));
        return 71;
    }

    int status = 0;
    while (waitpid(child, &status, 0) < 0 && errno == EINTR) continue;
    if (WIFSIGNALED(status)) return 128 + WTERMSIG(status);
    return WIFEXITED(status) ? WEXITSTATUS(status) : 70;
}
