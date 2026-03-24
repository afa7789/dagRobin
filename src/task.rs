use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the current state of a task.
///
/// # Variants
///
/// - `Pending` - Task is waiting to be worked on (default)
/// - `InProgress` - Task is currently being worked on
/// - `Done` - Task has been completed
/// - `Blocked` - Task cannot be worked on due to dependencies
///
/// # Example
///
/// ```rust
/// use dagrobin::task::TaskStatus;
///
/// let status = TaskStatus::Pending;
/// assert_eq!(status, TaskStatus::Pending);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Done,
    Blocked,
}

/// A task with dependencies tracked as a DAG.
///
/// Tasks can have dependencies on other tasks. A task is "ready" when
/// all of its dependencies have status `Done`.
///
/// # Creating a Task
///
/// ```rust
/// use dagrobin::task::Task;
///
/// let task = Task::new("setup", "Initial setup");
/// assert_eq!(task.id, "setup");
/// assert_eq!(task.title, "Initial setup");
/// assert_eq!(task.status, dagrobin::task::TaskStatus::Pending);
/// ```
///
/// # With Dependencies
///
/// ```rust
/// use dagrobin::task::Task;
///
/// let mut task = Task::new("api", "Build API");
/// task.deps = vec!["setup".to_string()];
/// assert!(task.deps.contains(&"setup".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Optional detailed description
    pub description: Option<String>,
    /// Current status
    pub status: TaskStatus,
    /// Priority (lower number = higher priority)
    pub priority: u32,
    /// List of task IDs this task depends on
    pub deps: Vec<String>,
    /// List of file paths this task affects
    pub files: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Additional key-value metadata
    pub metadata: HashMap<String, String>,
    /// When the task was created
    pub created_at: DateTime<Utc>,
    /// When the task was last updated
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Creates a new task with default values.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier (e.g., "setup-1", "feature-auth")
    /// * `title` - Human-readable description
    ///
    /// # Example
    ///
    /// ```rust
    /// use dagrobin::task::Task;
    ///
    /// let task = Task::new("t1", "My task");
    /// assert_eq!(task.id, "t1");
    /// assert_eq!(task.title, "My task");
    /// ```
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
