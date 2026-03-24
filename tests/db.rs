use dagrobin::{
    db::Database,
    task::{Task, TaskStatus},
};
use tempfile::TempDir;

mod helpers {
    use super::*;
    use std::path::PathBuf;

    pub fn create_test_db(prefix: &str) -> (TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db_path: PathBuf = dir.path().join(prefix);
        let db = Database::new(db_path.to_str().unwrap()).unwrap();
        (dir, db)
    }

    pub fn create_task(id: &str, title: &str) -> Task {
        Task::new(id, title)
    }
}

mod task_struct {
    use super::*;

    #[test]
    fn test_task_new_creates_task_with_defaults() {
        let task = helpers::create_task("test-1", "Test Task");

        assert_eq!(task.id, "test-1");
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, 5);
        assert!(task.deps.is_empty());
        assert!(task.tags.is_empty());
        assert!(task.files.is_empty());
        assert!(task.metadata.is_empty());
    }

    #[test]
    fn test_task_timestamps_are_set() {
        let task = helpers::create_task("t1", "Task");

        assert!(task.created_at <= chrono::Utc::now());
        assert_eq!(task.created_at, task.updated_at);
    }
}

mod task_status {
    use super::*;

    #[test]
    fn test_task_status_default_is_pending() {
        assert_eq!(TaskStatus::default(), TaskStatus::Pending);
    }

    #[test]
    fn test_task_status_variants() {
        let status = TaskStatus::Pending;
        assert_eq!(format!("{:?}", status), "Pending");
    }
}

mod crud_operations {
    use super::*;

    #[test]
    fn test_insert_and_retrieve_task() {
        let (_dir, db) = helpers::create_test_db("insert.db");

        let task = helpers::create_task("task-1", "My Task");
        db.upsert(&task).unwrap();

        let retrieved = db.get("task-1").unwrap();
        assert_eq!(retrieved.id, "task-1");
        assert_eq!(retrieved.title, "My Task");
    }

    #[test]
    fn test_update_existing_task() {
        let (_dir, db) = helpers::create_test_db("update.db");

        let task = helpers::create_task("task-1", "Original");
        db.upsert(&task).unwrap();

        let mut updated = db.get("task-1").unwrap();
        updated.title = "Updated".to_string();
        updated.status = TaskStatus::Done;
        db.upsert(&updated).unwrap();

        let retrieved = db.get("task-1").unwrap();
        assert_eq!(retrieved.title, "Updated");
        assert_eq!(retrieved.status, TaskStatus::Done);
    }

    #[test]
    fn test_delete_existing_task() {
        let (_dir, db) = helpers::create_test_db("delete.db");

        let task = helpers::create_task("task-1", "To Delete");
        db.upsert(&task).unwrap();

        db.delete("task-1").unwrap();

        assert!(db.get("task-1").is_err());
    }

    #[test]
    fn test_delete_nonexistent_task_does_not_panic() {
        let (_dir, db) = helpers::create_test_db("delete_nonexistent.db");

        let result = db.delete("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_nonexistent_task_returns_error() {
        let (_dir, db) = helpers::create_test_db("get_nonexistent.db");

        let result = db.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_upsert_overwrites_existing_task() {
        let (_dir, db) = helpers::create_test_db("upsert.db");

        let task1 = helpers::create_task("task-1", "Version 1");
        db.upsert(&task1).unwrap();

        let mut task2 = helpers::create_task("task-1", "Version 2");
        task2.priority = 1;
        db.upsert(&task2).unwrap();

        let retrieved = db.get("task-1").unwrap();
        assert_eq!(retrieved.title, "Version 2");
        assert_eq!(retrieved.priority, 1);
    }
}

mod list_operations {
    use super::*;

    #[test]
    fn test_list_all_returns_all_tasks() {
        let (_dir, db) = helpers::create_test_db("list_all.db");

        db.upsert(&helpers::create_task("t1", "Task 1")).unwrap();
        db.upsert(&helpers::create_task("t2", "Task 2")).unwrap();
        db.upsert(&helpers::create_task("t3", "Task 3")).unwrap();

        let tasks = db.list_all().unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn test_list_all_returns_empty_when_no_tasks() {
        let (_dir, db) = helpers::create_test_db("list_empty.db");

        let tasks = db.list_all().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_list_by_status_filters_correctly() {
        let (_dir, db) = helpers::create_test_db("list_by_status.db");

        let task1 = helpers::create_task("pending-1", "Pending Task");
        db.upsert(&task1).unwrap();

        let mut task2 = helpers::create_task("done-1", "Done Task");
        task2.status = TaskStatus::Done;
        db.upsert(&task2).unwrap();

        let pending = db.list_by_status(&TaskStatus::Pending).unwrap();
        let done = db.list_by_status(&TaskStatus::Done).unwrap();

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "pending-1");
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].id, "done-1");
    }
}

mod ready_tasks {
    use super::*;

