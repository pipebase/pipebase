use super::StoreContext;
use crate::common::{ConfigInto, Context, Result};
use tracing::info;
pub struct ContextStore<'a> {
    name: &'a str,
}

impl<'a> ContextStore<'a> {
    pub async fn run<S, C>(
        &mut self,
        config: C,
        contexts: Vec<(String, std::sync::Arc<Context>)>,
    ) -> Result<()>
    where
        S: StoreContext<C>,
        C: ConfigInto<S> + Send,
    {
        let mut store = config.config_into().await?;
        // add context
        for (name, context) in contexts {
            store.store_context(name, context);
        }
        info!(name = self.name, ty = "cstore", "run ...");
        store.run().await?;
        info!(name = self.name, ty = "cstore", "exit ...");
        Ok(())
    }
}

impl<'a> ContextStore<'a> {
    pub fn new(name: &'a str) -> Self {
        ContextStore { name }
    }

    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
}

#[macro_export]
macro_rules! cstore {
    (
        $name:expr
    ) => {{
        ContextStore::new($name)
    }};
}

#[macro_export]
macro_rules! run_cstore {
    (
        $cstore:ident, $config:ty, $path:expr, [$( $pipe:expr ), *]
    ) => {
        {
            let mut contexts = vec![];
            $(
                contexts.push(($pipe.get_name(), $pipe.get_context()));
            )*
            tokio::spawn(async move {
                let config = <$config>::from_path($path).await.expect(&format!("invalid context store config file '{}' for '{}'", $path, $cstore.get_name()));
                match $cstore.run(config, contexts).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        tracing::error!("context store exit with error '{:#?}'", err);
                        Err(err)
                    }
                }
            })
        }
    };
}
