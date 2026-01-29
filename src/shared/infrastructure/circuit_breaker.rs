use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    failure_count: u32,
    last_failure_time: Option<Instant>,
    failure_threshold: u32,
    open_timeout: Duration,
}

impl AppCircuitBreaker {
    pub fn new(failure_threshold: u32, open_timeout: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                state: State::Closed,
                failure_count: 0,
                last_failure_time: None,
                failure_threshold,
                open_timeout,
            })),
        }
    }

    pub fn is_call_permitted(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            State::Closed => true,
            State::Open => {
                if let Some(last_fail) = inner.last_failure_time {
                    if last_fail.elapsed() >= inner.open_timeout {
                        inner.state = State::HalfOpen;
                        return true;
                    }
                }
                false
            }
            State::HalfOpen => {
                // In a more complex implementation, we might want to ensure only one request 
                // acts as the "probe". For now, we allow calls in HalfOpen. 
                // If they succeed, we close. If fail, we open.
                true
            }
        }
    }

    pub fn on_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == State::HalfOpen {
            inner.state = State::Closed;
            inner.failure_count = 0;
            inner.last_failure_time = None;
        } else if inner.state == State::Closed {
            // Optional: Reset failure count on success? 
            // Usually yes, or sliding window. Simple count reset is fine.
            inner.failure_count = 0;
        }
    }

    pub fn on_failure(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.failure_count += 1;
        inner.last_failure_time = Some(Instant::now());

        match inner.state {
            State::Closed => {
                if inner.failure_count >= inner.failure_threshold {
                    inner.state = State::Open;
                }
            }
            State::HalfOpen => {
                inner.state = State::Open;
            }
            State::Open => {
                // Restart timer
                inner.last_failure_time = Some(Instant::now());
            }
        }
    }
}

pub fn create_circuit_breaker() -> AppCircuitBreaker {
    AppCircuitBreaker::new(3, Duration::from_secs(30))
}
