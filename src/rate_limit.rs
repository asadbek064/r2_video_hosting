use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AuthRateLimiter {
    // IP -> (attempt_count, lockout_until)
    attempts: Arc<RwLock<HashMap<IpAddr, (u32, Option<Instant>)>>>,
}

impl AuthRateLimiter {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_and_increment(&self, ip: IpAddr) -> Result<(), Duration> {
        let mut map = self.attempts.write().await;
        let now = Instant::now();

        let entry = map.entry(ip).or_insert((0, None));

        // Check if locked out
        if let Some(lockout_until) = entry.1 {
            if now < lockout_until {
                return Err(lockout_until.duration_since(now));
            } else {
                // Lockout expired, reset
                *entry = (0, None);
            }
        }

        // Increment attempts
        entry.0 += 1;

        // Lock out after 5 failed attempts
        if entry.0 >= 5 {
            let lockout_until = now + Duration::from_secs(60);
            entry.1 = Some(lockout_until);
            return Err(Duration::from_secs(60));
        }

        Ok(())
    }

    pub async fn reset(&self, ip: IpAddr) {
        let mut map = self.attempts.write().await;
        map.remove(&ip);
    }

    // Periodic cleanup to prevent memory growth
    pub async fn cleanup_old_entries(&self) {
        let mut map = self.attempts.write().await;
        let now = Instant::now();
        map.retain(|_, (_, lockout)| {
            lockout.map_or(false, |l| now < l + Duration::from_secs(300))
        });
    }
}
