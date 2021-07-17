use async_trait::async_trait;

#[async_trait]
pub trait Bag<T> {
    async fn collect(&mut self, t: T);
    async fn flush(&mut self) -> Vec<T>;
}

#[async_trait]
impl<T> Bag<T> for Vec<T>
where
    T: Clone + Send,
{
    async fn collect(&mut self, t: T) {
        self.push(t);
    }

    async fn flush(&mut self) -> Vec<T> {
        let buffer = self.to_owned();
        self.clear();
        buffer
    }
}
