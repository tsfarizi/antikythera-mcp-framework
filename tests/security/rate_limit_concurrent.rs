//! Concurrent rate limiter test to verify thread safety.

use antikythera_core::security::config::RateLimitConfig;
use antikythera_cli::security::rate_limit::RateLimiter;
use std::sync::Arc;
use std::thread;

#[test]
fn rate_limiter_concurrent_access_no_panic() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 100,
        requests_per_hour: 1000,
        requests_per_day: 10000,
        burst_allowance: 10,
        max_concurrent_sessions: 50,
        window_size_secs: 60,
        cleanup_interval_secs: 1,
    };
    let limiter = Arc::new(RateLimiter::new(config));

    let mut handles = vec![];

    // Spawn 50 concurrent check() calls across 10 sessions
    for i in 0..50 {
        let limiter = Arc::clone(&limiter);
        handles.push(thread::spawn(move || {
            let session_id = format!("session_{}", i % 10);
            // Should not panic even under concurrent access
            let _ = limiter.check(&session_id);
        }));
    }

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn rate_limiter_drops_cleanly() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 100,
        requests_per_hour: 1000,
        requests_per_day: 10000,
        burst_allowance: 10,
        max_concurrent_sessions: 50,
        window_size_secs: 60,
        cleanup_interval_secs: 1,
    };

    // Create and immediately drop — should complete quickly
    let start = std::time::Instant::now();
    {
        let _limiter = RateLimiter::new(config);
        // limiter dropped here
    }
    let elapsed = start.elapsed();

    // Drop should complete within 2 seconds (cleanup thread checks every 1s)
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "RateLimiter drop took too long: {:?}",
        elapsed
    );
}

#[test]
fn rate_limiter_update_config_stops_old_thread() {
    let config1 = RateLimitConfig {
        enabled: true,
        requests_per_minute: 100,
        requests_per_hour: 1000,
        requests_per_day: 10000,
        burst_allowance: 10,
        max_concurrent_sessions: 50,
        window_size_secs: 60,
        cleanup_interval_secs: 1,
    };
    let mut limiter = RateLimiter::new(config1);

    let config2 = RateLimitConfig {
        enabled: true,
        requests_per_minute: 200, // different
        requests_per_hour: 2000,
        requests_per_day: 20000,
        burst_allowance: 20,
        max_concurrent_sessions: 100,
        window_size_secs: 60,
        cleanup_interval_secs: 1,
    };

    let start = std::time::Instant::now();
    limiter.update_config(config2);
    let elapsed = start.elapsed();

    // update_config should stop old thread and start new one quickly
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "update_config took too long: {:?}",
        elapsed
    );
}