    #[test]
    fn test_ready_tasks_with_no_dependencies() {
        let (_dir, db) = helpers::create_test_db("ready_no_deps.db");

        db.upsert(&helpers::create_task("t1", "Task 1")).unwrap();
        db.upsert(&helpers::create_task("t2", "Task 2")).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_ready_tasks_with_satisfied_dependency() {
        let (_dir, db) = helpers::create_test_db("ready_satisfied.db");

        let mut t1 = helpers::create_task("t1", "Done Task");
        t1.status = TaskStatus::Done;
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Dependent Task");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t2");
    }

    #[test]
    fn test_ready_tasks_with_unsatisfied_dependency() {
        let (_dir, db) = helpers::create_test_db("ready_unsatisfied.db");

        let t1 = helpers::create_task("t1", "Pending Task");
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Dependent Task");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t1");
    }

    #[test]
    fn test_ready_tasks_with_multiple_dependencies_all_done() {
        let (_dir, db) = helpers::create_test_db("ready_multi_done.db");

        let mut t1 = helpers::create_task("t1", "Task 1");
        t1.status = TaskStatus::Done;
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Task 2");
        t2.status = TaskStatus::Done;
        db.upsert(&t2).unwrap();

        let mut t3 = helpers::create_task("t3", "Dependent on both");
        t3.deps = vec!["t1".to_string(), "t2".to_string()];
        db.upsert(&t3).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t3");
    }

    #[test]
    fn test_ready_tasks_with_multiple_dependencies_one_pending() {
        let (_dir, db) = helpers::create_test_db("ready_multi_pending.db");

        let mut t1 = helpers::create_task("t1", "Done");
        t1.status = TaskStatus::Done;
        db.upsert(&t1).unwrap();

        let t2 = helpers::create_task("t2", "Pending");
        db.upsert(&t2).unwrap();

        let mut t3 = helpers::create_task("t3", "Dependent on both");
        t3.deps = vec!["t1".to_string(), "t2".to_string()];
        db.upsert(&t3).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t2");
    }

    #[test]
    fn test_ready_tasks_only_returns_pending() {
        let (_dir, db) = helpers::create_test_db("ready_only_pending.db");

        let mut t1 = helpers::create_task("t1", "In Progress");
        t1.status = TaskStatus::InProgress;
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Done");
        t2.status = TaskStatus::Done;
        db.upsert(&t2).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert!(ready.is_empty());
    }
}

mod blocked_tasks {
    use super::*;

    #[test]
    fn test_blocked_tasks_with_unsatisfied_dependency() {
        let (_dir, db) = helpers::create_test_db("blocked_unsatisfied.db");

        let t1 = helpers::create_task("t1", "Pending Task");
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Blocked Task");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let blocked = db.blocked_tasks().unwrap();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].0.id, "t2");
        assert_eq!(blocked[0].1, vec!["t1"]);
    }

