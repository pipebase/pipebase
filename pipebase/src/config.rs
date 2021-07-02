use async_trait::async_trait;
use serde::de::DeserializeOwned;

pub trait FromPath: Sized + DeserializeOwned {
    fn from_path<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader::<std::fs::File, Self>(file)?;
        Ok(config)
    }
}

#[async_trait]
pub trait FromConfig<T>: Sized {
    async fn from_config(config: &T) -> anyhow::Result<Self>;
}

#[async_trait]
pub trait ConfigInto<T: FromConfig<Self>>: Sized {
    async fn config_into(&self) -> anyhow::Result<T> {
        T::from_config(self).await
    }
}
