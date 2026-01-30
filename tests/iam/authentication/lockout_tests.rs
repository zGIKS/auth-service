use auth_service::shared::infrastructure::services::account_lockout::{
    AccountLockoutService, AccountLockoutVerifier,
};
use redis::Client;

#[tokio::test]
async fn test_lockout_isolation_by_ip() {
    // Setup Redis client (assuming local redis is running as per other tests)
    // We'll use a prefix or mock if possible, but integration test is better given the redis dependency.
    let client = Client::open("redis://127.0.0.1/").unwrap();
    let service = AccountLockoutService::new(client.clone());

    let email = "victim@example.com";
    let attacker_ip = "192.168.1.666";
    let user_ip = "192.168.1.100";

    // Clear any existing state
    let _: () = service
        .reset_failure(email, Some(attacker_ip))
        .await
        .unwrap();
    let _: () = service.reset_failure(email, Some(user_ip)).await.unwrap();
    let _: () = service.reset_failure(email, None).await.unwrap();

    // 1. Attacker tries and fails 5 times
    for _ in 0..5 {
        service
            .register_failure(email, Some(attacker_ip), 5, 5)
            .await
            .unwrap();
    }

    // 2. Attacker should be locked out
    let result = service.check_locked(email, Some(attacker_ip)).await;
    assert!(result.is_err(), "Attacker should be locked out");

    // 3. User should NOT be locked out (using their own IP)
    let result = service.check_locked(email, Some(user_ip)).await;
    assert!(
        result.is_ok(),
        "Legitimate user should NOT be locked out despite attacker failures"
    );

    // 4. Global check (if we implemented it) - currently our check_locked checks global AND specific.
    // Since we didn't trigger global lock (we passed IP to register_failure), global check (without IP) should be clean?
    // Wait, check_locked(email, None) checks global only.
    // register_failure(email, Some(ip)...) ONLY sets IP-specific lock.
    // So global check should be OK.
    let result = service.check_locked(email, None).await;
    assert!(
        result.is_ok(),
        "Global lock should not be engaged by IP-specific failures"
    );

    // Cleanup
    let _: () = service
        .reset_failure(email, Some(attacker_ip))
        .await
        .unwrap();
}
