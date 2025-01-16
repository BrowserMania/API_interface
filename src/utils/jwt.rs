use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

/// Structure des claims d'un JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Sujet (l'identifiant de l'utilisateur, email ou autre)
    pub exp: usize,  // Date d'expiration en timestamp UNIX
}

/// Crée un JWT pour un utilisateur
pub fn create_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET doit être défini");
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .expect("Erreur lors du calcul de l'expiration")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(), // Convertir l'ID utilisateur en chaîne
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}


/// Valide un JWT et renvoie les claims s'ils sont valides
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET doit être défini");

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
