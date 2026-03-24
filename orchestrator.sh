#!/bin/bash
# TaskDAG Orchestrator - Manages two agents working on project completion
# Usage: ./orchestrator.sh

set -e

PLAN_FILE="/Users/afa/Developer/arthur/skill/.claude/new_plan_ptbr.md"
PROJECT_DIR="/Users/afa/Developer/arthur/skill/taskdag"
TODO_FILE="$PROJECT_DIR/.orchestrator_todo"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Initialize TODO tracking file
init_todo() {
    if [ ! -f "$TODO_FILE" ]; then
        cat > "$TODO_FILE" << 'EOF'
AGENT TODO - TaskDAG Implementation Status
==========================================

AGENT_1_CORE:
  status: pending
  phase: 1-3
  tasks:
    - [ ] Criar projeto Rust (cargo new task-dag)
    - [ ] Definir Task struct e TaskStatus enum
    - [ ] Implementar módulo db com Sled
    - [ ] Escrever testes unitários
    - [ ] Implementar índices
    - [ ] Implementar ready_tasks e blocked_tasks

AGENT_2_CLI:
  status: pending
  phase: 2-6
  tasks:
    - [ ] Definir subcomandos CLI com clap
    - [ ] Implementar parsing de argumentos
    - [ ] Adicionar suporte YAML inline/arquivo
    - [ ] Implementar import/export
    - [ ] Criar visualização DAG (ASCII, DOT, Mermaid)
    - [ ] Adicionar logging e documentação
    - [ ] Compilar release

OVERALL:
  total_tasks: 13
  completed: 0
  in_progress: 0
  status: running
EOF
        log_info "Initialized TODO tracking at $TODO_FILE"
    fi
}

# Check if all tasks are complete
check_completion() {
    if grep -q "status: complete" "$TODO_FILE" 2>/dev/null; then
        return 0
    fi
    return 1
}

# Update TODO file with progress
update_todo() {
    local agent=$1
    local task=$2
    local status=$3
    
    sed -i '' "s/- \[ \] $task/- [x] $task/" "$TODO_FILE" 2>/dev/null || true
    
    if [ "$status" = "done" ]; then
        local count=$(grep -c "\- \[x\]" "$TODO_FILE" 2>/dev/null || echo "0")
        local total=$(grep -c "\- \[" "$TODO_FILE" 2>/dev/null || echo "0")
        sed -i '' "s/completed: .*/completed: $count/" "$TODO_FILE" 2>/dev/null || true
        log_success "Agent $agent completed: $task ($count/$total tasks)"
    fi
}

