# dagRobin — Task Management Workflow

Use this workflow to coordinate task management with dagRobin, track progress, and coordinate multiple agents.

---

## Step 0 — Install dagRobin

1. Install Rust if not already installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone and build:
   ```bash
   git clone https://github.com/afa7789/dagRobin.git
   cd dagRobin
   cargo build --release
   ```

3. Run directly:
   ```bash
   ./target/release/dagRobin --help
   ```

---

## Step 1 — Check What Can Be Done

Run dagRobin to see ready tasks (tasks with all dependencies completed):

```bash
dagRobin ready --format yaml
```

This returns all tasks you can work on right now.

---

## Step 2 — Claim a Task

Before starting work, mark the task as `in_progress`:

```bash
dagRobin update <task-id> --status in_progress --metadata "agent=your-name,started=$(date +%s)"
```

This prevents other agents from working on the same task.

---

## Step 3 — Do the Work

Complete the task. Read files, write code, run tests — whatever the task requires.

---

## Step 4 — Mark as Done

When finished:

```bash
dagRobin update <task-id> --status done --metadata "agent=your-name,completed=$(date +%s)"
```

---

## Step 5 — Check Progress

```bash
# See all tasks
dagRobin list --format table

# See blocked tasks (waiting on something)
dagRobin blocked

# Visual overview
dagRobin graph --format mermaid
```

---

## Step 6 — Get Next Task

```bash
dagRobin ready --format yaml
```

Repeat from Step 2.

---

## Workflow for Multiple Agents

### Orchestrator Agent

The orchestrator creates tasks and assigns them:

```bash
# Create tasks with clear dependencies
dagRobin add setup "Initial setup" --priority 1
dagRobin add backend "Build backend" --deps setup --priority 2
dagRobin add frontend "Build frontend" --deps setup --priority 2
dagRobin add tests "Integration tests" --deps backend,frontend --priority 3

# Monitor progress
dagRobin list --format table
```

### Worker Agent

Workers pick up ready tasks:

```bash
# 1. Ask for work
dagRobin ready --format yaml

# 2. Claim task (checks if already being worked on)
dagRobin claim <task-id> --agent worker-2

# If claim fails (another agent is working), pick another task

# 3. Do work
# ... implement feature ...

# 4. Mark done
dagRobin update <task-id> --status done --metadata "agent=worker-2,completed=$(date +%s)"

# 5. Get next task
dagRobin ready
```

---

## Loop — Continue Until Done

Repeat Steps 1-6 until:

1. `dagRobin ready` returns no tasks
2. All tasks have status `done`

---

## Useful Commands Reference

```bash
dagRobin add <id> <title>           # Add a task
dagRobin list                         # List all tasks
dagRobin ready                        # Tasks ready to work on
dagRobin blocked                       # Blocked tasks
dagRobin claim <id> --agent <name>  # Claim task (prevents conflicts)
dagRobin check <id>                   # Is task ready? (exit code 0/1)
dagRobin update <id> --status done   # Mark as done
dagRobin graph --format mermaid      # Visual dependency graph
dagRobin export tasks.yaml            # Export to YAML
dagRobin import tasks.yaml --merge   # Import from YAML
```
