mod common;

#[tokio::test]
async fn multipart_lifecycle() {
    let server = common::start().await;
    let bucket = "multipart-ops";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    let created = server
        .client
        .create_multipart_upload()
        .bucket(bucket)
        .key("multi.txt")
        .send()
        .await
        .expect("create multipart");

    let upload_id = created.upload_id().expect("upload id").to_string();

    let p1 = server
        .client
        .upload_part()
        .bucket(bucket)
        .key("multi.txt")
        .upload_id(&upload_id)
        .part_number(1)
        .body("ab".as_bytes().to_vec().into())
        .send()
        .await
        .expect("upload part");

    let p2 = server
        .client
        .upload_part()
        .bucket(bucket)
        .key("multi.txt")
        .upload_id(&upload_id)
        .part_number(2)
        .body("cd".as_bytes().to_vec().into())
        .send()
        .await
        .expect("upload part");

    let completed = server
        .client
        .complete_multipart_upload()
        .bucket(bucket)
        .key("multi.txt")
        .upload_id(&upload_id)
        .multipart_upload(
            aws_sdk_s3::types::CompletedMultipartUpload::builder()
                .parts(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .part_number(1)
                        .e_tag(p1.e_tag().unwrap_or_default())
                        .build(),
                )
                .parts(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .part_number(2)
                        .e_tag(p2.e_tag().unwrap_or_default())
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .expect("complete multipart");

    assert!(completed.e_tag().is_some());
}

#[tokio::test]
async fn abort_and_invalid_part_paths() {
    let server = common::start().await;
    let bucket = "multipart-ops-2";

    server
        .client
        .create_bucket()
        .bucket(bucket)
        .send()
        .await
        .expect("create bucket");

    server
        .client
        .upload_part()
        .bucket(bucket)
        .key("missing.txt")
        .upload_id("missing")
        .part_number(1)
        .body("x".as_bytes().to_vec().into())
        .send()
        .await
        .expect_err("upload part for missing upload should fail");

    let created = server
        .client
        .create_multipart_upload()
        .bucket(bucket)
        .key("abort.txt")
        .send()
        .await
        .expect("create multipart");
    let upload_id = created.upload_id().expect("upload id").to_string();

    let uploaded = server
        .client
        .upload_part()
        .bucket(bucket)
        .key("abort.txt")
        .upload_id(&upload_id)
        .part_number(1)
        .body("part".as_bytes().to_vec().into())
        .send()
        .await
        .expect("upload part");
    assert!(uploaded.e_tag().is_some());

    server
        .client
        .complete_multipart_upload()
        .bucket(bucket)
        .key("abort.txt")
        .upload_id(&upload_id)
        .multipart_upload(
            aws_sdk_s3::types::CompletedMultipartUpload::builder()
                .parts(
                    aws_sdk_s3::types::CompletedPart::builder()
                        .part_number(1)
                        .e_tag("\"deadbeef\"")
                        .build(),
                )
                .build(),
        )
        .send()
        .await
        .expect_err("complete with wrong etag should fail");

    server
        .client
        .abort_multipart_upload()
        .bucket(bucket)
        .key("abort.txt")
        .upload_id(upload_id)
        .send()
        .await
        .expect("abort multipart");
}
