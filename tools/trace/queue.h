/* The `queued` byte an `@` line's issuer takes (items 3448, 3504).
 * Included by `tracer.c`, and by `queue_test.c` on the host, so the choice
 * is tested as the DLL compiles it. Needs `i32`.
 *
 * `QueuePos` is the PDB's (journal 590): QUEUE_FIRST 0, QUEUE_LAST 1,
 * QUEUE_NEW 2. An unmodified right-click passes QUEUE_NEW; the shift-click
 * QUEUE_LAST (`@attackadd`, `@buildadd`); the ctrl-click QUEUE_FIRST
 * (`@repairins`). `CommandManager::issue_swarm_around@009416b0` copies the
 * byte into the packet unchanged, `process_swarm_around@00949970` hands it
 * to `Group::action_swarm_around@0070fbe0`, and its QUEUE_FIRST arm
 * (`:95-108`) is the insert: `set_up_insert`, `action_halt`, the swarm again
 * as QUEUE_NEW, `finish_insert`. */

#ifndef QUEUE_H
#define QUEUE_H

#define QUEUE_FIRST 0
#define QUEUE_LAST 1
#define QUEUE_NEW 2

/* `add` is the shift-click's verb, `first` the ctrl-click's; a verb is at
 * most one of them. */
static i32 issue_queue(i32 add, i32 first) {
    return first ? QUEUE_FIRST : add ? QUEUE_LAST : QUEUE_NEW;
}

#endif
