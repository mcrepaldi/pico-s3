mod common;

#[tokio::test]
async fn create_and_head_bucket() {
    let server = common::start().await;
    let bucket = "bucket-ops";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .head_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("head bucket");
}

#[tokio::test]
async fn list_buckets_contains_created_bucket() {
    let server = common::start().await;
    let bucket = "bucket-list";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    let listed = server
        .client
        .list_buckets()
        .send()
        .await
        .expect("list buckets");

    let found = listed
        .buckets()
        .iter()
        .any(|b| b.name().unwrap_or_default() == bucket);
    assert!(found);
}

#[tokio::test]
async fn duplicate_create_and_delete_behaviors() {
    let server = common::start().await;
    let bucket = "bucket-dup";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect_err("duplicate create should fail");

    server
        .client
        .put_object()
        .bucket(bucket)
        .key("a.txt")
        .body("x".as_bytes().to_vec().into())
        .send()
        .await
        .expect("put object");

    server
        .client
        .delete_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect_err("delete non-empty should fail");

    server
        .client
        .delete_object()
        .bucket(bucket)
        .key("a.txt")
        .send()
        .await
        .expect("delete object");

    server
        .client
        .delete_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("delete empty bucket");
}
