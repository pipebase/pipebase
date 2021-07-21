use s3::{ByteStream, Client};

pub struct S3FileSystem {
    client: Client,
}

impl S3FileSystem {
    pub fn new(client: Client) -> Self {
        S3FileSystem { client }
    }

    pub async fn put_object(
        &self,
        bucket: String,
        key: String,
        body: ByteStream,
    ) -> anyhow::Result<()> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body)
            .send()
            .await?;
        Ok(())
    }
}
