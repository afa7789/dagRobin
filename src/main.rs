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
    #[arg(short, long, default_value = "dagrobin.db")]
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
        agent: String,
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
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" | "todo" => Ok(TaskStatusArg("pending".to_string())),
            "in_progress" | "progress" | "doing" | "inprogress" => {
                Ok(TaskStatusArg("in_progress".to_string()))
            }
            "done" | "completed" | "complete" => Ok(TaskStatusArg("done".to_string())),
            "blocked" => Ok(TaskStatusArg("blocked".to_string())),
            _ => Err(format!(
                "Invalid status '{}'. Valid options: pending, in_progress, done, blocked",
                s
            )),
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
    println!(
        "{:<12} {:<30} {:<10} {:<15} {:?}",
        "ID", "TITLE", "PRIORITY", "STATUS", "DEPS"
    );
    println!("{}", "-".repeat(80));
    for t in tasks {
        let deps = if t.deps.is_empty() {
            "-".to_string()
        } else {
            t.deps.join(", ")
        };
        println!(
            "{:<12} {:<30} {:<10} {:<15} {}",
            t.id,
            truncate(&t.title, 28),
            t.priority,
            format!("{:?}", t.status),
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
    if s.len() > max {
        format!("{}...", &s[..max - 3])
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
    let db = Database::new(cli.db.to_str().unwrap())?;

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
            tags: _,
            format,
        } => {
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

        Commands::Ready {
            format,
            priority_min,
        } => {
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
            let ready = task.deps.iter().all(|dep| {
                db.get(dep)
                    .map(|d| d.status == TaskStatus::Done)
                    .unwrap_or(false)
            });
            std::process::exit(if ready { 0 } else { 1 });
        }

        Commands::Claim { id, agent } => {
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
                if let Ok(task) = db.get(id) {
                    if !task.deps.is_empty() {
                        return Err(DagRobinError::TaskHasDependents {
                            task_id: id.clone(),
                        });
                    }
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

        Commands::Import {
            file,
            merge: _,
            replace,
        } => {
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
            println!("Imported tasks");
        }

        Commands::Export {
            file,
            status,
            tags: _,
        } => {
            let tasks = match status {
                Some(s) => db.list_by_status(&parse_status(&s.0))?,
                None => db.list_all()?,
            };
            let yaml = serde_yaml::to_string(&tasks)?;
            std::fs::write(file, &yaml)?;
            println!("Exported {} tasks", tasks.len());
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
