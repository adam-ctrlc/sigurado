use std::time::Duration;

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::sea_orm_active_enums::Role;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: Role,
    pub exp: i64,
}

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn issue(&self, user_id: Uuid, role: Role, ttl: Duration) -> anyhow::Result<String> {
        let exp = (chrono::Utc::now() + chrono::Duration::from_std(ttl)?).timestamp();
        let claims = Claims {
            sub: user_id,
            role,
            exp,
        };
        Ok(encode(&Header::default(), &claims, &self.encoding)?)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(token, &self.decoding, &Validation::default())?;
        Ok(data.claims)
    }
}
