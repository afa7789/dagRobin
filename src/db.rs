use crate::task::{Task, TaskStatus};
use anyhow::Result;
use sled::Tree;

pub struct Database {
    data: Tree,
    idx_status: Tree,
    idx_priority: Tree,
    idx_tag: Tree,
    idx_deps_rev: Tree,
}

impl Database {
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

    pub fn upsert(&self, task: &Task) -> Result<()> {
        let json = serde_json::to_string(task)?;
        self.data.insert(&task.id, json.as_bytes())?;
        self.update_indices(task)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Task> {
        let data = self
            .data
            .get(id)?
            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", id))?;
        let task: Task = serde_json::from_slice(&data)?;
        Ok(task)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        if let Ok(task) = self.get(id) {
            self.remove_indices(&task)?;
        }
        self.data.remove(id)?;
        Ok(())
    }

    pub fn list_all(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        for item in self.data.iter() {
            let (_, data) = item?;
            tasks.push(serde_json::from_slice(&data)?);
        }
        Ok(tasks)
    }

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

    pub fn ready_tasks(&self) -> Result<Vec<Task>> {
        let pending = self.list_by_status(&TaskStatus::Pending)?;
        Ok(pending
            .into_iter()
            .filter(|t| {
                t.deps.iter().all(|dep| {
                    self.get(dep)
                        .map(|d| d.status == TaskStatus::Done)
                        .unwrap_or(false)
                })
            })
            .collect())
    }

    pub fn blocked_tasks(&self) -> Result<Vec<(Task, Vec<String>)>> {
        let pending = self.list_by_status(&TaskStatus::Pending)?;
        let mut blocked = Vec::new();
        for task in pending {
            let missing: Vec<String> = task
                .deps
                .iter()
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
