use async_trait::async_trait;
use serde::Serialize;
use std::fmt::{self, Display};
use std::result::Result;
use std::sync::{RwLock, RwLockReadGuard};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Cache<T> {
    value: RwLock<T>,
    timeout: Duration,
    last_update: RwLock<Instant>,
}

#[derive(Debug, Serialize)]
pub enum Error {
    CacheExpired,
    HydrateError,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cache expired")
    }
}

impl<T: Clone> Cache<T> {
    pub fn new(value: T, timeout: u64) -> Self {
        Self {
            value: RwLock::new(value),
            timeout: Duration::from_secs(timeout),
            last_update: RwLock::new(Instant::now()),
        }
    }
    pub fn get_value(&self) -> Result<RwLockReadGuard<'_, T>, Error> {
        if self.is_valid() {
            let value_guard = self.value.read().unwrap();
            Ok(value_guard)
        } else {
            Err(Error::CacheExpired)
        }
    }
    pub fn get_unchecked(&self) -> RwLockReadGuard<'_, T> {
        let value_guard = self.value.read().unwrap();
        return value_guard;
    }
    pub fn last_updated(&self) -> Instant {
        let read_guard = self.last_update.read().unwrap();
        read_guard.clone()
    }
    pub fn is_valid(&self) -> bool {
        let read_guard = self.last_update.read().unwrap();
        read_guard.checked_add(self.timeout).unwrap() > Instant::now()
    }
    pub fn update_value(&self, value: T) {
        let mut value_guard = self.value.write().unwrap();
        *value_guard = value;
        let mut time_guard = self.last_update.write().unwrap();
        *time_guard = Instant::now();
    }
    pub async fn get_or_hydrate<H: Hydrator<T>>(
        &self,
        hydrator: &H,
    ) -> Result<RwLockReadGuard<'_, T>, Error> {
        if self.is_valid() {
            Ok(self.value.read().unwrap())
        } else {
            match hydrator.hydrate().await {
                Err(e) => Err(e),
                Ok(update) => {
                    self.update_value(update);
                    Ok(self.get_value().unwrap())
                }
            }
        }
    }
    pub fn expire_cache(&self) {
        let mut time_guard = self.last_update.write().unwrap();
        *time_guard = time_guard.checked_sub(self.timeout).unwrap();
    }
}

#[async_trait]
pub trait Hydrator<T> {
    async fn hydrate(&self) -> Result<T, Error>;
}

pub struct HydratingCache<T: Clone, H: Hydrator<T>> {
    cache: Cache<T>,
    hydrator: H,
}

impl<T: Clone, H: Hydrator<T>> HydratingCache<T, H> {
    pub async fn new(hydrator: H, timeout: u64) -> Result<Self, Error> {
        let Ok(content) = hydrator.hydrate().await else {
            return Err(Error::HydrateError);
        };
        Ok(Self {
            cache: Cache::new(content, timeout),
            hydrator,
        })
    }
    // retrieves the cached value or tries rehydrating it
    pub async fn get(&self) -> Result<RwLockReadGuard<'_, T>, Error> {
        self.cache.get_or_hydrate(&self.hydrator).await
    }
    pub async fn hydrate(&self) -> Result<(), Error> {
        self.cache.expire_cache();
        self.cache
            .get_or_hydrate(&self.hydrator)
            .await
            .map(|_cached| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn create_cache() {
        let test_cache = Cache::new(String::from("hello world"), 20);
        assert_eq!(
            String::from("hello world"),
            test_cache.get_value().unwrap().to_owned()
        );
    }

    #[test]
    fn update_updates_last_updated() {
        let cache = Cache::new(64, 20);
        let start = cache.last_updated();
        sleep(Duration::from_millis(20));
        cache.update_value(42);
        let end = cache.last_updated();
        assert_ne!(start, end);
    }
}
