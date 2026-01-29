use auth_service::shared::infrastructure::circuit_breaker::AppCircuitBreaker;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_initial_state_is_closed() {
    let cb = AppCircuitBreaker::new(3, Duration::from_secs(1), Duration::from_secs(60));
    assert!(cb.is_call_permitted().await);
}

#[tokio::test]
async fn test_opens_after_threshold_failures() {
    let cb = AppCircuitBreaker::new(3, Duration::from_secs(1), Duration::from_secs(60));

    cb.on_failure().await;
    cb.on_failure().await;
    assert!(cb.is_call_permitted().await); // Still closed (2 failures)

    cb.on_failure().await; // 3rd failure

    // Should be open now
    assert!(!cb.is_call_permitted().await);
}

#[tokio::test]
async fn test_half_open_after_timeout() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(100), Duration::from_secs(60));

    cb.on_failure().await; // Open
    assert!(!cb.is_call_permitted().await);

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;

    // Should be HalfOpen (permitted)
    assert!(cb.is_call_permitted().await);
}

#[tokio::test]
async fn test_half_open_to_closed_on_success() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(50), Duration::from_secs(60));

    cb.on_failure().await; // Open
    sleep(Duration::from_millis(60)).await;

    // Transitions to HalfOpen implicitly by checking permission
    assert!(cb.is_call_permitted().await);

    cb.on_success().await;

    // Should be Closed now
    assert!(cb.is_call_permitted().await);
}

#[tokio::test]
async fn test_half_open_to_open_on_failure() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(50), Duration::from_secs(60));

    cb.on_failure().await; // Open
    sleep(Duration::from_millis(60)).await;

    assert!(cb.is_call_permitted().await); // Transitions to HalfOpen

    cb.on_failure().await; // Failed probe

    // Should be Open again
    assert!(!cb.is_call_permitted().await);
}

#[tokio::test]
async fn test_half_open_rejects_concurrent_calls() {
    let cb = AppCircuitBreaker::new(1, Duration::from_millis(50), Duration::from_secs(60));

    cb.on_failure().await; // Open
    sleep(Duration::from_millis(60)).await;

    // First call transitions to HalfOpen and is allowed (The Probe)
    assert!(cb.is_call_permitted().await, "The probe should be allowed");

    // Second call sees HalfOpen and should be rejected
    assert!(
        !cb.is_call_permitted().await,
        "Concurrent calls during probe should be rejected"
    );

    // Probe succeeds
    cb.on_success().await;

    // Circuit closes
    assert!(
        cb.is_call_permitted().await,
        "Circuit should be closed after success"
    );
}

#[tokio::test]
async fn test_sliding_window_resets_failures() {
    // Window of 200ms
    let cb = AppCircuitBreaker::new(2, Duration::from_secs(1), Duration::from_millis(200));

    cb.on_failure().await; // Failure 1

    // Wait for window to pass
    sleep(Duration::from_millis(300)).await;

    cb.on_failure().await; // Failure 2 (Should be treated as 1st active failure)

    assert!(
        cb.is_call_permitted().await,
        "Circuit should stay Closed because first failure expired"
    );
}
