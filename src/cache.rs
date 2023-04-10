use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

use rocket::tokio::sync::RwLock;
use rocket::tokio::task;

const CACHE_TTL_SECS: u64 = 120;

#[async_trait]
pub trait Cache: Send + Sync + Clone + 'static {
    fn get_timestamp(&self) -> Instant;
    async fn fetch() -> Result<Self, Box<dyn Error + Send + Sync>>;
    async fn get(cache: &Arc<RwLock<Self>>) -> Self {
        let cached = cache.read().await.clone();
        let timestamp = cached.get_timestamp();
        if timestamp.elapsed().as_secs() > CACHE_TTL_SECS {
            // Update the cache in the background
            let cache: Arc<_> = cache.clone();
            task::spawn(async move {
                match Self::fetch().await {
                    Ok(data) => *cache.write().await = data,
                    Err(e) => eprintln!("failed to update cache: {e}"),
                }
            });
        }
        cached
    }
}
