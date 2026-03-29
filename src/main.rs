use clap::{Parser, Subcommand, ValueEnum};
use dagrobin::db::Database;
use dagrobin::error::{DagRobinError, Result};
use dagrobin::task::{Task, TaskStatus};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Clone)]
#[command(
    name = "dagRobin",
    about = "DAG-based task manager for AI agents",
    long_about = None,
    version = VERSION
)]
struct Cli {
    /// Database path. Defaults to ~/.local/share/dagRobin/dagrobin.db
    #[arg(short, long)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

fn default_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("dagRobin")
        .join("dagrobin.db")
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
        #[arg(long)]
        deps: Vec<String>,
        #[arg(long, short = 't')]
        tags: Vec<String>,
        #[arg(long)]
        files: Vec<String>,
    },
    Get {
        id: String,
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
    Claim {
        id: String,
        #[arg(long, short)]
        agent: Option<String>,
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
        /// Metadata as key:value pairs. Use semicolon (;) to separate multiple pairs, comma (,) allowed in values
        /// Examples: --metadata "notes:foo,bar;tags:tech" or --metadata "agent:me" --metadata "notes:test"
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
        replace: bool,
    },
    Export {
        file: String,
        #[arg(long)]
        status: Option<TaskStatusArg>,
        #[arg(long)]
        tags: Vec<String>,
    },
    /// Detect file-level conflicts between tasks
    Conflicts {
        /// Only consider tasks with this status (can be repeated)
        #[arg(long)]
        status: Vec<TaskStatusArg>,
        /// Only check tasks that are ready (dependencies satisfied)
        #[arg(long)]
        ready_only: bool,
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
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
struct TaskStatusArg(TaskStatus);

impl std::str::FromStr for TaskStatusArg {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" | "todo" => Ok(TaskStatusArg(TaskStatus::Pending)),
            "in_progress" | "progress" | "doing" | "inprogress" => {
                Ok(TaskStatusArg(TaskStatus::InProgress))
            }
            "done" | "completed" | "complete" => Ok(TaskStatusArg(TaskStatus::Done)),
            "blocked" => Ok(TaskStatusArg(TaskStatus::Blocked)),
            _ => Err(format!(
                "Invalid status '{}'. Valid options: pending, in_progress, done, blocked",
                s
            )),
        }
    }
}

fn print_tasks(tasks: &[Task], format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(tasks)?),
        OutputFormat::Yaml => println!("{}", serde_yml::to_string(tasks)?),
        OutputFormat::Table => format_table(tasks),
    }
    Ok(())
}

fn format_table(tasks: &[Task]) {
    println!(
        "{:<12} {:<30} {:<10} {:<15} {:<15} {:?}",
        "ID", "TITLE", "PRIORITY", "STATUS", "AGENT", "DEPS"
    );
    println!("{}", "-".repeat(95));
    for t in tasks {
        let deps = if t.deps.is_empty() {
            "-".to_string()
        } else {
            t.deps.join(", ")
        };
        let agent = t
            .metadata
            .get("agent")
            .cloned()
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<12} {:<30} {:<10} {:<15} {:<15} {}",
            t.id,
            truncate(&t.title, 28),
            t.priority,
            format!("{:?}", t.status),
            truncate(&agent, 13),
            deps
        );
    }
}

