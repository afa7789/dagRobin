# dagRobin

![dagRobin](./resources/dagRobin.png)

**Shared task tracking for multiple AI agents.**

> Single source of truth for coordinating multiple AI agents. Export/import as YAML, visualize dependencies, prevent duplicate work.
> 
> [Agent Integration](#for-ai-agents) - Add to your CLAUDE.md for multi-agent coordination
>
> [Documentation site](https://afa7789.github.io/dagRobin/) - Full command reference
>
> [AGENTS.md](./AGENTS.md) - Drop-in protocol, prompts and shell patterns for agents
>
> [examples/](./examples/) - Runnable task files you can import right now

---

## TL;DR

```bash
dagRobin add task-id "Description" --deps dep-id --priority 1
dagRobin ready                    # What can I work on?
dagRobin claim task-id --agent me # Claim before starting!
dagRobin update task-id --status done
dagRobin status                   # How far along is this round? 10/120 (8%)
dagRobin archive                  # Park finished work before the next round
dagRobin export tasks.yaml        # Save to file
dagRobin import tasks.yaml        # Load from file
```

dagRobin is an external task database that multiple AI agents (Claude, Cowork, OpenRoute, etc.) can query and update simultaneously. No more markdown files, no more "who's working on what?", no more duplicate work.

---

### The Simple Version

Imagine you have multiple AI agents working on the same project:
- Claude is fixing the auth module
- Cowork is building the API
- OpenRoute is writing tests

Without dagRobin: Agents step on each other, overwrite progress files, don't know who's doing what.

With dagRobin: All agents query the same database. If an agent starts a task (`in_progress`), other agents can see it's already being worked on and skip it.

dagRobin GIVES YOU THE TOOL to prevent agents from stepping on each other. It's a single source of truth that you can export/import, track progress, and coordinate work across tools.

---

Pt-Br

```
Criei um sistema de tickets pra organizar o que os agentes precisam fazer e guardei tudo num banco externo (tipo Redis). O orquestrador fica olhando nesse banco e distribui as tarefas. Quando um agente começa ou termina algo, ele atualiza o registro direto ali.

Isso resolve aquela bagunça de ficar procurando status em arquivo Markdown ou lista de "to-do". O acesso é direto e rápido, o que salva muito tempo.

O melhor de tudo é que centraliza tudo. Antes, cada agente criava seu próprio arquivo pra acompanhar o progresso e virava uma zona. Agora, como o contexto do ticket fica "fora" do modelo, dá pra colocar dois agentes de programas diferentes trabalhando no mesmo projeto.

Se os meus tokens do Claude acabarem no meio do caminho, por exemplo, eu consigo subir um modelo gratuito e ele assume exatamente de onde o outro parou, porque o contexto atualizado está salvo nesse banco externo.
```


---

### Important: Mark Tasks Before Starting

**ALWAYS mark a task as `in_progress` BEFORE starting work.**

```bash
# Check if task is already being worked on
dagRobin get <task-id>

# If status is "InProgress" with agent metadata, skip it!
# Only claim if status is "Pending"

# To claim a task:
dagRobin update <task-id> --status in_progress --metadata "agent:your-name"
```

If you try to work on a task that's already `in_progress`, another agent is already working on it. Don't duplicate the work!

---

## Quick Examples

```bash
# "I need to do this first"
dagRobin add setup-db "Setup the database" --priority 1

# "This depends on the database being done"
dagRobin add build-api "Build the API" --deps setup-db --priority 2

# "What can I work on right now?"
dagRobin ready

# "Show me everything"
dagRobin list

# "I finished the database!"
dagRobin update setup-db --status done

# "Now what can I do?"
dagRobin ready
```

---

## Features

- **Dependencies made easy** - Tell dagRobin what depends on what, it handles the rest
- **Always know what's next** - The `ready` command shows only tasks you can actually do now
- **See the big picture** - Visualize your task graph in ASCII, Mermaid, or DOT format
- **Multiple agents, one source of truth** - Perfect for coordinating multiple AI agents
- **Fast and lightweight** - Built in Rust, database embedded in a single file
- **Conflict detection** - Detect file-level conflicts between tasks before agents step on each other
- **Export/Import** - Share task lists as YAML files

---

## Installation

### npm (easiest — no Rust toolchain needed)

```bash
npm install -g dagrobin
dagRobin --version
```

Install pulls the prebuilt binary for your platform from the matching GitHub release
and verifies its SHA256. Supported: macOS and Linux, x86_64 and arm64. Update an npm install with `npm install -g dagrobin@latest` (or `dagRobin upgrade`, which delegates to npm).

### Updating

```bash
# Installed from source or as a prebuilt binary — update in place:
dagRobin upgrade            # download, verify SHA256, replace this binary
dagRobin upgrade --check    # exit 0 = up to date, 1 = update available
dagRobin upgrade --force    # reinstall even when already latest

# Installed via npm — the upgrade command delegates to npm:
npm install -g dagrobin@latest   # manual alternative to `dagRobin upgrade`
```

### Prebuilt binary

```bash
# Example: macOS ARM (Apple Silicon)
curl -L https://github.com/afa7789/dagRobin/releases/latest/download/dagRobin-macos-arm64.tar.gz | tar xz
sudo mv dagRobin /usr/local/bin/
```

### From source (Cargo)

```bash
# Clone the repo
git clone https://github.com/afa7789/dagRobin.git
cd dagRobin

# Build from source
cargo build --release

# Run directly
./target/release/dagRobin --help

# Or install globally
cargo install --path .
```

Full command reference: **<https://afa7789.github.io/dagRobin/>**

---

## Day-to-Day Usage

### Adding Tasks

```bash
# Simple task
dagRobin add t1 "Write documentation"

# With priority (lower = more important)
dagRobin add t2 "Fix critical bug" --priority 1

# With dependencies
dagRobin add t3 "Add tests" --deps t2

# With tags (for filtering)
dagRobin add t4 "Update README" --tags docs,ux

# With file context (know which files a task touches)
dagRobin add t5 "Refactor auth" --files "src/auth.rs,middleware.rs"
```

### Checking What to Do

```bash
# What's ready to work on?
dagRobin ready

# Show everything
dagRobin list

# Only pending tasks
dagRobin list --status pending

# Filter by tag
dagRobin list --tags backend

# Show blocked tasks (waiting on something)
dagRobin blocked

# Detect file conflicts between tasks
dagRobin conflicts

# Only check ready tasks, output as JSON
dagRobin conflicts --ready-only --format json
```

### Claiming Tasks (Recommended)

**Before starting any work, ALWAYS claim the task first!**

```bash
# Claim a task for your agent
dagRobin claim <task-id> --agent your-agent-name

# Example:
dagRobin claim feature-auth --agent claudeaude

# If someone else already claimed it, you'll see:
# Task 'feature-auth' is already being worked on by 'worker-2'
# Do NOT start work on this task!
# Exit code: 1
```

The `claim` command:
- Verifies the task isn't already being worked on
- Marks it as `in_progress`
- Records who is working on it
- Prevents other agents from duplicating work

### Updating Tasks

```bash
# Mark as done
dagRobin update t1 --status done

# Change title
dagRobin update t1 --title "New title"

# Add notes/metadata
dagRobin update t1 --metadata "notes:This was tricky, took 2 hours"

# Multiple metadata (semicolon separates pairs, commas allowed in values)
dagRobin update t1 --metadata "notes:a,b,c;tags:tech"

# Or use multiple --metadata flags
dagRobin update t1 --metadata "notes:test" --metadata "agent:me"
```

### Progress, Archive & Clear

```bash
# How far is the current round? (archived tasks excluded)
dagRobin status            # alias: dagRobin progress
# Round:     10/120 (8%)
#   pending 108  in_progress 2  blocked 0
# Archived:  380/380
# All-time:  390/500 (78%)

dagRobin status --format json   # machine-readable counts

# Before a big round: park finished work so it stops counting
dagRobin archive                    # archives every Done task (default)
dagRobin archive t1 t2              # specific ids
dagRobin archive --status done --status blocked
dagRobin archive --tags sprint-3
dagRobin archive --all              # everything still active
dagRobin archive --undo --all       # bring them back

# Archived tasks stay in the DB but are hidden from list/ready/blocked/status
dagRobin list --include-archived

# Nuclear option: delete tasks permanently
dagRobin clear --yes                     # everything
dagRobin clear --yes --status done       # only done tasks
dagRobin clear --yes --archived-only     # drop the archive
```

`archive` keeps history (counted under All-time), `clear` deletes it.

### Visualization

```bash
# See the dependency graph
dagRobin graph

# Mermaid format (great for Markdown)
dagRobin graph --format mermaid

# Save to file
dagRobin graph --format dot --output diagram.dot
```

### Import/Export

```bash
# Save all tasks to a file
dagRobin export my-tasks.yaml

# Merge with existing tasks (default - updates existing, adds new)
dagRobin import their-tasks.yaml

# Replace everything (deletes all existing tasks first)
dagRobin import fresh-start.yaml --replace
```

---

## For AI Agents

dagRobin exists so that autonomous agents can share one task list without stepping
on each other. The full agent protocol — drop-in blocks for `CLAUDE.md`, shell
patterns, and the mistakes agents actually make — lives in **[AGENTS.md](./AGENTS.md)**.
Runnable task files are in **[`examples/`](./examples/)**.

The short version:

```bash
dagRobin ready --format yaml        # 1. what is available
dagRobin claim <id> -a <agent>      # 2. take it — exit 1 means someone else has it
dagRobin get <id>                   # 3. read metadata.long-description: that is the spec
# ... do the work, run tests ...
dagRobin update <id> --status done  # 4. release it
```

**Why agents love it:**
- No more conflicting task lists
- One place for everything (no `progress.md`, `todo_v2.md`, `done.md`)
- Fast O(1) lookups instead of parsing files
- The claim lock is an exit code, so it works from any language or shell

---

## Example Prompts for AI Agents

Copy these into your agent prompts to get started.

### Worker agent

````markdown
## Task coordination — dagRobin

This project coordinates all work through dagRobin. You MUST use it.

Every session starts with `dagRobin ready --format yaml`.

### The loop
1. `dagRobin ready --format yaml` — see what is available.
2. Pick the lowest `priority` number you can actually do.
3. `dagRobin claim <id> -a <your-agent-name>`
   - **Exit code 1 means another agent already owns it.** Do not retry.
     Pick a different task.
4. `dagRobin get <id>` — read `metadata.long-description`. That is your spec;
   you do not have the original conversation.
5. Do the work. Run the project's tests and linter.
6. `dagRobin update <id> --status done --metadata "notes:<what you did>"`
7. Back to step 1.

### If you get stuck
`dagRobin update <id> --status blocked --metadata "notes:why"`, then say which
dependency or missing information blocked you.

### Rules
- NEVER work on a task you have not claimed.
- NEVER invent work that is not in dagRobin. If it needs doing, it needs a task.
- NEVER mark done without running the project's verification.
- One task at a time. Finish or block it before claiming another.
````

### Orchestrator agent

````markdown
## dagRobin orchestration

You create and assign tasks. You NEVER claim them — claiming is for workers.

### Planning a round
1. Decompose into 15–20 tasks of 30–60 min each.
2. Write them to YAML (see `examples/`) and `dagRobin import round-N.yaml`.
3. Every task carries a `metadata.long-description` complete enough for an agent
   with zero context, and lists the `files:` it will touch.

### Before dispatching agents in parallel
```bash
dagRobin conflicts --ready-only --format json
```
Two tasks touching the same file cannot run in parallel. Either add a dependency
or hand both to the same agent.

### While the round runs
```bash
dagRobin status                        # 12/120 (10%)
dagRobin ready                         # dispatchable right now
dagRobin list --status in_progress     # who is working on what
dagRobin blocked                       # what is stuck, and on what
dagRobin graph --format mermaid        # visual overview
```

### Closing a round
```bash
dagRobin export .claude/tasks-snapshot.yaml
dagRobin archive
dagRobin status                        # next round starts at 0/N
```
````

### Race-safe claiming in a shell loop

`claim` failing *is* the lock. Loop over `ready` and take the first successful
claim — this is safe with any number of agents running at once:

```bash
for id in $(dagRobin ready --format json | jq -r '.[].id'); do
  if dagRobin claim "$id" -a "$AGENT"; then
    echo "working on $id"
    break
  fi
done
```

### Multi-agent coordination example

```bash
# Orchestrator: create the round
dagRobin add auth-worker "Implement authentication" --priority 1 --tags backend
dagRobin add api-worker  "Build REST API" --deps auth-worker --priority 2 --tags backend
dagRobin add test-worker "Write integration tests" --deps api-worker --priority 3 --tags testing
dagRobin conflicts --ready-only        # no two ready tasks share a file

# Worker 1
dagRobin claim auth-worker -a worker-1     # exit 0: it is mine
# ... work ...
dagRobin update auth-worker --status done

# Worker 2 — api-worker just became ready
dagRobin claim api-worker -a worker-2
# ... work ...
dagRobin update api-worker --status done

# Worker 3 tries the same task Worker 2 already holds
dagRobin claim api-worker -a worker-3      # exit 1 -> pick something else

# Orchestrator watches
dagRobin status                            # 2/3 (67%)
```

---

## Configuration

dagRobin resolves its database the way git finds `.git/`, so subagents running in
any subdirectory of the project automatically share one database:

| # | Source | Notes |
|---|---|---|
| 1 | `-d` / `--db <path>` | Explicit override |
| 2 | `$DAGROBIN_DB` | Inherited by subprocesses and subagents automatically |
| 3 | `.dagrobin/db` | Walk-up search from the current directory (`dagRobin init` creates it) |
| 4 | `~/.local/share/dagRobin/dagrobin.db` | Global fallback |

```bash
dagRobin init                                  # create .dagrobin/db here
echo '.dagrobin/' >> .gitignore                # per-machine state, not source
dagRobin which-db                              # prove which database is in use

export DAGROBIN_DB=/tmp/scratch                # scratch database for experiments
dagRobin --db ~/.config/dagRobin/mytasks.db list   # one-off override
```

---

## License

MIT — use it however you want. See [LICENSE](./LICENSE).
