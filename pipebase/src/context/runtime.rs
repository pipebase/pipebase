use super::StoreContext;
use crate::common::{ConfigInto, ContextCollector, Result};
use tracing::info;
pub struct ContextStore<'a> {
    name: &'a str,
}

impl<'a> ContextStore<'a> {
    pub async fn run<S, C>(self, config: C, collector: ContextCollector) -> Result<()>
    where
        S: StoreContext<C>,
        C: ConfigInto<S> + Send,
    {
        let mut store = config.config_into().await?;
        // add context
        let contexts = collector.into_contexts();
        for (name, context) in contexts {
            store.store_context(name, context);
        }
        let name = self.name;
        info!(name = name, ty = "cstore", "run ...");
        store.run().await?;
        info!(name = name, ty = "cstore", "exit ...");
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
        $cstore:ident, $config:ident, $collector:ident
    ) => {{
        tokio::spawn(async move {
            match $cstore.run($config, $collector).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    tracing::error!("context store exit with error '{:#?}'", err);
                    Err(err)
                }
            }
        })
    }};
}
