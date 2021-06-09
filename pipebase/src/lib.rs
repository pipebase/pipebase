mod error;
mod fanout;
mod process;
mod source;

pub use fanout::*;
pub use pipederive::*;
pub use process::*;
use serde::de::DeserializeOwned;
pub use source::*;

use async_trait::async_trait;
use log::error;

pub trait FromFile: Sized + DeserializeOwned {
    fn from_file(path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader::<std::fs::File, Self>(file)?;
        Ok(config)
    }
}

#[async_trait]
pub trait FromConfig<T>: Sized {
    async fn from_config(config: &T) -> std::result::Result<Self, Box<dyn std::error::Error>>;
}

#[macro_export]
macro_rules! spawn_join {
    (
        $( $pipe:expr ), *
    ) => {

            tokio::join!($(
                tokio::spawn(async move {
                    match $pipe.run().await {
                        Ok(_) => (),
                        Err(err) => {
                            log::error!("pipe exit with error {:#?}", err);
                            ()
                        }
                    }
                })
            ),*)

    };
}