fn format_task_detail(task: &Task) {
    println!("ID:          {}", task.id);
    println!("Title:       {}", task.title);
    if let Some(desc) = &task.description {
        println!("Description: {}", desc);
    }
    println!("Status:      {:?}", task.status);
    println!("Priority:    {}", task.priority);
    println!("Deps:        {:?}", task.deps);
    println!("Tags:        {:?}", task.tags);
    println!("Files:       {:?}", task.files);
    println!("Created:     {}", task.created_at);
    println!("Updated:     {}", task.updated_at);
    if !task.metadata.is_empty() {
        println!("Metadata:");
        for (k, v) in &task.metadata {
            println!("  {}: {}", k, v);
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let truncated: String = s.chars().take(max - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

fn main() {
    if let Err(e) = run() {
        dagrobin::error::exit_with_error(&e);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| DagRobinError::InvalidInput {
                message: format!(
                    "Failed to create database directory {}: {}",
                    parent.display(),
                    e
                ),
            })?;
        }
    }

    let db_str = db_path
        .to_str()
        .ok_or_else(|| DagRobinError::InvalidInput {
            message: "Database path contains invalid UTF-8".to_string(),
        })?;
    let db = Database::new(db_str)?;

    match &cli.command {
        Commands::Add {
            id,
            title,
            description,
            priority,
            deps,
            tags,
            files,
        } => {
            let mut task = Task::new(id, title);
            task.description = description.clone();
            if let Some(p) = priority {
                task.priority = *p;
            }
            task.deps = deps.clone();
            task.tags = tags.clone();
            task.files = files.clone();
            db.upsert(&task)?;
            println!("Created task: {}", id);
        }

        Commands::Get { id } => {
            let task = db.get(id).map_err(|_| DagRobinError::TaskNotFound {
                task_id: id.clone(),
            })?;
            format_task_detail(&task);
        }

        Commands::List {
            status,
            priority_min,
            tags,
            format,
        } => {
            let mut tasks = match status {
                Some(s) => db.list_by_status(&s.0)?,
                None => db.list_all()?,
            };

            if let Some(min) = priority_min {
                tasks.retain(|t| t.priority <= *min);
            }
            if !tags.is_empty() {
                tasks.retain(|t| tags.iter().any(|tag| t.tags.contains(tag)));
            }

            print_tasks(&tasks, format)?;
        }

        Commands::Ready {
            format,
            priority_min,
        } => {
            let mut tasks = db.ready_tasks()?;
            if let Some(min) = priority_min {
                tasks.retain(|t| t.priority <= *min);
            }
            print_tasks(&tasks, format)?;
        }

        Commands::Blocked { format } => {
            let blocked = db.blocked_tasks()?;
            match format {
                OutputFormat::Json => {
                    let data: Vec<_> = blocked
                        .iter()
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
            let task = db.get(id).map_err(|_| DagRobinError::TaskNotFound {
                task_id: id.clone(),
            })?;
            std::process::exit(if db.is_ready(&task) { 0 } else { 1 });
        }

        Commands::Claim { id, agent } => {
            let agent = agent.clone().unwrap_or_else(|| {
                std::env::var("AGENT")
                    .or_else(|_| std::env::var("USER"))
                    .unwrap_or_else(|_| "cli".to_string())
            });

            let task = db.get(id).map_err(|_| DagRobinError::TaskNotFound {
                task_id: id.clone(),
            })?;

            if task.status == TaskStatus::InProgress {
                let current_agent = task
                    .metadata
                    .get("agent")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());
                return Err(DagRobinError::TaskAlreadyClaimed {
                    task_id: id.clone(),
                    agent: current_agent,
                });
            }

            if task.status == TaskStatus::Done {
                return Err(DagRobinError::TaskAlreadyDone {
                    task_id: id.clone(),
                });
            }

            let mut updated = task;
            updated.status = TaskStatus::InProgress;
            updated.metadata.insert("agent".to_string(), agent.clone());
            updated
                .metadata
                .insert("started_at".to_string(), chrono::Utc::now().to_rfc3339());
            db.upsert(&updated)?;
            println!("Claimed task '{}' for agent '{}'", id, agent);
        }

        Commands::Delete { id, force } => {
            if !*force {
                let dependents = db.get_dependents(id)?;
                if !dependents.is_empty() {
                    return Err(DagRobinError::TaskHasDependents {
                        task_id: id.clone(),
                    });
                }
            }
            db.delete(id).map_err(|_| DagRobinError::TaskNotFound {
                task_id: id.clone(),
            })?;
            println!("Deleted: {}", id);
        }

        Commands::Update {
            id,
            status,
            title,
            description,
            metadata,
        } => {
            let mut task = db.get(id).map_err(|_| DagRobinError::TaskNotFound {
                task_id: id.clone(),
            })?;
            if let Some(s) = status {
                task.status = s.0;
            }
            if let Some(t) = title {
                task.title = t.clone();
            }
            if let Some(d) = description {
                task.description = Some(d.clone());
            }
            for m in metadata {
                for part in m.split(';') {
                    if let Some((k, v)) = part.split_once(':') {
                        let key = k.trim().to_string();
                        let value = v.trim().to_string();
                        if !key.is_empty() && !value.is_empty() {
                            task.metadata.insert(key, value);
                        }
                    }
                }
            }
            task.updated_at = chrono::Utc::now();
            db.upsert(&task)?;
            println!("Updated: {}", id);
        }

        Commands::Graph { format, output } => {
            let tasks = db.list_all()?;
            let graph = generate_graph(&tasks, format);

            match output {
                Some(f) => {
                    std::fs::write(f, &graph)?;
                    println!("Graph written to file");
                }
                None => print!("{}", graph),
            }
        }

        Commands::Import { file, replace } => {
            if *replace {
                for task in db.list_all()? {
                    let _ = db.delete(&task.id);
                }
            }
            let content = std::fs::read_to_string(file)?;
            match serde_yml::from_str::<Vec<Task>>(&content) {
                Ok(tasks) => {
                    let count = tasks.len();
                    for task in tasks {
                        db.upsert(&task)?;
                    }
                    println!("Imported {} tasks", count);
                }
                Err(e) => {
                    eprintln!("Failed to parse YAML: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Export { file, status, tags } => {
            let mut tasks = match status {
                Some(s) => db.list_by_status(&s.0)?,
                None => db.list_all()?,
            };
            if !tags.is_empty() {
                tasks.retain(|t| tags.iter().any(|tag| t.tags.contains(tag)));
            }
            let yaml = serde_yml::to_string(&tasks)?;
            std::fs::write(file, &yaml)?;
            println!("Exported {} tasks", tasks.len());
        }

        Commands::Conflicts {
            status,
            ready_only,
            format,
        } => {
            let statuses: Vec<_> = status.iter().map(|s| s.0).collect();
            let conflicts = db.file_conflicts(&statuses, *ready_only)?;

            if conflicts.is_empty() {
                println!("No file conflicts detected.");
                return Ok(());
            }

            match format {
                OutputFormat::Json => {
                    let data: Vec<_> = conflicts
                        .iter()
                        .map(|c| {
                            let tasks: Vec<_> = c
                                .tasks
                                .iter()
                                .map(|t| {
                                    serde_json::json!({
                                        "id": t.id,
                                        "title": t.title,
                                        "status": format!("{:?}", t.status),
                                    })
                                })
                                .collect();
                            serde_json::json!({
                                "file": c.file,
                                "tasks": tasks,
                            })
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({ "conflicts": data }))?
                    );
                }
                OutputFormat::Yaml => {
                    for c in &conflicts {
                        println!("- file: {}", c.file);
                        println!("  tasks:");
                        for t in &c.tasks {
                            println!("    - id: {}", t.id);
                            println!("      title: {}", t.title);
                            println!("      status: {:?}", t.status);
                        }
                    }
                }
                OutputFormat::Table => {
                    for c in &conflicts {
                        println!("Conflict: {}", c.file);
                        for t in &c.tasks {
                            println!("  - {}: \"{}\" ({:?})", t.id, t.title, t.status);
                        }
                    }
                }
            }
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
                lines.push(format!(
                    "    \"{}\" [label=\"{}\\n({:?})\"];",
                    t.id, label, t.status
                ));
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
