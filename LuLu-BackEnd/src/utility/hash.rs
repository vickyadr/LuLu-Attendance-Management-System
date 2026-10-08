use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use hex;
use rand_core::OsRng;
use ring::digest::{Context, SHA256};
use std::sync::LazyLock;

/// The server secret, read once.
static SECRET: LazyLock<String> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    std::env::var("SECRET_KEY").expect("SECRET_KEY must be set; main.rs checks this at startup")
});

/// Fail fast at boot when the secret is missing, instead of letting the first
/// authenticated request panic a worker.
pub fn ensure_secret_configured() -> Result<(), String> {
    dotenvy::dotenv().ok();
    match std::env::var("SECRET_KEY") {
        Ok(v) if !v.trim().is_empty() => {
            // Touch the cache so the value is resolved before any request.
            LazyLock::force(&SECRET);
            Ok(())
        }
        _ => Err("SECRET_KEY is missing or empty".to_string()),
    }
}

/// Legacy fast hash, kept ONLY for session-token derivation and for
/// verifying password rows written before the Argon2 migration.
/// Must never be used to store a new password.
pub async fn encode_256(text: &str) -> String {
    let mut ctx = Context::new(&SHA256);
    ctx.update(text.as_bytes());
    ctx.update(SECRET.as_bytes());
    let diggest = ctx.finish();
    hex::encode(diggest.as_ref())
}

/// Hash a clear-text password with Argon2id and a fresh random salt.
/// Output is a ~97 char PHC string, so `users.user_password` must be TEXT.
pub async fn hash_password(clear: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(clear.as_bytes(), &salt)
        .map(|h| h.to_string())
        .expect("argon2 hashing should succeed")
}

/// True when the stored value is a legacy hex hash, not a PHC string.
pub fn needs_rehash(stored: &str) -> bool {
    !stored.starts_with("$argon2")
}

pub async fn verify_password(stored: &str, candidate: &str) -> bool {
    if stored.starts_with("$argon2") {
        match PasswordHash::new(stored) {
            Ok(parsed) => Argon2::default()
                .verify_password(candidate.as_bytes(), &parsed)
                .is_ok(),
            Err(_) => false,
        }
    } else {
        encode_256(candidate).await == stored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn require_secret() {
        assert!(
            std::env::var("SECRET_KEY").is_ok(),
            "SECRET_KEY must be set for these tests: run with .env sourced"
        );
    }

    #[tokio::test]
    async fn hash_password_produces_a_verifiable_phc_string() {
        let stored = hash_password("correct horse battery").await;
        assert!(stored.starts_with("$argon2"), "not a PHC string: {stored}");
        // A PHC string must fit the column it is stored in; the pre-migration
        // column was VARCHAR(80), which cannot hold this.
        assert!(
            stored.len() > 80,
            "expected >80 chars, got {}",
            stored.len()
        );
        assert!(verify_password(&stored, "correct horse battery").await);
        assert!(!verify_password(&stored, "Correct horse battery").await);
        assert!(!verify_password(&stored, "").await);
    }

    /// Two hashes of the same password must differ, or the column is a
    /// rainbow table waiting to happen.
    #[tokio::test]
    async fn hash_password_salts_every_call() {
        let a = hash_password("same-password").await;
        let b = hash_password("same-password").await;
        assert_ne!(a, b, "identical hashes mean no random salt");
        assert!(verify_password(&a, "same-password").await);
        assert!(verify_password(&b, "same-password").await);
    }

    #[tokio::test]
    async fn needs_rehash_only_flags_the_legacy_format() {
        assert!(!needs_rehash(&hash_password("x").await));
        assert!(needs_rehash("0123456789abcdef0123456789abcdef"));
        assert!(needs_rehash(""));
    }

    /// A row written before the migration must still authenticate, otherwise
    /// every existing account is locked out the moment the verifier switches.
    #[tokio::test]
    async fn legacy_sha256_rows_still_verify_and_are_flagged_for_upgrade() {
        require_secret();
        let legacy = encode_256("demo12345").await;
        assert!(!legacy.starts_with("$argon2"));
        assert!(verify_password(&legacy, "demo12345").await);
        assert!(!verify_password(&legacy, "wrong-password").await);
        assert!(needs_rehash(&legacy));
    }

    /// A malformed PHC string must fail closed, never fall through to the
    /// legacy comparison.
    #[tokio::test]
    async fn malformed_phc_strings_fail_closed() {
        for bad in ["$argon2", "$argon2id$broken", "$argon2id$v=19$m=1,t=1,p=1$"] {
            assert!(!verify_password(bad, "anything").await, "accepted {bad:?}");
            // Anything that is not a PHC string is treated as legacy, so a
            // corrupt row cannot silently verify against an empty hash.
            assert!(needs_rehash("not-a-phc-string"));
        }
    }

    /// The legacy hash is still the session-token transform, so it must stay
    /// deterministic — a salted token would break every stored session.
    #[tokio::test]
    async fn encode_256_stays_deterministic() {
        require_secret();
        assert_eq!(
            encode_256("token-value").await,
            encode_256("token-value").await
        );
        assert_ne!(
            encode_256("token-value").await,
            encode_256("other-token").await
        );
        assert_eq!(encode_256("token-value").await.len(), 64);
    }
}
