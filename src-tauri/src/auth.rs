use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const TOKEN_EXPIRATION_HOURS: i64 = 24 * 7; // 7 days

/// Per-installation JWT secret, initialized once at startup from the DB.
static JWT_SECRET: OnceLock<String> = OnceLock::new();

/// Initialize the JWT secret from the database. If none exists, generates
/// a cryptographically random 64-char hex string and persists it.
/// Must be called once during app setup before any token operations.
pub fn init_jwt_secret(db: &rusqlite::Connection) -> Result<()> {
    // Ensure the table exists
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_secrets (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    ).context("Failed to create app_secrets table")?;

    // Try to load existing secret
    let existing: Option<String> = db.prepare("SELECT value FROM app_secrets WHERE key = 'jwt_secret'")
        .ok()
        .and_then(|mut stmt| stmt.query_row([], |row| row.get(0)).ok());

    let secret = match existing {
        Some(s) => s,
        None => {
            // Generate a random 32-byte (64 hex char) secret
            let random_bytes: [u8; 32] = {
                let mut buf = [0u8; 32];
                getrandom::fill(&mut buf).context("Failed to generate random bytes")?;
                buf
            };
            let secret = hex::encode(random_bytes);
            db.execute(
                "INSERT INTO app_secrets (key, value) VALUES ('jwt_secret', ?1)",
                [&secret],
            ).context("Failed to persist JWT secret")?;
            log::info!("Generated new per-installation JWT secret");
            secret
        }
    };

    JWT_SECRET.set(secret).ok(); // Ignore if already set (e.g. tests)
    Ok(())
}

fn get_secret() -> &'static str {
    JWT_SECRET.get().expect("JWT secret not initialized — call auth::init_jwt_secret() during setup")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user_id
    pub username: String,
    pub is_admin: bool,
    pub exp: i64,         // Expiration timestamp
    pub iat: i64,         // Issued at timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: String,
}

/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String> {
    hash(password, DEFAULT_COST)
        .context("Failed to hash password")
}

/// Verify a password against a bcrypt hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    verify(password, hash)
        .context("Failed to verify password")
}

/// Generate a JWT token for a user
pub fn generate_token(user_id: &str, username: &str, is_admin: bool) -> Result<AuthToken> {
    let now = Utc::now();
    let expiration = now + Duration::hours(TOKEN_EXPIRATION_HOURS);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        is_admin,
        exp: expiration.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_secret().as_ref()),
    )
    .context("Failed to generate JWT token")?;

    Ok(AuthToken {
        token,
        expires_at: expiration.to_rfc3339(),
    })
}

/// Validate a JWT token and extract claims
pub fn validate_token(token: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_secret().as_ref()),
        &Validation::default(),
    )
    .context("Invalid or expired token")?;

    Ok(token_data.claims)
}

/// Extract the token from an Authorization header
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_test_secret() {
        // For tests, set a deterministic secret
        JWT_SECRET.set("test-secret-for-unit-tests-only".to_string()).ok();
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_token_generation_and_validation() {
        ensure_test_secret();
        let token = generate_token("user123", "testuser", false).unwrap();
        let claims = validate_token(&token.token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert!(!claims.is_admin);
    }

    #[test]
    fn test_bearer_token_extraction() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(extract_bearer_token("abc123"), None);
        assert_eq!(extract_bearer_token(""), None);
    }
}