    #[test]
    fn test_blocked_tasks_with_nonexistent_dependency() {
        let (_dir, db) = helpers::create_test_db("blocked_nonexistent.db");

        let mut t1 = helpers::create_task("t1", "Task");
        t1.deps = vec!["nonexistent".to_string()];
        db.upsert(&t1).unwrap();

        let blocked = db.blocked_tasks().unwrap();
        assert_eq!(blocked.len(), 1);
        assert!(blocked[0].1.contains(&"nonexistent".to_string()));
    }

    #[test]
    fn test_blocked_tasks_empty_when_all_ready() {
        let (_dir, db) = helpers::create_test_db("blocked_empty.db");

        let mut t1 = helpers::create_task("t1", "Done");
        t1.status = TaskStatus::Done;
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Ready");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let blocked = db.blocked_tasks().unwrap();
        assert!(blocked.is_empty());
    }

    #[test]
    fn test_blocked_tasks_returns_only_pending() {
        let (_dir, db) = helpers::create_test_db("blocked_only_pending.db");

        let t1 = helpers::create_task("t1", "Pending");
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "In Progress");
        t2.status = TaskStatus::InProgress;
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let blocked = db.blocked_tasks().unwrap();
        assert!(blocked.is_empty());
    }
}

mod complex_scenarios {
    use super::*;

    #[test]
    fn test_chain_of_dependencies() {
        let (_dir, db) = helpers::create_test_db("chain.db");

        db.upsert(&helpers::create_task("t1", "First")).unwrap();

        let mut t2 = helpers::create_task("t2", "Second");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let mut t3 = helpers::create_task("t3", "Third");
        t3.deps = vec!["t2".to_string()];
        db.upsert(&t3).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t1");
    }

    #[test]
    fn test_diamond_dependency() {
        let (_dir, db) = helpers::create_test_db("diamond.db");

        db.upsert(&helpers::create_task("t1", "Base")).unwrap();

        let mut t2 = helpers::create_task("t2", "Branch A");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let mut t3 = helpers::create_task("t3", "Branch B");
        t3.deps = vec!["t1".to_string()];
        db.upsert(&t3).unwrap();

        let mut t4 = helpers::create_task("t4", "Merge");
        t4.deps = vec!["t2".to_string(), "t3".to_string()];
        db.upsert(&t4).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t1");
    }

    #[test]
    fn test_multiple_independent_chains() {
        let (_dir, db) = helpers::create_test_db("multi_chain.db");

        let mut a1 = helpers::create_task("a1", "Chain A");
        a1.deps = vec![];
        db.upsert(&a1).unwrap();

        let mut b1 = helpers::create_task("b1", "Chain B");
        b1.deps = vec![];
        db.upsert(&b1).unwrap();

        let mut a2 = helpers::create_task("a2", "Chain A2");
        a2.deps = vec!["a1".to_string()];
        db.upsert(&a2).unwrap();

        let mut b2 = helpers::create_task("b2", "Chain B2");
        b2.deps = vec!["b1".to_string()];
        db.upsert(&b2).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 2);
        let ids: Vec<_> = ready.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"a1"));
        assert!(ids.contains(&"b1"));
    }

    #[test]
    fn test_update_dependency_to_done_makes_task_ready() {
        let (_dir, db) = helpers::create_test_db("update_dep.db");

        let t1 = helpers::create_task("t1", "Pending");
        db.upsert(&t1).unwrap();

        let mut t2 = helpers::create_task("t2", "Blocked");
        t2.deps = vec!["t1".to_string()];
        db.upsert(&t2).unwrap();

        let ready_before = db.ready_tasks().unwrap();
        assert_eq!(ready_before.len(), 1);
        assert_eq!(ready_before[0].id, "t1");

        let mut t1_updated = db.get("t1").unwrap();
        t1_updated.status = TaskStatus::Done;
        db.upsert(&t1_updated).unwrap();

        let ready = db.ready_tasks().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "t2");
    }
}
