//! Integration tests for stable `account_id` keying (#618).
//!
//! Covers the three scenarios required by the issue:
//!
//! 1. Multi-tab: same CF-Access email from two browser tabs → same `account_id`.
//! 2. Email-change: `link_provider` re-links with a new email but the stored
//!    `account_id` (which is the old `session_id` value) remains stable.
//! 3. Auth regression: the `resolve_account_id` path mints a valid UUID and
//!    returns it consistently on every subsequent call.

use parish_server::session_store_impl::{SqliteIdentityStore, open_sessions_db};

use parish_core::identity::IdentityStore as _;

// ── 1. Multi-tab: same email → same account_id ──────────────────────────────

/// When the same CF-Access email arrives from two separate browser tabs, both
/// tabs must resolve to the same stable `account_id` UUID.
///
/// Simulates the `resolve_account_id` logic in `lib.rs` by calling
/// `SqliteIdentityStore` directly: first call mints a UUID and persists it,
/// second call returns the same UUID without minting a new one.
#[test]
fn same_email_two_tabs_get_same_account_id() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_sessions_db(tmp.path()).unwrap();
    let store = SqliteIdentityStore::new(std::sync::Arc::clone(&conn));

    const PROVIDER: &str = "cf-access";
    let email = "alice@example.com";

    // Tab 1 — first request: no account yet → mint.
    let account_id_1 = {
        let existing = store.lookup_by_provider(PROVIDER, email);
        assert!(existing.is_none(), "no account should exist yet");

        let new_id = uuid::Uuid::new_v4().to_string();
        store.create_account(&new_id);
        store.link_provider(PROVIDER, email, &new_id, email);
        new_id
    };

    // Tab 2 — second request: account already linked → return same ID.
    let account_id_2 = {
        let existing = store.lookup_by_provider(PROVIDER, email);
        assert!(existing.is_some(), "account should exist after tab 1");
        existing.unwrap()
    };

    assert_eq!(
        account_id_1, account_id_2,
        "both tabs must resolve to the same account_id UUID"
    );
}

// ── 2. Email-change: account_id stays stable after re-link ──────────────────

/// `link_provider` re-links an OAuth identity with a new display name (simulating
/// an email update at the provider).  The stored `account_id` (returned by
/// `lookup_by_provider`) must remain the same UUID — it must not be replaced.
#[test]
fn link_provider_email_change_preserves_account_id() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_sessions_db(tmp.path()).unwrap();
    let store = SqliteIdentityStore::new(std::sync::Arc::clone(&conn));

    let original_account_id = uuid::Uuid::new_v4().to_string();
    store.create_account(&original_account_id);

    // Initial link with the original email.
    store.link_provider(
        "google",
        "google-sub-abc",
        &original_account_id,
        "alice@old.com",
    );

    // Simulate email change at Google — provider sub is stable, display_name changes.
    store.link_provider(
        "google",
        "google-sub-abc",
        &original_account_id,
        "alice@new.com",
    );

    // account_id must not have changed.
    let looked_up = store
        .lookup_by_provider("google", "google-sub-abc")
        .expect("account must still exist after re-link");

    assert_eq!(
        looked_up, original_account_id,
        "account_id must remain stable after email/display-name change"
    );

    // get_account should return the updated display name.
    let (sub, display_name) = store
        .get_account(&original_account_id)
        .expect("get_account must work after re-link");
    assert_eq!(sub, "google-sub-abc");
    assert_eq!(
        display_name, "alice@new.com",
        "display_name must reflect the updated email"
    );
}

// ── 3. Auth flow regression: valid AuthContext is produced ───────────────────

/// Simulates the `resolve_account_id` logic in `lib.rs` end-to-end:
/// a fresh email produces a valid UUID, and repeated calls return the same UUID.
#[test]
fn auth_flow_produces_valid_auth_context_uuid() {
    let tmp = tempfile::tempdir().unwrap();
    let conn = open_sessions_db(tmp.path()).unwrap();
    let store = SqliteIdentityStore::new(std::sync::Arc::clone(&conn));

    const PROVIDER: &str = "cf-access";
    let email = "bob@example.com";
    let flags = parish_core::config::FeatureFlags::default();

    // Helper that mirrors `resolve_account_id` from lib.rs.
    let resolve = |email: &str| -> uuid::Uuid {
        if flags.is_disabled("account-id-keying") {
            return uuid::Uuid::nil();
        }
        if let Some(existing_id) = store.lookup_by_provider(PROVIDER, email)
            && let Ok(id) = uuid::Uuid::parse_str(&existing_id)
        {
            return id;
        }
        let new_id = uuid::Uuid::new_v4();
        let id_str = new_id.to_string();
        store.create_account(&id_str);
        store.link_provider(PROVIDER, email, &id_str, email);
        new_id
    };

    // First resolution mints a new account.
    let id1 = resolve(email);
    assert_ne!(
        id1,
        uuid::Uuid::nil(),
        "account_id must not be nil when flag is enabled"
    );

    // Second resolution returns the same UUID.
    let id2 = resolve(email);
    assert_eq!(
        id1, id2,
        "repeated auth resolutions must return the same UUID"
    );

    // A different email gets a different UUID.
    let id3 = resolve("charlie@example.com");
    assert_ne!(id1, id3, "different emails must get different account_ids");
}
