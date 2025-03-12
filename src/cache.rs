use std::time::{Duration, Instant};
use std::result::Result;

#[derive(Debug)]
pub struct Cache<T> {
    value: T,
    timeout: Duration,
    last_update: Instant,
}

#[derive(Debug)]
pub enum Error {
    CacheExpired,
}

impl<T> Cache<T> {
    fn new(value: T, timeout: u64) -> Self {
        Self {
            value,
            timeout: Duration::from_secs(timeout),
            last_update: Instant::now(),
        }
    }
    fn get_value(&self) -> Result<&T, Error>{
        if self.is_valid() {
            Ok(&self.value)
        } else {
            Err(Error::CacheExpired)
        }
    }
    fn last_updated(&self) -> &Instant {
        &self.last_update
    }
    fn is_valid(&self) -> bool {
        self.last_update + self.timeout > Instant::now()
    }
    fn update_value(&mut self, value: T) {
        self.value = value;
        self.last_update = Instant::now();
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
        assert_eq!(String::from("hello world"), test_cache.get_value().unwrap().to_owned());
    }

    #[test]
    fn update_updates_last_updated() {
        let mut cache = Cache::new(64, 20);
        let start = cache.last_updated().to_owned();
        sleep(Duration::from_millis(20));
        cache.update_value(42);
        let end = cache.last_updated();
        assert_ne!(start, *end);
    }
}
