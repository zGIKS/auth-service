use auth_service::shared::infrastructure::circuit_breaker::AppCircuitBreaker;
use std::thread;
use std::time::Duration;

#[test]
fn test_initial_state_is_closed() {
    let cb = AppCircuitBreaker::new(3, Duration::from_secs(1));
    assert!(cb.is_call_permitted());
}

#[test]
fn test_opens_after_threshold_failures() {
    let cb = AppCircuitBreaker::new(3, Duration::from_secs(1));

    cb.on_failure();
    cb.on_failure();
    assert!(cb.is_call_permitted()); // Still closed (2 failures)

    cb.on_failure(); // 3rd failure
    
    // Should be open now
    assert!(!cb.is_call_permitted());
}

#[test]
fn test_half_open_after_timeout() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(100));

    cb.on_failure(); // Open
    assert!(!cb.is_call_permitted());

    // Wait for timeout
    thread::sleep(Duration::from_millis(150));

    // Should be HalfOpen (permitted)
    assert!(cb.is_call_permitted());
}

#[test]
fn test_half_open_to_closed_on_success() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(50));

    cb.on_failure(); // Open
    thread::sleep(Duration::from_millis(60));
    
    assert!(cb.is_call_permitted()); // Transitions to HalfOpen

    cb.on_success();
    
    // Should be Closed now
    assert!(cb.is_call_permitted());
}

#[test]
fn test_half_open_to_open_on_failure() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(50));

    cb.on_failure(); // Open
    thread::sleep(Duration::from_millis(60));
    
    assert!(cb.is_call_permitted()); // Transitions to HalfOpen

    cb.on_failure(); // Failed probe
    
    // Should be Open again
    assert!(!cb.is_call_permitted());
}
