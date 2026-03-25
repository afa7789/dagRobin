//! Database module for dagRobin
//!
//! Provides persistent storage for tasks using Sled embedded database.
//!
//! # Example
//!
//! ```rust,ignore
//! use dagrobin::db::Database;
//! use dagrobin::task::Task;
//!
//! let db = Database::new("tasks.db").unwrap();
//! let task = Task::new("t1", "My task");
//! db.upsert(&task).unwrap();
//! ```

use crate::error::{DagRobinError, Result};
use crate::task::{Task, TaskStatus};
use sled::Tree;

/// Database for storing and querying tasks.
///
/// The database uses Sled embedded storage with multiple index trees
/// for fast queries by status, priority, tags, and dependencies.
///
/// # Creating a Database
///
/// ```rust,ignore
/// use dagrobin::db::Database;
///
/// let db = Database::new("my_tasks.db").unwrap();
/// ```
pub struct Database {
    data: Tree,
    idx_status: Tree,
    idx_priority: Tree,
    idx_tag: Tree,
    idx_deps_rev: Tree,
}

impl Database {
    /// Opens or creates a database at the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file (or directory)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    ///
    /// let db = Database::new("/tmp/tasks.db").unwrap();
    /// ```
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            data: db.open_tree("data")?,
            idx_status: db.open_tree("idx_status")?,
            idx_priority: db.open_tree("idx_priority")?,
            idx_tag: db.open_tree("idx_tag")?,
            idx_deps_rev: db.open_tree("idx_deps_rev")?,
        })
    }

    /// Inserts or updates a task.
    ///
    /// If a task with the same ID exists, it will be replaced.
    /// Indices are automatically updated.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    /// use dagrobin::task::Task;
    ///
    /// let db = Database::new("tasks.db").unwrap();
    /// let task = Task::new("t1", "My task");
    /// db.upsert(&task).unwrap();
    /// ```
    pub fn upsert(&self, task: &Task) -> Result<()> {
        if let Ok(existing) = self.get(&task.id) {
            self.remove_indices(&existing)?;
        }
        let json = serde_json::to_string(task)?;
        self.data.insert(&task.id, json.as_bytes())?;
        self.update_indices(task)?;
        Ok(())
    }

    /// Retrieves a task by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the task does not exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    ///
    /// let db = Database::new("tasks.db").unwrap();
    ///
    /// // Success case
    /// let task = db.get("t1").unwrap();
    /// println!("Task: {}", task.title);
    ///
    /// // Error case - task not found
    /// let result = db.get("nonexistent");
    /// assert!(result.is_err());
    /// ```
    pub fn get(&self, id: &str) -> Result<Task> {
        let data = self
            .data
            .get(id)?
            .ok_or_else(|| DagRobinError::TaskNotFound {
                task_id: id.to_string(),
            })?;
        let task: Task = serde_json::from_slice(&data)?;
        Ok(task)
    }

    /// Deletes a task by ID.
    ///
    /// Does not error if task doesn't exist (idempotent).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    ///
    /// let db = Database::new("tasks.db").unwrap();
    ///
    /// // Delete existing task - succeeds
    /// db.delete("t1").unwrap();
    ///
    /// // Delete non-existent task - also succeeds (idempotent)
    /// db.delete("nonexistent").unwrap();
    /// ```
    pub fn delete(&self, id: &str) -> Result<()> {
        if let Ok(task) = self.get(id) {
            self.remove_indices(&task)?;
        }
        self.data.remove(id)?;
        Ok(())
    }

    /// Lists all tasks.
    ///
    /// Returns an empty vector if no tasks exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    ///
    /// let db = Database::new("tasks.db").unwrap();
    /// let tasks = db.list_all().unwrap();
    /// println!("Total tasks: {}", tasks.len());
    /// ```
    pub fn list_all(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        for item in self.data.iter() {
            let (_, data) = item?;
            let task: Task = serde_json::from_slice(&data)?;
            tasks.push(task);
        }
        Ok(tasks)
    }

    /// Lists tasks filtered by status.
    ///
    /// # Errors
    ///
    /// Returns an error if database read fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    /// use dagrobin::task::TaskStatus;
    ///
    /// let db = Database::new("tasks.db").unwrap();
    ///
    /// // Get pending tasks
    /// let pending = db.list_by_status(&TaskStatus::Pending).unwrap();
    ///
    /// // Get done tasks
    /// let done = db.list_by_status(&TaskStatus::Done).unwrap();
    /// ```
    pub fn list_by_status(&self, status: &TaskStatus) -> Result<Vec<Task>> {
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

    /// Returns tasks that are ready to work on.
    ///
    /// A task is "ready" if:
    /// - Its status is `Pending`
    /// - All its dependencies have status `Done`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    /// use dagrobin::task::{Task, TaskStatus};
    ///
    /// let db = Database::new("tasks.db").unwrap();
    ///
    /// // Create a chain: t1 -> t2
    /// let t1 = Task::new("t1", "Setup");
    /// db.upsert(&t1).unwrap();
    ///
    /// let mut t2 = Task::new("t2", "API");
    /// t2.deps = vec!["t1".to_string()];
    /// db.upsert(&t2).unwrap();
    ///
    /// // t1 has no deps, so it's ready
    /// let ready = db.ready_tasks().unwrap();
    /// assert_eq!(ready.len(), 1);
    ///
    /// // After t1 is done, t2 becomes ready
    /// let mut t1_done = db.get("t1").unwrap();
    /// t1_done.status = TaskStatus::Done;
    /// db.upsert(&t1_done).unwrap();
    ///
    /// let ready = db.ready_tasks().unwrap();
    /// assert_eq!(ready[0].id, "t2");
    /// ```
    pub fn ready_tasks(&self) -> Result<Vec<Task>> {
        let pending = self.list_by_status(&TaskStatus::Pending)?;
        Ok(pending
            .into_iter()
            .filter(|t| self.is_ready(t))
            .collect())
    }

    /// Returns tasks that are blocked by incomplete dependencies.
    ///
    /// Each entry contains the blocked task and a list of missing dependency IDs.
    /// Returns an empty vector if no tasks are blocked.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use dagrobin::db::Database;
    /// use dagrobin::task::Task;
    ///
    /// let db = Database::new("tasks.db").unwrap();
    ///
    /// // Create blocked task
    /// let t1 = Task::new("t1", "Setup"); // pending
    /// db.upsert(&t1).unwrap();
    ///
    /// let mut t2 = Task::new("t2", "API");
    /// t2.deps = vec!["t1".to_string()];
    /// db.upsert(&t2).unwrap();
    ///
    /// // t2 is blocked by t1
    /// let blocked = db.blocked_tasks().unwrap();
    /// assert_eq!(blocked.len(), 1);
    /// assert_eq!(blocked[0].0.id, "t2");
    /// assert_eq!(blocked[0].1, vec!["t1"]);
    /// ```
    pub fn blocked_tasks(&self) -> Result<Vec<(Task, Vec<String>)>> {
        let pending = self.list_by_status(&TaskStatus::Pending)?;
        let mut blocked = Vec::new();
        for task in pending {
            let missing: Vec<String> = task
                .deps
                .iter()
                .filter(|dep| {
                    !self
                        .get(dep)
                        .map(|d| d.status == TaskStatus::Done)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            if !missing.is_empty() {
                blocked.push((task, missing));
            }
        }
        Ok(blocked)
    }

    /// Checks if a task's dependencies are all done.
    pub fn is_ready(&self, task: &Task) -> bool {
        task.deps.iter().all(|dep| {
            self.get(dep)
                .map(|d| d.status == TaskStatus::Done)
                .unwrap_or(false)
        })
    }

    /// Returns IDs of tasks that depend on the given task.
    pub fn get_dependents(&self, id: &str) -> Result<Vec<String>> {
        let prefix = format!("{}:", id);
        let mut dependents = Vec::new();
        for item in self.idx_deps_rev.scan_prefix(&prefix) {
            let (key, _) = item?;
            let dependent_id = String::from_utf8_lossy(&key[prefix.len()..]).to_string();
            dependents.push(dependent_id);
        }
        Ok(dependents)
    }

    fn update_indices(&self, task: &Task) -> Result<()> {
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

    fn remove_indices(&self, task: &Task) -> Result<()> {
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
