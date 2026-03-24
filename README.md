# TaskDAG

DAG-based task manager for autonomous agents.

## Features

- **DAG-based dependencies**: Tasks can depend on other tasks
- **Ready task detection**: Automatically find tasks ready to execute
- **Multiple output formats**: Table, JSON, YAML
- **Graph visualization**: ASCII, DOT, Mermaid
- **Fast queries**: <5ms using Sled embedded database

## Install

```bash
cargo build --release
cargo install --path .
```

## Usage

### Add Tasks

```bash
# Simple task
./target/release/taskdag add t1 "Setup database" --priority 1

# Task with dependencies
./target/release/taskdag add t2 "API implementation" --deps t1 --priority 2

# Task with tags and files
./target/release/taskdag add t3 "Tests" --tags "testing" --files "src/api.rs" --priority 3
```

### List Tasks

```bash
# List all tasks in table format
./target/release/taskdag list

# List only pending tasks
./target/release/taskdag list --status pending

# List as JSON
./target/release/taskdag list --format json

# List as YAML
./target/release/taskdag list --format yaml

# Filter by priority
./target/release/taskdag list --priority-min 2
```

### Ready Tasks (tasks with all dependencies done)

```bash
./target/release/taskdag ready
./target/release/taskdag ready --format yaml
./target/release/taskdag ready --priority-min 1
```

### Blocked Tasks

```bash
./target/release/taskdag blocked
./target/release/taskdag blocked --format json
```

### Check Task Readiness

```bash
./target/release/taskdag check t2 && echo "Ready to work!" || echo "Still blocked"
```

### Update Tasks

```bash
# Mark as done
./target/release/taskdag update t1 --status done

# Update title
./target/release/taskdag update t1 --title "Database Setup Complete"

# Add metadata
./target/release/taskdag update t1 --metadata "completed_by=agent"
```

### Delete Tasks

```bash
./target/release/taskdag delete t1
./target/release/taskdag delete t1 --force  # Force delete even with dependents
```

### Visualize DAG

```bash
# ASCII format
./target/release/taskdag graph

# Mermaid format (for Markdown)
./target/release/taskdag graph --format mermaid

# DOT format (for Graphviz)
./target/release/taskdag graph --format dot > dag.dot

# Export to file
./target/release/taskdag graph --format mermaid --output dag.md
```

### Import/Export

```bash
# Export all tasks
./target/release/taskdag export tasks.yaml

# Export only pending tasks
./target/release/taskdag export pending.yaml --status pending

# Import (merge mode - keeps existing)
./target/release/taskdag import backup.yaml --merge

# Import (replace mode - clears existing)
./target/release/taskdag import backup.yaml --replace
```

## Architecture

- `src/task.rs`: Task struct and status enum
- `src/db.rs`: Sled database with indices
- `src/main.rs`: CLI with clap

## Running Tests

```bash
cargo test
```

## Database Location

Default database is `taskdag.db` in current directory. Override with:

```bash
./target/release/taskdag --db /path/to/db add t1 "Task"
```
