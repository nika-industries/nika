use std::sync::Arc;

use async_trait::async_trait;
use aws_sdk_s3::{
  primitives::ByteStream,
  types::{CompletedMultipartUpload, CompletedPart},
  Client,
};
use object_store::multipart::{PartId, PutPart};

use crate::error::Error;

pub(crate) struct MultiPartUpload {
  pub(crate) bucket:    String,
  pub(crate) location:  String,
  pub(crate) upload_id: String,
  pub(crate) client:    Arc<Client>,
}

#[async_trait]
impl PutPart for MultiPartUpload {
  async fn put_part(
    &self,
    buf: Vec<u8>,
    part_idx: usize,
  ) -> Result<PartId, object_store::Error> {
    let part = part_idx + 1;

    let response = self
      .client
      .upload_part()
      .bucket(&self.bucket)
      .key(&self.location)
      .upload_id(&self.upload_id)
      .part_number(part as i32)
      .body(ByteStream::from(buf))
      .send()
      .await
      .map_err(Error::from)?;

    Ok(PartId {
      content_id: response
        .e_tag()
        .map(|x| x.to_string())
        .ok_or(Error::Unknown)?,
    })
  }

  async fn complete(
    &self,
    completed_parts: Vec<PartId>,
  ) -> Result<(), object_store::Error> {
    let upload = CompletedMultipartUpload::builder().set_parts(Some(
      completed_parts
        .into_iter()
        .map(|x| {
          Ok(
            CompletedPart::builder()
              .part_number(x.content_id.parse()?)
              .build(),
          )
        })
        .collect::<Result<Vec<_>, Error>>()?,
    ));
    self
      .client
      .complete_multipart_upload()
      .bucket(&self.bucket)
      .key(&self.location)
      .upload_id(&self.upload_id)
      .multipart_upload(upload.build())
      .send()
      .await
      .map_err(Error::from)?;
    Ok(())
  }
}
