use async_trait::async_trait;

#[async_trait]
pub trait Bag<T> {
    async fn collect(&mut self, t: T) -> anyhow::Result<()>;
    async fn flush(&mut self) -> anyhow::Result<Vec<T>>;
}

#[async_trait]
impl<T> Bag<T> for Vec<T>
where
    T: Clone + Send,
{
    async fn collect(&mut self, t: T) -> anyhow::Result<()> {
        self.push(t);
        Ok(())
    }

    async fn flush(&mut self) -> anyhow::Result<Vec<T>> {
        let buffer = self.to_owned();
        self.clear();
        Ok(buffer)
    }
}
