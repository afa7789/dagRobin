//! dagRobin - DAG-based task manager for autonomous agents
//!
//! This crate provides a task management system with native DAG (Directed Acyclic Graph)
//! support. It serves as a single source of truth for autonomous agents.
//!
//! # Quick Example
//!
//! ```rust
//! use dagrobin::{db::Database, task::{Task, TaskStatus}};
//! use tempfile::tempdir;
//!
//! let dir = tempdir().unwrap();
//! let db = Database::new(dir.path().join("test.db").to_str().unwrap()).unwrap();
//!
//! // Create and save a task
//! let task = Task::new("setup", "Initial setup");
//! db.upsert(&task).unwrap();
//!
//! // Retrieve it
//! let retrieved = db.get("setup").unwrap();
//! assert_eq!(retrieved.title, "Initial setup");
//!
//! // Try to get non-existent task - returns Error
//! let result = db.get("nonexistent");
//! assert!(result.is_err());
//! ```

pub mod db;
pub mod error;
pub mod task;