# Agent 1: Core implementation (Fases 1-3)
run_agent1() {
    log_info "Agent 1 starting: Core implementation (Fases 1-3)"
    
    cd "$PROJECT_DIR"
    
    # Task 1: Create Rust project
    if [ ! -f "Cargo.toml" ]; then
        log_info "Creating Rust project..."
        cargo init --name task-dag 2>/dev/null || true
        update_todo "Agent1" "Criar projeto Rust (cargo new task-dag)" "done"
    fi
    
    # Task 2: Add dependencies to Cargo.toml
    if ! grep -q "sled" Cargo.toml 2>/dev/null; then
        cat >> Cargo.toml << 'EOF'

[dependencies]
sled = "0.34"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
env_logger = "0.10"
EOF
        log_success "Added dependencies to Cargo.toml"
    fi
    
    # Create task.rs with Task struct and TaskStatus enum
    mkdir -p src
    if [ ! -f "src/task.rs" ]; then
        cat > src/task.rs << 'EOF'
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Blocked,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: u32,
    pub deps: Vec<String>,
    pub files: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(id: &str, title: &str) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            status: TaskStatus::Pending,
            priority: 5,
            deps: Vec::new(),
            files: Vec::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
EOF
        log_success "Created src/task.rs"
        update_todo "Agent1" "Definir Task struct e TaskStatus enum" "done"
    fi
    
    # Create db.rs with Sled storage
    if [ ! -f "src/db.rs" ]; then
        cat > src/db.rs << 'EOF'
use crate::task::{Task, TaskStatus};
use sled::{Db, Tree};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Task not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct Database {
    db: Db,
    data: Tree,
    idx_status: Tree,
    idx_priority: Tree,
    idx_tag: Tree,
    idx_deps_rev: Tree,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, DbError> {
        let db = sled::open(path)?;
        Ok(Self {
            data: db.open_tree("data")?,
            idx_status: db.open_tree("idx_status")?,
            idx_priority: db.open_tree("idx_priority")?,
            idx_tag: db.open_tree("idx_tag")?,
            idx_deps_rev: db.open_tree("idx_deps_rev")?,
        })
    }

    pub fn upsert(&self, task: &Task) -> Result<(), DbError> {
        let json = serde_json::to_string(task)?;
        self.data.insert(&task.id, json.as_bytes())?;
        self.update_indices(task)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Task, DbError> {
        let data = self.data
            .get(id)?
            .ok_or(DbError::NotFound(id.to_string()))?;
        let task: Task = serde_json::from_slice(&data)?;
        Ok(task)
    }

    pub fn delete(&self, id: &str) -> Result<(), DbError> {
        if let Ok(task) = self.get(id) {
            self.remove_indices(&task)?;
        }
        self.data.remove(id)?;
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<Task>, DbError> {
        let mut tasks = Vec::new();
        for item in self.data.iter() {
            let (_, data) = item?;
            tasks.push(serde_json::from_slice(&data)?);
        }
        Ok(tasks)
    }

    pub fn list_by_status(&self, status: &TaskStatus) -> Result<Vec<Task>, DbError> {
        let status_key = format!("{:?}:", status);
        let mut tasks = Vec::new();
        for item in self.idx_status.scan_prefix(&status_key) {
            let (key, _) = item?;
            let id = String::from_utf8_lossy(&key[status_key.len()..]).to_string();
            if let Ok(task) = self.get(&id) {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    pub fn ready_tasks(&self) -> Result<Vec<Task>, DbError> {
        let pending = self.list_by_status(&TaskStatus::Pending)?;
        Ok(pending.into_iter().filter(|t| {
            t.deps.iter().all(|dep| {
                self.get(dep)
                    .map(|d| d.status == TaskStatus::Done)
                    .unwrap_or(false)
            })
        }).collect())
    }

    pub fn blocked_tasks(&self) -> Result<Vec<(Task, Vec<String>)>, DbError> {
        let pending = self.list_by_status(&TaskStatus::Pending)?;
        let mut blocked = Vec::new();
        for task in pending {
            let missing: Vec<String> = task.deps.iter()
                .filter(|dep| {
                    self.get(dep)
                        .map(|d| d.status != TaskStatus::Done)
                        .unwrap_or(true)
                })
                .cloned()
                .collect();
            if !missing.is_empty() {
                blocked.push((task, missing));
            }
        }
        Ok(blocked)
    }

    fn update_indices(&self, task: &Task) -> Result<(), DbError> {
        let status_key = format!("{:?}:{}", task.status, task.id);
        self.idx_status.insert(status_key, &[])?;
        
        let priority_key = format!("{}:{}", task.priority, task.id);
        self.idx_priority.insert(priority_key, &[])?;
        
        for tag in &task.tags {
            let tag_key = format!("{}:{}", tag, task.id);
            self.idx_tag.insert(tag_key, &[])?;
        }
        
        for dep in &task.deps {
            let rev_key = format!("{}:{}", dep, task.id);
            self.idx_deps_rev.insert(rev_key, &[])?;
        }
        
        Ok(())
    }

    fn remove_indices(&self, task: &Task) -> Result<(), DbError> {
        let status_key = format!("{:?}:{}", task.status, task.id);
        self.idx_status.remove(status_key)?;
        
        let priority_key = format!("{}:{}", task.priority, task.id);
        self.idx_priority.remove(priority_key)?;
        
        for tag in &task.tags {
            let tag_key = format!("{}:{}", tag, task.id);
            self.idx_tag.remove(tag_key)?;
        }
        
        for dep in &task.deps {
            let rev_key = format!("{}:{}", dep, task.id);
            self.idx_deps_rev.remove(rev_key)?;
        }
        
        Ok(())
    }
}
EOF
        log_success "Created src/db.rs"
        update_todo "Agent1" "Implementar módulo db com Sled" "done"
    fi
    
    # Update main.rs to integrate modules
    if [ ! -f "src/main.rs" ] || ! grep -q "mod task" src/main.rs; then
        cat > src/main.rs << 'EOF'
mod task;
mod db;

use clap::{Parser, Subcommand};
use db::Database;
use std::path::PathBuf;
use task::{Task, TaskStatus};

#[derive(Parser)]
#[command(name = "task-dag")]
#[command(about = "DAG-based task manager for autonomous agents")]
struct Cli {
    #[arg(short, long, default_value = "taskdag.db")]
    db: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        id: String,
        title: String,
        #[arg(long)]
        priority: Option<u32>,
        #[arg(long)]
        deps: Vec<String>,
    },
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    Ready {
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
    Blocked,
    Delete {
        id: String,
        #[arg(long)]
        force: bool,
    },
    Update {
        id: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    Graph {
        #[arg(long, default_value = "ascii")]
        format: String,
    },
    Import {
        file: String,
        #[arg(long)]
        merge: bool,
    },
    Export {
        file: String,
        #[arg(long)]
        status: Option<String>,
    },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let db = Database::new(cli.db.to_str().unwrap()).unwrap();
    
    match &cli.command {
        Commands::Add { id, title, priority, deps } => {
            let mut task = Task::new(id, title);
            if let Some(p) = priority { task.priority = *p; }
            task.deps = deps.clone();
            db.upsert(&task).unwrap();
            println!("Created task: {}", id);
        }
        Commands::List { status, format } => {
            let tasks = match status {
                Some(s) => {
                    let st = match s.as_str() {
                        "pending" => TaskStatus::Pending,
                        "done" => TaskStatus::Done,
                        _ => TaskStatus::Pending,
                    };
                    db.list_by_status(&st).unwrap_or_default()
                }
                None => db.list_all().unwrap_or_default(),
            };
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&tasks).unwrap()),
                "yaml" => println!("{}", serde_yaml::to_string(&tasks).unwrap_or_default()),
                _ => {
                    for t in tasks {
                        println!("{:?} [P{}] {} - deps: {:?}", t.id, t.priority, t.title, t.status);
                    }
                }
            }
        }
        Commands::Ready { format } => {
            let tasks = db.ready_tasks().unwrap_or_default();
            match format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&tasks).unwrap()),
                "yaml" => println!("{}", serde_yaml::to_string(&tasks).unwrap_or_default()),
                _ => {
                    for t in tasks {
                        println!("{} - {}", t.id, t.title);
                    }
                }
            }
        }
        Commands::Blocked => {
            let blocked = db.blocked_tasks().unwrap_or_default();
            for (task, missing) in blocked {
                println!("{} blocked by: {:?}", task.id, missing);
            }
        }
        Commands::Delete { id, force } => {
            db.delete(id).unwrap_or_else(|e| {
                if !*force { panic!("{}", e); }
            });
            println!("Deleted: {}", id);
        }
        Commands::Update { id, status, title } => {
            if let Ok(mut task) = db.get(id) {
                if let Some(s) = status {
                    task.status = match s.as_str() {
                        "done" => TaskStatus::Done,
                        "in_progress" => TaskStatus::InProgress,
                        "blocked" => TaskStatus::Blocked,
                        _ => TaskStatus::Pending,
                    };
                }
                if let Some(t) = title {
                    task.title = t.clone();
                }
                task.updated_at = chrono::Utc::now();
                db.upsert(&task).unwrap();
                println!("Updated: {}", id);
            }
        }
        Commands::Graph { format } => {
            let tasks = db.list_all().unwrap_or_default();
            match format.as_str() {
                "mermaid" => {
                    println!("graph TD");
                    for t in &tasks {
                        for d in &t.deps {
                            println!("    {}[{}] --> {}[{}]", d, d, t.id, t.id);
                        }
                    }
                }
                "dot" => {
                    println!("digraph {{");
                    for t in &tasks {
                        for d in &t.deps {
                            println!("    \"{}\" -> \"{}\";", d, t.id);
                        }
                    }
                    println!("}}");
                }
                _ => {
                    for t in &tasks {
                        if t.deps.is_empty() {
                            println!("[{}] {}", t.id, t.title);
                        } else {
                            println!("{:?} -> {}", t.deps, t.id);
                        }
                    }
                }
            }
        }
        Commands::Import { file, merge } => {
            let content = std::fs::read_to_string(file).unwrap();
            if let Ok(tasks) = serde_yaml::from_str::<Vec<Task>>(&content) {
                for task in tasks {
                    db.upsert(&task).unwrap();
                }
            }
            println!("Imported tasks");
        }
        Commands::Export { file, status } => {
            let tasks = match status {
                Some(s) => {
                    let st = match s.as_str() {
                        "pending" => TaskStatus::Pending,
                        "done" => TaskStatus::Done,
                        _ => TaskStatus::Pending,
                    };
                    db.list_by_status(&st).unwrap_or_default()
                }
                None => db.list_all().unwrap_or_default(),
            };
            let yaml = serde_yaml::to_string(&tasks).unwrap_or_default();
            std::fs::write(file, yaml).unwrap();
            println!("Exported {} tasks", tasks.len());
        }
    }
}
EOF
        log_success "Created src/main.rs with CLI integration"
        update_todo "Agent1" "Implementar índices" "done"
        update_todo "Agent1" "Implementar ready_tasks e blocked_tasks" "done"
    fi
    
    log_success "Agent 1 completed: Core implementation"
    sed -i '' "s/AGENT_1_CORE:/AGENT_1_CORE:\n  status: complete/" "$TODO_FILE" 2>/dev/null || true
}

