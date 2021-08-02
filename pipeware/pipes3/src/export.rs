use std::path::Path;

use crate::{
    constants::{AWS_DEFAULT_REGION, DEFAULT_FILENAME_LENGTH},
    fs::S3FileSystem,
};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    export::Export,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use s3::{ByteStream, Client, Config, Region};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct S3WriterConfig {
    region: Option<String>,
    bucket: String,
    directory: String,
    filename_length: Option<usize>,
    filename_ext: Option<String>,
}

impl FromPath for S3WriterConfig {}

impl ConfigInto<S3Writer> for S3WriterConfig {}

pub struct S3Writer {
    fs: S3FileSystem,
    bucket: String,
    // key prefix
    directory: String,
    filename_length: usize,
    filename_ext: Option<String>,
}

#[async_trait]
impl FromConfig<S3WriterConfig> for S3Writer {
    async fn from_config(config: S3WriterConfig) -> anyhow::Result<Self> {
        let region = config
            .region
            .unwrap_or_else(|| AWS_DEFAULT_REGION.to_owned());
        let region = Region::new(region);
        let conf = Config::builder().region(region).build();
        let client = Client::from_conf(conf);
        let filename_length = config.filename_length.unwrap_or(DEFAULT_FILENAME_LENGTH);
        let mut directory = config.directory;
        if !directory.ends_with('/') {
            directory.push('/')
        }
        Ok(S3Writer {
            fs: S3FileSystem::new(client),
            bucket: config.bucket,
            directory,
            filename_length,
            filename_ext: config.filename_ext,
        })
    }
}

#[async_trait]
impl<T> Export<T, S3WriterConfig> for S3Writer
where
    T: AsRef<Path> + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.write(t).await
    }
}

impl S3Writer {
    async fn write<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let bytes = ByteStream::from_path(path).await?;
        let key = self.generate_file_path();
        self.fs.put_object(self.bucket.to_owned(), key, bytes).await
    }

    fn generate_file_path(&self) -> String {
        let filename: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.filename_length)
            .map(char::from)
            .collect();
        let filename = match self.filename_ext {
            Some(ref ext) => format!("{}.{}", filename, ext),
            None => filename,
        };
        format!("{}{}", self.directory, filename)
    }
}
