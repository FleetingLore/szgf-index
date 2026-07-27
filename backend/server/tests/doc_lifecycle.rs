//! 文档生命周期集成测试
//!
//! 测试文档的创建、弃用、锁定和删除的完整流程。
//! 运行: cargo test --test doc_lifecycle

use ducia_core::doc::model::CreateDocRequest;
use ducia_core::doc::repo::DocRepository;
use ducia_storage_fs::FsStorage;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup() -> (FsStorage, TempDir, TempDir, TempDir) {
    let config = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    let docs = TempDir::new().unwrap();
    let storage = FsStorage::new(
        PathBuf::from(config.path()),
        PathBuf::from(data.path()),
        PathBuf::from(docs.path()),
    );
    (storage, config, data, docs)
}

async fn create_test_doc(storage: &FsStorage) -> ducia_core::doc::model::DocMeta {
    let req = CreateDocRequest {
        title: "测试文档".into(),
        content: "# 测试\n内容".into(),
    };
    storage.create_doc(req).await.unwrap()
}

#[tokio::test]
async fn test_create_and_list() {
    let (storage, _config, _data, _docs) = setup();
    let meta = create_test_doc(&storage).await;
    assert_eq!(meta.title, "测试文档");
    assert!(!meta.deprecated);
    assert!(!meta.deleted);
    assert!(!meta.locked);

    let docs = storage.list_docs(false).await.unwrap();
    assert_eq!(docs.len(), 1);
}

#[tokio::test]
async fn test_deprecate_and_restore() {
    let (storage, _config, _data, _docs) = setup();
    let meta = create_test_doc(&storage).await;

    storage
        .update_meta(&meta.id, Some(true), None, None)
        .await
        .unwrap();
    assert!(storage.get_doc(&meta.id).await.unwrap().unwrap().deprecated);

    storage
        .update_meta(&meta.id, Some(false), None, None)
        .await
        .unwrap();
    assert!(!storage.get_doc(&meta.id).await.unwrap().unwrap().deprecated);
}

#[tokio::test]
async fn test_lock_persists() {
    let (storage, _config, _data, _docs) = setup();
    let meta = create_test_doc(&storage).await;

    storage
        .update_meta(&meta.id, None, None, Some(true))
        .await
        .unwrap();
    assert!(storage.get_doc(&meta.id).await.unwrap().unwrap().locked);

    storage
        .update_meta(&meta.id, Some(true), None, None)
        .await
        .unwrap();
    let doc = storage.get_doc(&meta.id).await.unwrap().unwrap();
    assert!(doc.deprecated);
    assert!(doc.locked);
}

#[tokio::test]
async fn test_locked_deprecated_then_delete() {
    let (storage, _config, _data, _docs) = setup();
    let meta = create_test_doc(&storage).await;

    storage
        .update_meta(&meta.id, Some(true), None, Some(true))
        .await
        .unwrap();
    let doc = storage.get_doc(&meta.id).await.unwrap().unwrap();
    assert!(doc.deprecated);
    assert!(doc.locked);

    storage
        .update_meta(&meta.id, None, Some(true), None)
        .await
        .unwrap();
    assert!(storage.list_docs(false).await.unwrap().is_empty());
}

#[tokio::test]
async fn test_soft_delete() {
    let (storage, _config, _data, _docs) = setup();
    let meta = create_test_doc(&storage).await;

    storage
        .update_meta(&meta.id, None, Some(true), None)
        .await
        .unwrap();

    assert!(storage.list_docs(false).await.unwrap().is_empty());
}

#[tokio::test]
async fn test_unlock() {
    let (storage, _config, _data, _docs) = setup();
    let meta = create_test_doc(&storage).await;

    storage
        .update_meta(&meta.id, None, None, Some(true))
        .await
        .unwrap();
    assert!(storage.get_doc(&meta.id).await.unwrap().unwrap().locked);

    storage
        .update_meta(&meta.id, None, None, Some(false))
        .await
        .unwrap();
    assert!(!storage.get_doc(&meta.id).await.unwrap().unwrap().locked);
}