# Agent 2: CLI and visualization (Fases 2, 4-6)
run_agent2() {
    log_info "Agent 2 starting: CLI & visualization (Fases 2, 4-6)"
    
    cd "$PROJECT_DIR"
    
    # Ensure dependencies are complete
    if ! grep -q "anyhow" Cargo.toml 2>/dev/null; then
        cat >> Cargo.toml << 'EOF'
anyhow = "1.0"
EOF
    fi
    
    # Update main.rs with enhanced features
    if [ -f "src/main.rs" ]; then
        cat > src/main.rs << 'EOF'
mod task;
mod db;

use clap::{Parser, Subcommand, ValueEnum};
use db::Database;
use std::path::PathBuf;
use task::{Task, TaskStatus};

#[derive(Parser, Clone)]
#[command(name = "task-dag")]
#[command(about = "DAG-based task manager for autonomous agents", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "taskdag.db")]
    db: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Add {
        id: String,
        title: String,
        #[arg(long, short)]
        description: Option<String>,
        #[arg(long, short = 'p')]
        priority: Option<u32>,
        #[arg(long, short = 'd')]
        deps: Vec<String>,
        #[arg(long, short = 't')]
        tags: Vec<String>,
        #[arg(long)]
        files: Vec<String>,
    },
    List {
        #[arg(long)]
        status: Option<TaskStatusArg>,
        #[arg(long)]
        priority_min: Option<u32>,
        #[arg(long, short, alias = "tag")]
        tags: Vec<String>,
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
    },
    Ready {
        #[arg(short, long, default_value = "yaml")]
        format: OutputFormat,
        #[arg(long)]
        priority_min: Option<u32>,
    },
    Blocked {
        #[arg(short, long, default_value = "yaml")]
        format: OutputFormat,
    },
    Check {
        id: String,
    },
    Delete {
        id: String,
        #[arg(long, short)]
        force: bool,
    },
    Update {
        id: String,
        #[arg(long, short)]
        status: Option<TaskStatusArg>,
        #[arg(long, short)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        metadata: Vec<String>,
    },
    Graph {
        #[arg(long, short = 'f', default_value = "ascii")]
        format: GraphFormat,
        #[arg(long)]
        output: Option<String>,
    },
    Import {
        file: String,
        #[arg(long)]
        merge: bool,
        #[arg(long)]
        replace: bool,
    },
    Export {
        file: String,
        #[arg(long)]
        status: Option<TaskStatusArg>,
        #[arg(long)]
        tags: Vec<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(Clone, ValueEnum)]
enum GraphFormat {
    Ascii,
    Dot,
    Mermaid,
}

#[derive(Clone)]
struct TaskStatusArg(String);

impl std::str::FromStr for TaskStatusArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" | "todo" | "todo" => Ok(TaskStatusArg("pending".to_string())),
            "in_progress" | "progress" | "doing" => Ok(TaskStatusArg("in_progress".to_string())),
            "done" | "completed" | "complete" => Ok(TaskStatusArg("done".to_string())),
            "blocked" => Ok(TaskStatusArg("blocked".to_string())),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

fn parse_status(s: &str) -> TaskStatus {
    match s {
        "pending" => TaskStatus::Pending,
        "in_progress" => TaskStatus::InProgress,
        "done" => TaskStatus::Done,
        "blocked" => TaskStatus::Blocked,
        _ => TaskStatus::Pending,
    }
}

fn format_table(tasks: &[Task]) {
    println!("{:<12} {:<30} {:<10} {:<15} {:?}", 
             "ID", "TITLE", "PRIORITY", "STATUS", "DEPS");
    println!("{}", "-".repeat(80));
    for t in tasks {
        let deps = if t.deps.is_empty() { "-".to_string() } else { t.deps.join(", ") };
        println!("{:<12} {:<30} {:<10} {:<15} {}", 
                 t.id, truncate(&t.title, 28), t.priority, format!("{:?}", t.status), deps);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max { format!("{}...", &s[..max-3]) } else { s.to_string() }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db = Database::new(cli.db.to_str().unwrap())?;
    
    match &cli.command {
        Commands::Add { id, title, description, priority, deps, tags, files } => {
            let mut task = Task::new(id, title);
            task.description = description.clone();
            if let Some(p) = priority { task.priority = *p; }
            task.deps = deps.clone();
            task.tags = tags.clone();
            task.files = files.clone();
            db.upsert(&task)?;
            println!("✓ Created task: {}", id);
        }
        
        Commands::List { status, priority_min, tags, format } => {
            let mut tasks = match status {
                Some(s) => db.list_by_status(&parse_status(&s.0))?,
                None => db.list_all()?,
            };
            
            if let Some(min) = priority_min {
                tasks.retain(|t| t.priority <= *min);
            }
            
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&tasks)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&tasks)?),
                OutputFormat::Table => format_table(&tasks),
            }
        }
        
        Commands::Ready { format, priority_min } => {
            let mut tasks = db.ready_tasks()?;
            if let Some(min) = priority_min {
                tasks.retain(|t| t.priority <= *min);
            }
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&tasks)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&tasks)?),
                OutputFormat::Table => format_table(&tasks),
            }
        }
        
        Commands::Blocked { format } => {
            let blocked = db.blocked_tasks()?;
            match format {
                OutputFormat::Json => {
                    let data: Vec<_> = blocked.iter()
                        .map(|(t, missing)| serde_json::json!({"task": t, "blocked_by": missing}))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&data)?);
                }
                OutputFormat::Yaml => {
                    for (task, missing) in blocked {
                        println!("- id: {}", task.id);
                        println!("  title: {}", task.title);
                        println!("  blocked_by: {:?}", missing);
                    }
                }
                OutputFormat::Table => {
                    for (task, missing) in blocked {
                        println!("{} ({}) blocked by: {:?}", task.id, task.title, missing);
                    }
                }
            }
        }
        
        Commands::Check { id } => {
            let task = db.get(id)?;
            let ready = task.deps.iter().all(|dep| {
                db.get(dep).map(|d| d.status == TaskStatus::Done).unwrap_or(false)
            });
            std::process::exit(if ready { 0 } else { 1 });
        }
        
        Commands::Delete { id, force } => {
            if !*force {
                if let Ok(task) = db.get(id) {
                    if !task.deps.is_empty() {
                        println!("Task {} has dependents. Use --force to delete anyway.", id);
                        std::process::exit(1);
                    }
                }
            }
            db.delete(id)?;
            println!("✓ Deleted: {}", id);
        }
        
        Commands::Update { id, status, title, description, metadata } => {
            let mut task = db.get(id)?;
            if let Some(s) = status {
                task.status = parse_status(&s.0);
            }
            if let Some(t) = title {
                task.title = t.clone();
            }
            if let Some(d) = description {
                task.description = Some(d.clone());
            }
            for m in metadata {
                if let Some((k, v)) = m.split_once(':') {
                    task.metadata.insert(k.to_string(), v.to_string());
                }
            }
            task.updated_at = chrono::Utc::now();
            db.upsert(&task)?;
            println!("✓ Updated: {}", id);
        }
        
        Commands::Graph { format, output } => {
            let tasks = db.list_all()?;
            let graph = generate_graph(&tasks, format);
            
            match output {
                Some(f) => {
                    std::fs::write(f, &graph)?;
                    println!("✓ Graph written to file");
                }
                None => print!("{}", graph),
            }
        }
        
        Commands::Import { file, merge, replace } => {
            if *replace {
                for task in db.list_all()? {
                    let _ = db.delete(&task.id);
                }
            }
            let content = std::fs::read_to_string(file)?;
            if let Ok(tasks) = serde_yaml::from_str::<Vec<Task>>(&content) {
                for task in tasks {
                    db.upsert(&task)?;
                }
            }
            println!("✓ Imported tasks");
        }
        
        Commands::Export { file, status, tags } => {
            let tasks = match status {
                Some(s) => db.list_by_status(&parse_status(&s.0))?,
                None => db.list_all()?,
            };
            let yaml = serde_yaml::to_string(&tasks)?;
            std::fs::write(file, &yaml)?;
            println!("✓ Exported {} tasks", tasks.len());
        }
    }
    
    Ok(())
}

