# dagRobin for AI Agents

Drop-in instructions for autonomous agents. Copy the blocks below into your
`CLAUDE.md`, `AGENTS.md`, `.cursorrules` or system prompt — they are written to be
pasted as-is.

- New to dagRobin? Read the [README](./README.md) first.
- Task file examples: [`examples/`](./examples/).
- Full command reference: <https://afa7789.github.io/dagrobin/>

---

## 0. One-time setup (human or orchestrator)

```bash
cd /path/to/project
dagRobin init                 # creates .dagrobin/db here
echo '.dagrobin/' >> .gitignore
```

Every agent launched from anywhere inside the project now resolves the same
database automatically — dagRobin walks up looking for `.dagrobin/db`, the way
git finds `.git/`. No `-d` flag, no path juggling between subagents.

```bash
dagRobin which-db             # prove which database you are on
```

---

## 1. Drop-in block: worker agent

````markdown
## Task coordination — dagRobin

This project coordinates all work through dagRobin. You MUST use it.

### Every session starts here

```bash
dagRobin ready --format yaml
```

### The loop

1. `dagRobin ready --format yaml` — see what is available.
2. Pick the lowest `priority` number you can actually do.
3. `dagRobin claim <id> -a <your-agent-name>`
   - **Exit code 1 means another agent already owns it.** Do not retry, do not
     force it. Pick a different task.
4. `dagRobin get <id>` — read `metadata.long-description`. That is your spec.
   There is no other source of truth; you do not have the original conversation.
5. Do the work. Run the project's tests and linter.
6. `dagRobin update <id> --status done --metadata "notes:<what you did>"`
7. Back to step 1.

### If you get stuck

```bash
dagRobin update <id> --status blocked --metadata "notes:why you are blocked"
```

Say in your reply which dependency or missing information blocked you.

### Rules

- NEVER work on a task you have not claimed.
- NEVER invent work that is not in dagRobin. If it needs doing, it needs a task.
- NEVER mark done without running the project's verification (tests/lint/build).
- One task at a time. Finish or block it before claiming another.
````

---

## 2. Drop-in block: orchestrator agent

````markdown
## dagRobin orchestration

You create and assign tasks. You NEVER claim them — claiming is for workers.

### Planning a round

1. Decompose the goal into 15–20 tasks, 30–60 min each.
2. Write them to YAML (see `examples/`), then:
   ```bash
   dagRobin import round-N.yaml
   ```
3. Every task MUST carry a `metadata.long-description` complete enough that an
   agent with zero context can implement it correctly.
4. Every task MUST list the `files:` it will touch.

### Before dispatching agents in parallel

```bash
dagRobin conflicts --ready-only --format json
```

Two tasks that touch the same file cannot run in parallel. Fix it by either:
- making one depend on the other (`deps:`), or
- handing both to the same agent.

### While the round runs

```bash
dagRobin status              # 12/120 (10%) — how far along the round is
dagRobin ready               # what is dispatchable right now
dagRobin list --status in_progress    # who is working on what
dagRobin blocked             # what is stuck and on which dependency
dagRobin graph --format mermaid       # visual overview
```

### Closing a round

```bash
dagRobin export .claude/tasks-snapshot.yaml   # snapshot before touching anything
dagRobin archive                              # park finished work
dagRobin status                               # next round starts at 0/N
```
````

---

## 3. Shell patterns worth stealing

### Claim the first task you can actually get

```bash
for id in $(dagRobin ready --format json | jq -r '.[].id'); do
  if dagRobin claim "$id" -a "$AGENT"; then
    echo "working on $id"
    break
  fi
done
```

`claim` failing is the lock. Looping over `ready` and taking the first successful
claim is race-safe with any number of agents.

### Gate a script on readiness

```bash
dagRobin check deploy && ./deploy.sh
```

`check` prints nothing and exits 0 (ready) or 1 (not ready).

### Pull the spec out of a task

```bash
dagRobin get "$ID"                       # human-readable detail view

# Just the long-description, for piping into a subagent prompt
dagRobin list --format json \
  | jq -r --arg id "$ID" '.[] | select(.id==$id) | .metadata["long-description"]'
```

### Watch a round from another terminal

```bash
while true; do clear; dagRobin status; dagRobin list --status in_progress; sleep 10; done
```

### Machine-readable progress

```bash
dagRobin status --format json
# {"active":{"pending":108,"in_progress":2,"done":10,"blocked":0},
#  "archived":{"pending":0,"in_progress":0,"done":380,"blocked":0}}
```

---

## 4. Why agents get this wrong (and how not to)

| Mistake | Consequence | Do this instead |
|---|---|---|
| Starting work before `claim` | Two agents write the same file | `claim` first, always; exit 1 = move on |
| Retrying a failed `claim` | Deadlock or duplicated work | Treat exit 1 as final. Pick another task. |
| Task with no `long-description` | Worker guesses, guesses wrong | Spec it before importing |
| No `files:` on tasks | Parallel agents collide | List every file the task writes |
| `clear` instead of `archive` | History gone, no undo | `archive` parks; `clear --yes` deletes |
| Marking done without tests | Broken main, silent failures | Run the gate, then `update --status done` |
| Working outside the task list | Untracked work, lost context | If it matters, it gets a task |

---

## 5. Minimal end-to-end example

```bash
# Orchestrator
dagRobin init
cat > round-1.yaml <<'YAML'
- id: setup-db
  title: Create schema
  priority: 1
  files: [migrations/001_init.sql]
  metadata:
    long-description: |
      Create the initial Postgres schema: users(id uuid pk, email citext unique,
      created_at timestamptz default now()). Migration file only, no app code.
- id: auth
  title: JWT auth middleware
  priority: 2
  deps: [setup-db]
  files: [src/auth/mod.rs]
  metadata:
    long-description: |
      Axum middleware reading the Bearer token from Authorization, decoding with
      HS256 + JWT_SECRET, injecting Claims { sub, exp, role } into extensions.
      401 with {"error":"Unauthorized"} on a missing or invalid token.
YAML
dagRobin import round-1.yaml
dagRobin conflicts --ready-only    # "No file conflicts detected."

# Worker
dagRobin ready                     # setup-db
dagRobin claim setup-db -a worker-1
dagRobin get setup-db              # read long-description, implement it
dagRobin update setup-db --status done --metadata "notes:migration added, tests green"

dagRobin status                    # Round: 1/2 (50%)
dagRobin ready                     # auth is now unblocked
```
