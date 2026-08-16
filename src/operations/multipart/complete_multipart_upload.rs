//! CompleteMultipartUpload operation.

use std::collections::HashMap;

use crate::error::S3Error;
use crate::models::Object;
use crate::store::{MetadataStore, ObjectStore};
use crate::types::ETag;
use crate::types::{S3Request, S3Response, S3ResponseBody};
use crate::xml::parser::XmlParser;
use crate::xml::templates::complete_multipart::complete_multipart_xml;

/// Handles the S3 CompleteMultipartUpload operation.
pub struct CompleteMultipartUpload;

impl CompleteMultipartUpload {
    /// Assemble the uploaded parts into a final object and finalise the
    /// multipart upload.
    ///
    /// The request body must be an XML `CompleteMultipartUpload` document
    /// listing the part numbers and their ETags in ascending order.  The
    /// operation verifies that every referenced part exists and that the
    /// provided ETags match before concatenating the part files into a single
    /// object payload.
    ///
    /// On success the response carries HTTP 200 and an `application/xml`
    /// `CompleteMultipartUploadResult` body.  The upload record and all
    /// temporary part files are deleted after the object is assembled.
    ///
    /// # Errors
    ///
    /// * [`S3Error::InvalidRequest`]: bucket, key, or `uploadId` is absent
    ///   from the request.
    /// * [`S3Error::NoSuchUpload`]: no in-progress upload exists for the
    ///   given `(bucket, key, uploadId)`.
    /// * [`S3Error::MalformedXml`]: the request body is not valid UTF-8 or
    ///   cannot be parsed as a `CompleteMultipartUpload` document.
    /// * [`S3Error::InvalidPartOrder`]: the part numbers in the request are
    ///   not strictly ascending.
    /// * [`S3Error::InvalidPart`]: a referenced part number was never
    ///   uploaded, or its provided ETag does not match the stored ETag.
    /// * [`S3Error::InternalError`]: a storage failure occurred while
    ///   assembling the parts or persisting the final object.
    pub async fn execute(
        metadata: &dyn MetadataStore,
        objects: &dyn ObjectStore,
        request: &S3Request,
    ) -> Result<S3Response, S3Error> {
        let bucket = request
            .context
            .bucket
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "bucket is required".into(),
            })?;
        let key = request
            .context
            .key
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "key is required".into(),
            })?;
        let upload_id = request
            .query
            .upload_id
            .clone()
            .ok_or_else(|| S3Error::InvalidRequest {
                message: "uploadId is required".into(),
            })?;

        if metadata
            .get_upload(&bucket, &key, &upload_id)
            .await?
            .is_none()
        {
            return Err(S3Error::NoSuchUpload { upload_id });
        }

        let body_text =
            String::from_utf8(request.body.to_vec()).map_err(|e| S3Error::MalformedXml {
                message: format!("invalid UTF-8 body: {e}"),
            })?;
        let requested_parts = XmlParser::parse_complete_multipart(&body_text)?;

        let existing_parts = metadata
            .list_upload_parts(&request.query.upload_id.clone().unwrap_or_default())
            .await?;
        let existing_by_number = existing_parts
            .into_iter()
            .map(|p| (p.part_number, p))
            .collect::<HashMap<_, _>>();

        let mut ordered_part_numbers = Vec::new();
        let mut previous = 0;
        for (part_number, etag) in &requested_parts {
            if *part_number <= previous {
                return Err(S3Error::InvalidPartOrder {
                    message: "parts must be in ascending order".into(),
                });
            }
            previous = *part_number;

            let existing =
                existing_by_number
                    .get(part_number)
                    .ok_or_else(|| S3Error::InvalidPart {
                        message: format!("missing part number {part_number}"),
                    })?;
            let expected = normalize_etag(&existing.etag);
            let provided = normalize_etag(etag);
            if expected != provided {
                return Err(S3Error::InvalidPart {
                    message: format!("etag mismatch for part {part_number}"),
                });
            }
            ordered_part_numbers.push(*part_number);
        }

        let final_etag = objects
            .assemble_parts(
                &request.query.upload_id.clone().unwrap_or_default(),
                &ordered_part_numbers,
                &bucket,
                &key,
            )
            .await?;

        let (reader, size) = objects.get_object(&bucket, &key).await?;
        drop(reader);

        metadata
            .put_object(Object {
                bucket: bucket.clone(),
                key: key.clone(),
                size,
                content_type: request
                    .headers
                    .content_type
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".into()),
                etag: final_etag.clone(),
                last_modified: chrono::Utc::now(),
                metadata: HashMap::new(),
            })
            .await?;

        metadata
            .delete_upload(
                request.context.bucket.as_deref().unwrap_or_default(),
                request.context.key.as_deref().unwrap_or_default(),
                &request.query.upload_id.clone().unwrap_or_default(),
            )
            .await?;
        metadata
            .delete_upload_parts(&request.query.upload_id.clone().unwrap_or_default())
            .await?;
        objects
            .delete_parts(&request.query.upload_id.clone().unwrap_or_default())
            .await?;

        tracing::info!(
            bucket = %bucket,
            key = %key,
            upload_id = %upload_id,
            part_count = ordered_part_numbers.len(),
            size,
            etag = %final_etag,
            "multipart upload completed"
        );

        let location = format!("/{bucket}/{key}");
        let xml = complete_multipart_xml(&location, &bucket, &key, &final_etag);
        Ok(S3Response {
            status: 200,
            headers: vec![("Content-Type".into(), "application/xml".into())],
            body: S3ResponseBody::Xml(xml),
        })
    }
}

fn normalize_etag(value: &str) -> String {
    let mut v = value.replace("&quot;", "\"");
    v = v.replace("\\\"", "\"");
    ETag::unquote(v.trim())
}
