use taskdag::{
    db::Database,
    task::{Task, TaskStatus},
};
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
    assert_eq!(ready.len(), 1);
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