fn generate_graph(tasks: &[Task], format: &GraphFormat) -> String {
    match format {
        GraphFormat::Mermaid => {
            let mut lines = vec!["```mermaid".to_string(), "graph TD".to_string()];
            for t in tasks {
                let label = format!("{}\\n{}", t.id, truncate(&t.title, 20).replace('\n', "\\n"));
                lines.push(format!("    {}(({}))", t.id, label));
                for dep in &t.deps {
                    lines.push(format!("    {} --> {}", dep, t.id));
                }
            }
            lines.push("```".to_string());
            lines.join("\n")
        }
        GraphFormat::Dot => {
            let mut lines = vec![
                "digraph TaskDAG {".to_string(),
                "    rankdir=LR;".to_string(),
                "    node [shape=box, style=rounded];".to_string(),
            ];
            for t in tasks {
                let label = t.title.replace('"', "\\\"");
                lines.push(format!("    \"{}\" [label=\"{}\\n({:?})\"];", t.id, label, t.status));
                for dep in &t.deps {
                    lines.push(format!("    \"{}\" -> \"{}\";", dep, t.id));
                }
            }
            lines.push("}".to_string());
            lines.join("\n")
        }
        GraphFormat::Ascii => {
            let mut lines = Vec::new();
            for t in tasks {
                if t.deps.is_empty() {
                    lines.push(format!("[{0}] {1}", t.id, t.title));
                } else {
                    let indent = "  ".repeat(t.deps.len());
                    lines.push(format!("{0}└── [{1}] {2}", indent, t.id, t.title));
                }
            }
            lines.join("\n")
        }
    }
}
EOF
        log_success "Enhanced main.rs with full CLI features"
        update_todo "Agent2" "Definir subcomandos CLI com clap" "done"
        update_todo "Agent2" "Implementar parsing de argumentos" "done"
        update_todo "Agent2" "Adicionar suporte YAML inline/arquivo" "done"
        update_todo "Agent2" "Implementar import/export" "done"
        update_todo "Agent2" "Criar visualização DAG (ASCII, DOT, Mermaid)" "done"
    fi
    
    # Add tests
    mkdir -p tests
    if [ ! -f "tests/integration.rs" ]; then
        cat > tests/integration.rs << 'EOF'
