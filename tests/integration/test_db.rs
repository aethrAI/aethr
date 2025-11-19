use aethr_core::db::local::LocalDB;
use uuid::Uuid;
use std::path::PathBuf;
use std::fs;

#[test]
fn insert_and_search_roundtrip() {
    // Create a temporary directory for the DB
    let tmp = std::env::temp_dir().join(format!("aethr_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&tmp).expect("create tmp dir");
    let db_path = tmp.join("history.db");

    // Open DB and insert
    let db = LocalDB::new(&db_path).expect("open db");
    db.insert_command("echo HelloAethr", ".", 0, 1_700_000_000).expect("insert");
    db.insert_command("git status", ".", 0, 1_700_000_001).expect("insert");

    // Search for echo
    let res = db.search("echo", 10).expect("search");
    assert!(res.iter().any(|r| r.contains("HelloAethr")));

    // Search for git
    let res2 = db.search("git", 10).expect("search");
    assert!(res2.iter().any(|r| r.contains("git status")));
}
