use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug)]
pub struct RateLimiter {
    rate: u32,
    burst: u32,
    buckets: Mutex<HashMap<String, Bucket>>,
}

#[derive(Debug)]
struct Bucket {
    tokens: f64,
    last_check: Instant,
}

impl RateLimiter {
    pub fn new(rate: u32, burst: u32) -> Self {
        Self {
            rate,
            burst,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_allowed(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().expect("rate limiter mutex poisoned");
        let now = Instant::now();
        let bucket = buckets.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: self.burst as f64,
            last_check: now,
        });

        let elapsed = now.duration_since(bucket.last_check).as_secs_f64();
        bucket.tokens =
            (bucket.tokens + elapsed * (self.rate as f64 / 60.0)).min(self.burst as f64);
        bucket.last_check = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
