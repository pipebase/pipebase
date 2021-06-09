mod error;
mod fanout;
mod process;
mod source;

pub use fanout::*;
pub use pipederive::*;
pub use process::*;
pub use source::*;

use async_trait::async_trait;

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
                    $pipe.run().await;
                })
            ),*)

    };
}
