use async_trait::async_trait;
use std::collections::HashSet;
use std::hash::Hash;

#[async_trait]
pub trait Set<T> {
    async fn collect(&mut self, t: T);
    async fn flush(&mut self) -> Vec<T>;
}

#[async_trait]
impl<T> Set<T> for HashSet<T>
where
    T: Hash + Eq + Clone + Send,
{
    async fn collect(&mut self, t: T) {
        self.insert(t);
    }

    async fn flush(&mut self) -> Vec<T> {
        let mut buffer: Vec<T> = Vec::new();
        for item in self.iter() {
            buffer.push(item.to_owned())
        }
        self.clear();
        buffer
    }
}
