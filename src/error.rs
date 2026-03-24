use thiserror::Error;

#[derive(Error, Debug)]
pub enum DagRobinError {
    #[error("Task not found: '{task_id}'\nHint: Run 'dagRobin list' to see available tasks")]
    TaskNotFound { task_id: String },

    #[error("Task '{task_id}' is already being worked on by '{agent}'\nHint: Pick a different task or wait for it to be completed")]
    TaskAlreadyClaimed { task_id: String, agent: String },

    #[error("Task '{task_id}' is already DONE\nHint: Pick a different task")]
    TaskAlreadyDone { task_id: String },

    #[error("Task '{task_id}' has dependent tasks\nHint: Use --force to delete anyway")]
    TaskHasDependents { task_id: String },

    #[error(
        "Cannot read file: '{path}'\nHint: Check if the file exists and you have read permissions"
    )]
    FileNotFound { path: String },

    #[error("Cannot write file: '{path}'\nHint: Check if the directory exists and you have write permissions")]
    FileWriteError { path: String },

    #[error("Database error: {0}\nHint: Check if the database path is valid")]
    DatabaseError(String),

    #[error("Invalid YAML format\nHint: Check the file format matches dagRobin task format")]
    InvalidYaml(String),
}

impl From<sled::Error> for DagRobinError {
    fn from(e: sled::Error) -> Self {
        DagRobinError::DatabaseError(e.to_string())
    }
}

impl From<std::io::Error> for DagRobinError {
    fn from(e: std::io::Error) -> Self {
        DagRobinError::FileWriteError {
            path: e.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for DagRobinError {
    fn from(e: serde_yaml::Error) -> Self {
        DagRobinError::InvalidYaml(e.to_string())
    }
}

impl From<serde_json::Error> for DagRobinError {
    fn from(e: serde_json::Error) -> Self {
        DagRobinError::InvalidYaml(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DagRobinError>;

pub fn print_error(err: &DagRobinError) {
    eprintln!("Error: {}", err);
}

pub fn exit_with_error(err: &DagRobinError) -> ! {
    print_error(err);
    std::process::exit(1);
}
