# dagRobin

DAG-based task manager for autonomous agents.

[![Crates.io](https://img.shields.io/crates/v/dagrobin)](https://crates.io/crates/dag-robin)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](./LICENSE)

![dagRobin](./resources/dagRobin.png)

## Overview

dagRobin is a CLI tool for managing tasks with native DAG (Directed Acyclic Graph) support. It serves as a single source of truth for autonomous agents and human teams, enabling fast queries, batch updates, and dependency visualization.

## Features

- DAG-based dependencies between tasks
- Automatic detection of tasks ready for execution
- Multiple output formats: Table, JSON, YAML
- Graph visualization: ASCII, DOT, Mermaid
- Fast queries using embedded Sled database
- Atomic operations for data consistency

## Installation

```bash
cargo build --release
cargo install --path .
```

## Quick Start

```bash
# Add tasks
dagRobin add t1 "Setup database" --priority 1
dagRobin add t2 "API implementation" --deps t1 --priority 2

# List and query
dagRobin list
dagRobin ready --format yaml
dagRobin blocked

# Update status
dagRobin update t1 --status done

# Visualize
dagRobin graph --format mermaid
```

## Commands

### add

Create a new task.

```bash
dagRobin add <id> <title> [options]

Options:
  -p, --priority <n>      Priority (1-10, lower is higher)
  -d, --deps <ids>        Dependency task IDs
  -t, --tags <tags>       Comma-separated tags
  --files <paths>          Affected file paths
  --description <text>     Task description
```

### list

List tasks with optional filters.

```bash
dagRobin list [options]

Options:
  --status <status>        Filter by status (pending, in_progress, done, blocked)
  --priority-min <n>       Minimum priority
  -t, --tags <tags>       Filter by tags
  -f, --format <format>   Output format: table, json, yaml
```

### ready

List tasks with all dependencies resolved.

```bash
dagRobin ready [options]

Options:
  --priority-min <n>       Minimum priority
  -f, --format <format>   Output format: table, json, yaml
```

### blocked

List tasks blocked by incomplete dependencies.

```bash
dagRobin blocked [-f, --format <format>]
```

### check

Check if a task is ready (exit code 0 if ready, 1 if blocked).

```bash
dagRobin check <id>
```

### update

Update task fields.

```bash
dagRobin update <id> [options]

Options:
  -s, --status <status>   New status
  -t, --title <text>      New title
  --description <text>    New description
  --metadata <k:v>        Add metadata key-value pair
```

### delete

Delete a task.

```bash
dagRobin delete <id> [-f, --force]
```

### graph

Generate dependency graph visualization.

```bash
dagRobin graph [options]

Options:
  -f, --format <format>   Graph format: ascii, dot, mermaid
  -o, --output <file>    Write to file instead of stdout
```

### import / export

Import or export tasks in YAML format.

```bash
dagRobin export <file> [options]
dagRobin import <file> [options]

Options:
  --status <status>        Filter by status
  --tags <tags>           Filter by tags
  --merge                  Import: merge with existing (default)
  --replace                Import: replace all existing
```

## Configuration

Default database location is `dagrobin.db` in the current directory.

```bash
dagRobin --db /path/to/database add t1 "Task"
```

## Architecture

```
src/
  task.rs    - Task struct and TaskStatus enum
  db.rs      - Sled database with indices
  main.rs    - CLI commands (clap)
```

## AI Agent Integration Guide

This section explains how autonomous agents (Claude, OpenAI, etc.) can effectively use dagRobin to coordinate work.

### Task ID Naming Convention

Use clear, descriptive IDs with a consistent format:

```
[type]-[number]   # Examples: setup-1, api-1, test-2, deploy-3
feature-[name]    # Examples: feature-auth, feature-payments
fix-[issue]       # Examples: fix-login, fix-memory-leak
```

Avoid generic IDs like `t1`, `task1` - they become confusing at scale.

### Recommended Workflow for Agents

#### 1. At Session Start: Check Ready Tasks

```bash
dagRobin ready --format yaml
```

This returns all tasks ready to work on (all dependencies resolved).

#### 2. Pick One Task to Work On

When starting a task, mark it as in_progress and record who is working on it:

```bash
dagRobin update setup-1 --status in_progress --metadata "agent=claude,started_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

#### 3. After Completing: Mark Done and Add Metadata

```bash
dagRobin update setup-1 --status done --metadata "completed_at=$(date -u +%Y-%m-%dT%H:%M:%SZ),agent=claude"
```

#### 4. View Current Progress

```bash
dagRobin list --format table
dagRobin graph --format mermaid
```

### Task Creation Template

When adding tasks, include all relevant information:

```bash
dagRobin add feature-auth "Implement user authentication" \
  --priority 1 \
  --deps setup-db \
  --tags "backend,security" \
  --files "src/auth.rs,src/middleware.rs" \
  --description "Implement JWT-based authentication with refresh tokens"
```

### YAML Format for Batch Operations

For importing multiple tasks, use this YAML format:

```yaml
- id: setup-1
  title: Initialize project structure
  priority: 1
  tags: [setup]
  deps: []

- id: api-users
  title: Create user API endpoints
  priority: 2
  tags: [backend,api]
  deps: [setup-1]
  files: [src/api/users.rs]
  description: CRUD operations for users

- id: api-posts
  title: Create posts API endpoints
  priority: 2
  tags: [backend,api]
  deps: [setup-1]
  files: [src/api/posts.rs]
  description: CRUD operations for posts

- id: tests
  title: Write integration tests
  priority: 3
  tags: [testing]
  deps: [api-users,api-posts]
```

Import with:

```bash
dagRobin import tasks.yaml --replace
```

### Best Practices for Agent Coordination

1. **Always check ready before starting**: Never assume a task is ready. Always run `dagRobin ready` or `dagRobin check <id>`.

2. **One agent per task**: When an agent starts working on a task, mark it `in_progress` with agent metadata to prevent duplicate work.

3. **Use atomic commits**: Each agent should complete one task fully (status=done) before moving to the next.

4. **Add context to metadata**: Record agent name, time spent, and any notes.

```bash
# Agent starting work
dagRobin update task-1 --status in_progress --metadata "agent=claudeaude,started=$(date +%s)"

# Agent completing work
dagRobin update task-1 --status done --metadata "agent=claudeaude,completed=$(date +%s),notes=Required refactoring of auth module"
```

5. **Use tags for filtering**: Tags help agents find relevant tasks.

```bash
# Find backend tasks
dagRobin list --tags backend --status pending

# Find tasks affecting specific files
dagRobin list --files src/database.rs
```

### Claude Code Integration Example

Add this to your CLAUDE.md:

```markdown
# Project Task Management

This project uses dagRobin for task coordination.

## Key Commands
- `dagRobin ready` - List tasks ready to work on
- `dagRobin list` - View all tasks
- `dagRobin graph --format mermaid` - Visualize dependencies

## Workflow
1. Run `dagRobin ready` at session start
2. Pick one task, mark as in_progress with your agent name
3. Complete the task, mark as done
4. Repeat

## Example Session
dagRobin update auth-1 --status in_progress --metadata "agent=claudeaude"
# ... do work ...
dagRobin update auth-1 --status done --metadata "agent=claudeaude,completed=$(date +%s)"
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success or task is ready (check command) |
| 1 | Task is blocked (check command) or error |

Use exit codes to script agent behavior:

```bash
if dagRobin check $TASK_ID; then
  echo "Task $TASK_ID is ready to work"
else
  echo "Task $TASK_ID is blocked"
fi
```

## Testing

```bash
cargo test
```

## Dependencies

| Component | Choice | Reason |
|-----------|--------|--------|
| Language | Rust | Performance, safety, static binary |
| Database | Sled | Pure Rust, ACID, embedded |
| CLI | clap | Derive macros, automatic validation |
| Serialization | serde | YAML/JSON interoperability |

## License

Licensed under the MIT OR Apache-2.0 license.