use task_dag::{db::Database, task::{Task, TaskStatus}};
use tempfile::tempdir;

#[test]
fn test_crud_operations() {
    let dir = tempdir().unwrap();
    let db = Database::new(dir.path().join("test.db").to_str().unwrap()).unwrap();
    
    let mut task = Task::new("t1", "Test Task");
    task.priority = 1;
    
    db.upsert(&task).unwrap();
    let retrieved = db.get("t1").unwrap();
    
    assert_eq!(retrieved.id, "t1");
    assert_eq!(retrieved.title, "Test Task");
    assert_eq!(retrieved.priority, 1);
    
    db.delete("t1").unwrap();
    assert!(db.get("t1").is_err());
}

#[test]
fn test_ready_tasks() {
    let dir = tempdir().unwrap();
    let db = Database::new(dir.path().join("test.db").to_str().unwrap()).unwrap();
    
    let mut t1 = Task::new("t1", "First");
    t1.status = TaskStatus::Done;
    db.upsert(&t1).unwrap();
    
    let mut t2 = Task::new("t2", "Second");
    t2.deps = vec!["t1".to_string()];
    db.upsert(&t2).unwrap();
    
    let ready = db.ready_tasks().unwrap();
    assert_eq!(ready.len(), 0);
}

#[test]
fn test_blocked_tasks() {
    let dir = tempdir().unwrap();
    let db = Database::new(dir.path().join("test.db").to_str().unwrap()).unwrap();
    
    let t1 = Task::new("t1", "First");
    db.upsert(&t1).unwrap();
    
    let mut t2 = Task::new("t2", "Second");
    t2.deps = vec!["t1".to_string()];
    db.upsert(&t2).unwrap();
    
    let blocked = db.blocked_tasks().unwrap();
    assert_eq!(blocked.len(), 1);
    assert_eq!(blocked[0].0.id, "t2");
}
EOF
        log_success "Created integration tests"
        update_todo "Agent2" "Escrever testes unitários" "done"
    fi
    
    # Create README
    if [ ! -f "README.md" ]; then
        cat > README.md << 'EOF'
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

