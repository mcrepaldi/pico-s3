mod common;

#[tokio::test]
async fn no_such_bucket_for_head() {
    let server = common::start().await;
    let err = server
        .client
        .head_bucket()
        .bucket("missing-bucket")
        .send()
        .await
        .expect_err("expected error");
    let rendered = format!("{err}");
    assert!(!rendered.is_empty());
}

#[tokio::test]
async fn get_object_errors_for_missing_bucket_and_key() {
    let server = common::start().await;

    server
        .client
        .get_object()
        .bucket("missing-bucket")
        .key("k")
        .send()
        .await
        .expect_err("missing bucket should fail");

    server
        .client
        .create_bucket()
        .bucket("existing-bucket")
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .get_object()
        .bucket("existing-bucket")
        .key("missing-key")
        .send()
        .await
        .expect_err("missing key should fail");
}

#[tokio::test]
async fn complete_missing_upload_fails() {
    let server = common::start().await;
    let bucket = "missing-upload-bucket";
    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .complete_multipart_upload()
        .bucket(bucket)
        .key("x")
        .upload_id("nope")
        .multipart_upload(aws_sdk_s3::types::CompletedMultipartUpload::builder().build())
        .send()
        .await
        .expect_err("missing upload should fail");
}
