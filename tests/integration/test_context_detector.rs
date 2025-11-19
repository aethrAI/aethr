use aethr_core::context::detector::detect_project_context;
use std::fs::{self, File};
use uuid::Uuid;

#[test]
fn detector_identifies_files() {
    let tmp = std::env::temp_dir().join(format!("aethr_ctx_{}", Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    File::create(tmp.join("Dockerfile")).unwrap();
    File::create(tmp.join("package.json")).unwrap();
    let tags = detect_project_context(&tmp).unwrap();
    assert!(tags.contains(&"docker".to_string()));
    assert!(tags.contains(&"nodejs".to_string()));
}