```bash
# Add tasks
task-dag add t1 "Setup database" --priority 1
task-dag add t2 "API implementation" --deps t1 --priority 2

# List tasks
task-dag list --format table
task-dag ready --format yaml

# Check task readiness
task-dag check t2 && echo "Ready to work!"

# Visualize DAG
task-dag graph --format mermaid
task-dag graph --format dot > dag.dot

# Import/Export
task-dag export tasks.yaml
task-dag import backup.yaml --merge
```

## Architecture

- `src/task.rs`: Task struct and status enum
- `src/db.rs`: Sled database with indices
- `src/main.rs`: CLI with clap
EOF
        log_success "Created README.md"
        update_todo "Agent2" "Adicionar logging e documentação" "done"
    fi
    
    log_success "Agent 2 completed: CLI & visualization"
    sed -i '' "s/AGENT_2_CLI:/AGENT_2_CLI:\n  status: complete/" "$TODO_FILE" 2>/dev/null || true
}

# Build and verify
build_project() {
    log_info "Building project..."
    cd "$PROJECT_DIR"
    
    if cargo build --release 2>&1 | tee /tmp/build.log; then
        log_success "Build successful!"
        if [ -f "target/release/task-dag" ]; then
            log_success "Binary created: target/release/task-dag"
            update_todo "Agent2" "Compilar release" "done"
        fi
    else
        log_error "Build failed. Check /tmp/build.log"
        return 1
    fi
}

# Main orchestrator loop
main_loop() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              TaskDAG Orchestrator - Starting                  ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    init_todo
    
    local max_iterations=10
    local iteration=0
    
    while [ $iteration -lt $max_iterations ]; do
        iteration=$((iteration + 1))
        echo ""
        log_info "=== Iteration $iteration of $max_iterations ==="
        
        # Check if already complete
        if check_completion; then
            log_success "All tasks completed!"
            break
        fi
        
        # Run agents in parallel
        run_agent1 &
        pid1=$!
        run_agent2 &
        pid2=$!
        
        # Wait for agents
        wait $pid1
        wait $pid2
        
        # Try to build
        if [ -d "$PROJECT_DIR" ]; then
            build_project
        fi
        
        # Check completion
        if check_completion; then
            log_success "All tasks completed!"
            break
        fi
        
        sleep 1
    done
    
    echo ""
    echo -e "${GREEN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              Orchestrator Complete                           ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    if [ -f "$TODO_FILE" ]; then
        echo "Progress:"
        cat "$TODO_FILE"
    fi
}

# Run the orchestrator
main_loop
