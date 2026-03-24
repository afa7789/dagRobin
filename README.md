# dagRobin

DAG-based task manager for autonomous agents.

[![Crates.io](https://img.shields.io/crates/v/dagrobin)](https://crates.io/crates/dag-robin)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](./LICENSE)

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
