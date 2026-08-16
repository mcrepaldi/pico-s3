mod common;

#[tokio::test]
async fn put_and_get_object() {
    let server = common::start().await;
    let bucket = "object-ops";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .put_object()
        .bucket(bucket)
        .key("hello.txt")
        .body("hello world".as_bytes().to_vec().into())
        .send()
        .await
        .expect("put object");

    let out = server
        .client
        .get_object()
        .bucket(bucket)
        .key("hello.txt")
        .send()
        .await
        .expect("get object");

    let data = out.body.collect().await.expect("collect").into_bytes();
    assert_eq!(data.as_ref(), b"hello world");
}

#[tokio::test]
async fn head_list_copy_delete_and_overwrite_object() {
    let server = common::start().await;
    let bucket = "object-ops-2";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .put_object()
        .bucket(bucket)
        .key("photos/2026/pic1.txt")
        .content_type("text/plain")
        .body("first".as_bytes().to_vec().into())
        .send()
        .await
        .expect("put first object");

    let head = server
        .client
        .head_object()
        .bucket(bucket)
        .key("photos/2026/pic1.txt")
        .send()
        .await
        .expect("head object");
    assert_eq!(head.content_length(), Some(5));
    assert!(head.e_tag().is_some());

    let listed = server
        .client
        .list_objects_v2()
        .bucket(bucket)
        .prefix("photos/")
        .delimiter("/")
        .send()
        .await
        .expect("list objects");
    assert!(
        listed
            .common_prefixes()
            .iter()
            .any(|cp| cp.prefix().unwrap_or_default() == "photos/2026/")
    );

    server
        .client
        .copy_object()
        .bucket(bucket)
        .key("photos/2026/pic1-copy.txt")
        .copy_source(format!("{}/{}", bucket, "photos/2026/pic1.txt"))
        .send()
        .await
        .expect("copy object");

    server
        .client
        .delete_object()
        .bucket(bucket)
        .key("missing.txt")
        .send()
        .await
        .expect("delete missing object is idempotent");

    server
        .client
        .put_object()
        .bucket(bucket)
        .key("photos/2026/pic1.txt")
        .body("second".as_bytes().to_vec().into())
        .send()
        .await
        .expect("overwrite object");

    let out = server
        .client
        .get_object()
        .bucket(bucket)
        .key("photos/2026/pic1.txt")
        .send()
        .await
        .expect("get overwritten object");
    let bytes = out.body.collect().await.expect("collect").into_bytes();
    assert_eq!(bytes.as_ref(), b"second");
}

#[tokio::test]
async fn supports_special_character_keys() {
    let server = common::start().await;
    let bucket = "object-ops-special";
    let key = "nested path/hello world ü.txt";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body("hello".as_bytes().to_vec().into())
        .send()
        .await
        .expect("put special object");

    let out = server
        .client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .expect("get special object");
    let bytes = out.body.collect().await.expect("collect").into_bytes();
    assert_eq!(bytes.as_ref(), b"hello");
}
