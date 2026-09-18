# dagRobin

**Shared task tracking for multiple AI agents.**

A single source of truth for coordinating agents. Tasks form a DAG: each one
declares its dependencies, and dagRobin tells you what is ready, what is blocked,
and who is already working on it. No more `progress.md`, `todo_v2.md`, `done.md`.

```bash
npm install -g dagrobin
dagRobin --version
```

Install downloads the prebuilt binary for your platform from the matching GitHub
release and verifies its SHA256. Supported: **macOS and Linux, x86_64 and arm64.**

- Full command reference: <https://afa7789.github.io/dagRobin/>
- Agent protocol and drop-in prompts: [AGENTS.md](https://github.com/afa7789/dagRobin/blob/main/AGENTS.md)
- Runnable task files: [examples/](https://github.com/afa7789/dagRobin/tree/main/examples)

---

## Quick start

```bash
cd /path/to/your/project
dagRobin init                  # creates .dagrobin/db here
echo '.dagrobin/' >> .gitignore

dagRobin add setup-db "Create schema" --priority 1
dagRobin add api      "Build API"     --deps setup-db
dagRobin add ui       "Build UI"      --deps api

dagRobin ready                      # setup-db
dagRobin claim setup-db -a alice    # claim BEFORE starting
dagRobin update setup-db --status done

dagRobin ready                      # api is now unblocked
dagRobin status                     # Round: 1/3 (33%)
```

dagRobin finds its database the way git finds `.git/` — walking up from the
current directory — so subagents in any subdirectory share one database with no
flags. Override with `$DAGROBIN_DB` or `--db`.

## The agent loop

```bash
dagRobin ready --format yaml        # 1. what is available
dagRobin claim <id> -a <agent>      # 2. take it — exit 1 means someone else has it
dagRobin get <id>                   # 3. read metadata.long-description: the spec
# ... do the work, run tests ...
dagRobin update <id> --status done  # 4. release it
```

**`claim` failing is the lock.** Exit code 1 means another agent already owns the
task — pick a different one. That makes race-safe dispatch a shell loop:

```bash
for id in $(dagRobin ready --format json | jq -r '.[].id'); do
  dagRobin claim "$id" -a "$AGENT" && break
done
```

## Parallel work without collisions

Each task lists the `files:` it writes. Before dispatching agents in parallel:

```bash
dagRobin conflicts --ready-only
# Conflict: src/main.rs
#   - implement-auth: "JWT middleware" (Pending)
#   - fix-routing: "Fix router" (Pending)
```

Two tasks, one file. Either add a dependency or hand both to the same agent.

## Rounds: progress that means something

Over a long project the database fills with hundreds of finished tasks, and
`10/1230` tells you nothing about the feature you are shipping today.

```bash
dagRobin status
```
```
Round:     10/120 (8%)
  pending 108  in_progress 2  blocked 0
Archived:  380/380
All-time:  390/500 (78%)
```

```bash
dagRobin archive            # park finished work before a new round
dagRobin archive --undo --all
dagRobin clear --yes        # permanent deletion, no undo
```

`archive` keeps history and is reversible; `clear` deletes.

## Task files

```yaml
- id: auth
  title: JWT auth middleware
  priority: 2
  deps: [setup-db]
  files: [src/auth/mod.rs]
  tags: [backend]
  metadata:
    long-description: |
      Axum middleware reading the Bearer token from Authorization, decoding with
      HS256 + JWT_SECRET, injecting Claims { sub, exp, role } into extensions.
      401 with {"error":"Unauthorized"} on a missing or invalid token.
```

```bash
dagRobin import round-1.yaml
dagRobin export snapshot.yaml
```

`metadata.long-description` is the worker's only source of truth — it has no
access to the conversation that produced the task. A task without one gets
implemented by guessing.

## Command reference

| Command | What it does |
|---|---|
| `init` / `which-db` | Create `.dagrobin/db` here / print the resolved path |
| `add <id> <title>` | Create a task (`--description --priority --deps --tags --files`) |
| `get <id>` | Show one task in full |
| `list` | List tasks (`--status --priority-min --tags --include-archived --format`) |
| `ready` | Tasks whose dependencies are all done |
| `blocked` | Pending tasks and what is holding them |
| `check <id>` | Exit 0 if ready, 1 if not. No output — for scripts |
| `claim <id> -a <agent>` | Take ownership; exit 1 if already claimed |
| `update <id>` | `--status --title --description --metadata` |
| `delete <id>` | Remove a task (`--force` ignores dependents) |
| `status` / `progress` | Round progress, archived counts, all-time totals |
| `archive [ids…]` | Hide from the round (`--status --tags --all --undo`) |
| `clear --yes` | Delete permanently (`--status --archived-only`) |
| `conflicts` | Files touched by 2+ tasks (`--ready-only --format`) |
| `graph` | Render the DAG (`--format ascii\|dot\|mermaid --output`) |
| `import` / `export` | YAML in and out (`--replace` / `--status --tags`) |

All listings accept `--format table|json|yaml`.

## Other install methods

```bash
# Prebuilt binary
curl -L https://github.com/afa7789/dagRobin/releases/latest/download/dagRobin-macos-arm64.tar.gz | tar xz
sudo mv dagRobin /usr/local/bin/

# From source
cargo install --git https://github.com/afa7789/dagRobin
```

Windows is not supported. macOS and Linux only.

MIT © Arthur
