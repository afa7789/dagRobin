# dagRobin

![dagRobin](./resources/dagRobin.png)

**Shared task tracking for multiple AI agents.**

> Single source of truth for coordinating multiple AI agents. Export/import as YAML, visualize dependencies, prevent duplicate work.
> 
> [Agent Integration](#for-ai-agents) - Add to your CLAUDE.md for multi-agent coordination

---

## TL;DR

```bash
dagRobin add task-id "Description" --deps dep-id --priority 1
dagRobin ready                    # What can I work on?
dagRobin claim task-id --agent me # Claim before starting!
dagRobin update task-id --status done
dagRobin export tasks.yaml        # Save to file
dagRobin import tasks.yaml --merge # Load from file
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

dagRobin is designed for autonomous agents working together. Add this to your project's CLAUDE.md or similar:

```markdown
# Task Management

This project uses dagRobin for task coordination.

## Commands
- `dagRobin ready` - What can I work on?
- `dagRobin list` - Show all tasks
- `dagRobin graph --format mermaid` - Visualize dependencies

## Workflow
1. Start: run `dagRobin ready`
2. Pick a task, mark it `in_progress` with your agent name
3. Work on it, mark it `done` when finished
4. Repeat
```

**Why agents love it:**
- No more conflicting task lists
- One place for everything (no `progress.md`, `todo_v2.md`, `done.md`)
- Fast O(1) lookups instead of parsing files

---

## Example Prompts for AI Agents

Copy these into your agent prompts to get started:

### Basic Agent Prompt

```
You have access to dagRobin for task management. Use it to coordinate your work.

Setup:
1. Run `dagRobin ready` to see what tasks are available
2. Use `dagRobin claim <task-id> --agent your-name` to claim a task
3. Complete the task
4. Mark it as done with `dagRobin update <task-id> --status done`
5. Run `dagRobin ready` again

Key commands:
- dagRobin ready --format yaml
- dagRobin claim <id> --agent your-name
- dagRobin update <id> --status done
- dagRobin graph --format mermaid

IMPORTANT: If dagRobin claim returns exit code 1, another agent is already working on that task. Pick a different one!
```

### Full Claude Code Integration

Add this to your CLAUDE.md file:

```markdown
# Project Task Management

This project uses dagRobin. You MUST use it for all task coordination.

## First Thing You Do
Every session starts with: dagRobin ready

## Task Lifecycle
1. dagRobin ready --format yaml
2. dagRobin claim <task-id> --agent claudecode
3. Do the work
4. dagRobin update <task-id> --status done --metadata "agent:claudecode" --metadata "completed:$(date +%s)"

## Rules
- ALWAYS use `dagRobin claim` before starting work
- If claim fails (exit code 1), pick a different task
- NEVER work on unclaimed tasks
- NEVER skip the task system and work on random things
- When blocked, explain which dependencies are blocking you

## Useful Commands
dagRobin list                           # See all tasks
dagRobin blocked                        # What's waiting on something
dagRobin graph --format mermaid        # Visual overview
dagRobin check <id>                    # Is this task ready? (exit code 0/1)
dagRobin conflicts --ready-only        # File conflicts among ready tasks
```

### Orchestrator Agent Prompt

You are the orchestrator. Your job is to:
1. Load tasks from dagRobin
2. Assign tasks to worker agents
3. Monitor progress
4. Handle dependencies

```bash
# Get tasks ready to work on
dagRobin ready --format yaml

# Check specific task
dagRobin check <task-id>

# Assign to agent (update metadata)
dagRobin update <task-id> --status in_progress --metadata "agent:worker-1"

# Mark complete
dagRobin update <task-id> --status done

# Check for file conflicts before assigning parallel work
dagRobin conflicts --ready-only --format json

# See overall progress
dagRobin list --format table
dagRobin graph --format mermaid
```

### Worker Agent Prompt

You are a worker agent. Your workflow:

```bash
# 1. Ask for work
dagRobin ready --format yaml

# 2. Claim a task
dagRobin update <task-id> --status in_progress --metadata "agent:worker-2" --metadata "started:$(date +%s)"

# 3. Do the work (implement the feature, fix the bug, etc)

# 4. Mark complete
dagRobin update <task-id> --status done --metadata "agent:worker-2" --metadata "completed:$(date +%s)"

# 5. Get next task
dagRobin ready
```

### Multi-Agent Coordination Example

```bash
# Orchestrator: Create tasks with clear ownership
dagRobin add auth-worker "Implement authentication" --priority 1 --tags "backend,auth"
dagRobin add api-worker "Build REST API" --deps auth-worker --priority 2 --tags "backend,api"
dagRobin add test-worker "Write integration tests" --deps api-worker --priority 3 --tags "testing"

# Worker 1: Claims auth task
dagRobin update auth-worker --status in_progress --metadata "agent:claudeaude"
# ... does auth work ...
dagRobin update auth-worker --status done

# Worker 2: Now api-worker is ready, claims it
dagRobin update api-worker --status in_progress --metadata "agent:worker-2"
# ... does API work ...
dagRobin update api-worker --status done

# Worker 3: test-worker now ready
dagRobin update test-worker --status in_progress --metadata "agent:worker-3"
```

---

## Configuration

Database location (defaults to `dagrobin.db` in current folder):

```bash
dagRobin --db ~/.config/dagRobin/mytasks.db list
```

---

## License

MIT OR Apache-2.0 - use it however you want.
