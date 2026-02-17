use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const JWT_SECRET: &str = "citinet-secret-key-change-in-production"; // TODO: Make configurable
const TOKEN_EXPIRATION_HOURS: i64 = 24 * 7; // 7 days

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
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
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
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
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

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_token_generation_and_validation() {
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
