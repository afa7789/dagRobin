# dagRobin — Task Management Workflow

dagRobin is an external task database that gives you the TOOL to prevent multiple AI agents from stepping on each other. It provides a single source of truth for tracking progress, coordinating work, and maintaining task history across different tools and agents.

---

## What dagRobin Does

dagRobin is NOT automatic coordination. It's a SHARED DATABASE that YOU USE to:

- Track which tasks exist and their status
- Prevent two agents from working on the same task
- Maintain a single source of truth (no more scattered markdown files)
- Export/import tasks for backup and sharing
- Query who is working on what

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

## Step 1 — Set Up Task Database

Initialize your project with dagRobin:

```bash
# Create first task
dagRobin add setup "Initial project setup" --priority 1
```

Or import existing tasks:

```bash
dagRobin import tasks.yaml --merge
dagRobin export backup.yaml
```

---

## Step 2 — Add Tasks with Dependencies

Create tasks that form a dependency graph:

```bash
# Foundation tasks (no dependencies)
dagRobin add setup-db "Setup database" --priority 1
dagRobin add setup-config "Setup configuration" --priority 1

# Tasks that depend on foundation
dagRobin add build-api "Build REST API" --deps setup-db,setup-config --priority 2
dagRobin add build-frontend "Build frontend" --deps setup-config --priority 2

# Final tasks (depend on intermediate tasks)
dagRobin add tests "Integration tests" --deps build-api,build-frontend --priority 3
```

For batch import, use YAML:

```yaml
- id: setup-1
  title: Initialize project structure
  priority: 1
  deps: []

- id: feature-auth
  title: Implement authentication
  priority: 2
  deps: [setup-1]
  tags: [backend,security]
  files: [src/auth.rs,src/middleware.rs]

- id: feature-api
  title: Build REST API
  priority: 2
  deps: [setup-1]
  tags: [backend,api]
  files: [src/api/users.rs]
```

Import with:
```bash
dagRobin import tasks.yaml --replace
```

---

## Step 3 — Check What Can Be Done

Before starting ANY work, check what tasks are ready:

```bash
dagRobin ready --format yaml
```

This returns all tasks where:
- Status is `Pending`
- All dependencies have status `Done`

---

## Step 4 — Claim a Task (CRITICAL)

**ALWAYS use `claim` before starting work.**

```bash
dagRobin claim <task-id> --agent your-agent-name
```

Example:
```bash
dagRobin claim feature-auth --agent claudeaude
```

### What happens:

**If task is available (Pending):**
```
Claimed task 'feature-auth' for agent 'claudeaude'
```

**If someone else is working on it (InProgress):**
```
Task 'feature-auth' is already being worked on by 'worker-2'
Do NOT start work on this task!
Exit code: 1
```

**If task is already Done:**
```
Task 'feature-auth' is already DONE
Exit code: 1
```

### Why this matters:

- Prevents two agents from duplicating work
- Shows who's working on what
- Gives you proof that you checked before starting

### The Rule:

**If `claim` returns exit code 1, pick a different task. Do NOT work on it!**

---

## Step 5 — Do the Work

Complete the task. This may include:
- Reading/writing code
- Running tests
- Creating files
- Updating documentation

---

## Step 6 — Mark as Done

When finished:

```bash
dagRobin update <task-id> --status done --metadata "agent=your-agent-name,completed=$(date +%s)"
```

---

## Step 7 — Check Progress

### Monitor overall status:

```bash
# List all tasks
dagRobin list --format table

# Only pending tasks
dagRobin list --status pending

# Only done tasks
dagRobin list --status done

# Only in-progress tasks (see who's working)
dagRobin list --status in_progress
```

### Check blocked tasks:

```bash
dagRobin blocked --format yaml
```

### Visualize dependencies:

```bash
# ASCII graph
dagRobin graph

# Mermaid (for Markdown)
dagRobin graph --format mermaid

# DOT (for Graphviz)
dagRobin graph --format dot --output dag.dot
```

---

## Step 8 — Repeat

```bash
dagRobin ready --format yaml
```

Repeat from Step 4 until no tasks are ready.

---

## Multi-Agent Coordination Pattern

### Orchestrator Agent

The orchestrator creates tasks and monitors progress:

```bash
# Create tasks with dependencies
dagRobin add orch-1 "Setup"
dagRobin add orch-2 "Build backend" --deps orch-1
dagRobin add orch-3 "Build frontend" --deps orch-1
dagRobin add orch-4 "Tests" --deps orch-2,orch-3

# Monitor progress
dagRobin list --format table
dagRobin graph --format mermaid
```

### Worker Agents

Each worker follows this cycle:

```bash
# 1. Check what can be worked on
dagRobin ready --format yaml

# 2. Try to claim a task
dagRobin claim <task-id> --agent worker-1

# If claim fails (exit code 1), pick another task
# REPEAT until claim succeeds

# 3. Do the work
# ... implement feature ...

# 4. Mark as done
dagRobin update <task-id> --status done --metadata "agent=worker-1,completed=$(date +%s)"

# 5. Get next task
dagRobin ready
```

---

## Conflict Prevention

The `claim` command is your tool to prevent conflicts:

```bash
# Always check before starting
dagRobin claim task-id --agent my-agent

# If this fails, someone else is working on it
# Pick a different task instead
```

---

## Export and Import

dagRobin stores everything in a single database file. You can export and import:

```bash
# Export all tasks
dagRobin export tasks.yaml

# Export only pending tasks
dagRobin export pending.yaml --status pending

# Import (merge with existing)
dagRobin import tasks.yaml --merge

# Import (replace everything)
dagRobin import fresh.yaml --replace
```

This makes it easy to:
- Share task lists
- Backup progress
- Onboard new agents

---

## Useful Commands Reference

```bash
dagRobin add <id> <title>              # Add task
dagRobin list                              # List all
dagRobin list --status pending           # Filter by status
dagRobin list --status in_progress      # Who's working
dagRobin ready                           # Ready tasks
dagRobin blocked                         # Blocked tasks
dagRobin claim <id> --agent <name>    # Claim (prevents conflicts!)
dagRobin check <id>                     # Is ready? (exit 0/1)
dagRobin update <id> --status done     # Mark done
dagRobin update <id> --status in_progress --metadata "agent=name"
dagRobin delete <id>                   # Remove task
dagRobin graph --format mermaid        # Visualize
dagRobin export tasks.yaml              # Save to YAML
dagRobin import tasks.yaml --merge      # Load from YAML
dagRobin --db /path/to/db <command>  # Custom database
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success, or task is ready (check command) |
| 1 | Task blocked/already claimed/already done, or error |

Use in scripts:
```bash
if dagRobin claim task-id --agent my-agent; then
  echo "Claimed task-id, starting work..."
else
  echo "Task is already being worked on, picking another..."
fi
```

---

## The Golden Rule

**ALWAYS use `dagRobin claim` before starting work.**

If the claim fails (exit code 1), another agent is working on that task. Pick a different one.

This is the tool that PREVENTS conflicts. Use it!
