# dagRobin

![dagRobin](./resources/dagRobin.png)

**The task manager that actually helps you get things done.**

Ever feel lost about what to do next? dagRobin keeps track of your tasks and their dependencies, so you always know exactly what can be worked on right now.

---

### The Simple Version

Think of dagRobin like a to-do list on steroids. You can say things like:
- "First I need to set up the database"
- "Then I can build the API"
- "Finally, I can write tests"

dagRobin figures out what you can actually do right now, and what you need to wait for.

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
```

### Updating Tasks

```bash
# Mark as done
dagRobin update t1 --status done

# Change title
dagRobin update t1 --title "New title"

# Add notes/metadata
dagRobin update t1 --metadata "notes=This was tricky, took 2 hours"
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

# Share with someone else
dagRobin import their-tasks.yaml --merge

# Replace everything
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
2. Pick one task and mark it as in_progress
3. Complete the task
4. Mark it as done
5. Run `dagRobin ready` again

Key commands:
- dagRobin ready --format yaml
- dagRobin list --status pending
- dagRobin update <id> --status in_progress
- dagRobin update <id> --status done
- dagRobin graph --format mermaid
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
2. dagRobin update <task-id> --status in_progress --metadata "agent=claudeaude"
3. Do the work
4. dagRobin update <task-id> --status done --metadata "agent=claudeaude,completed=$(date +%s)"

## Rules
- NEVER work on a task without marking it in_progress first
- ALWAYS check dagRobin ready before starting work
- NEVER skip the task system and work on random things
- When blocked, explain which dependencies are blocking you

## Useful Commands
dagRobin list                           # See all tasks
dagRobin blocked                        # What's waiting on something
dagRobin graph --format mermaid        # Visual overview
dagRobin check <id>                    # Is this task ready? (exit code 0/1)
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
dagRobin update <task-id> --status in_progress --metadata "agent=worker-1"

# Mark complete
dagRobin update <task-id> --status done

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
dagRobin update <task-id> --status in_progress --metadata "agent=worker-2,started=$(date +%s)"

# 3. Do the work (implement the feature, fix the bug, etc)

# 4. Mark complete
dagRobin update <task-id> --status done --metadata "agent=worker-2,completed=$(date +%s)"

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
dagRobin update auth-worker --status in_progress --metadata "agent=claudeaude"
# ... does auth work ...
dagRobin update auth-worker --status done

# Worker 2: Now api-worker is ready, claims it
dagRobin update api-worker --status in_progress --metadata "agent=worker-2"
# ... does API work ...
dagRobin update api-worker --status done

# Worker 3: test-worker now ready
dagRobin update test-worker --status in_progress --metadata "agent=worker-3"
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
