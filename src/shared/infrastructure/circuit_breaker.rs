use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone)]
pub struct AppCircuitBreaker {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    state: State,
    failures: Vec<Instant>, // Sliding window of failures
    last_failure_time: Option<Instant>,
    failure_threshold: usize,
    open_timeout: Duration,
    failure_window: Duration, // Time window to consider failures valid
}

impl AppCircuitBreaker {
    pub fn new(failure_threshold: usize, open_timeout: Duration, failure_window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                state: State::Closed,
                failures: Vec::new(),
                last_failure_time: None,
                failure_threshold,
                open_timeout,
                failure_window,
            })),
        }
    }

    /// Checks if a request is permitted.
    /// Returns true if Closed, or if Open/HalfOpen logic allows a probe.
    pub async fn is_call_permitted(&self) -> bool {
        let mut inner = self.inner.lock().await;

        match inner.state {
            State::Closed => true,
            State::Open => {
                if let Some(last_fail) = inner.last_failure_time
                    && last_fail.elapsed() >= inner.open_timeout
                {
                    // Transition to HalfOpen for a probe
                    inner.state = State::HalfOpen;
                    return true;
                }
                false
            }
            State::HalfOpen => {
                // Only one probe allowed. If we are already in HalfOpen,
                // it means a probe is in flight. Reject others.
                false
            }
        }
    }

    pub async fn on_success(&self) {
        let mut inner = self.inner.lock().await;

        if inner.state == State::HalfOpen {
            inner.state = State::Closed;
            inner.failures.clear(); // Reset failure history on successful probe
            inner.last_failure_time = None;
        } else if inner.state == State::Closed {
            // Optional: Reset failures on success in Closed state?
            // Usually valid failures inside the window should imply "system is shaky",
            // but a success usually implies "system is healthy".
            // Implementation choice: We clear failures to reward success.
            inner.failures.clear();
        }
    }

    pub async fn on_failure(&self) {
        let mut inner = self.inner.lock().await;
        let now = Instant::now();

        // Add new failure
        inner.failures.push(now);
        inner.last_failure_time = Some(now);

        // Prune old failures based on sliding window
        let window = inner.failure_window;
        inner.failures.retain(|&t| now.duration_since(t) <= window);

        match inner.state {
            State::Closed => {
                if inner.failures.len() >= inner.failure_threshold {
                    inner.state = State::Open;
                }
            }
            State::HalfOpen => {
                inner.state = State::Open;
            }
            State::Open => {
                // Already open, just updated last_failure_time
            }
        }
    }
}

pub fn create_circuit_breaker() -> AppCircuitBreaker {
    // 3 failures within 60 seconds trigger Open state.
    // Open state lasts 30 seconds before attempting HalfOpen.
    AppCircuitBreaker::new(3, Duration::from_secs(30), Duration::from_secs(60))
}
