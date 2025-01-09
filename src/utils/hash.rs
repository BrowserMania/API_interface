use bcrypt::{hash, verify};

/// Hache un mot de passe en utilisant bcrypt
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, 4) // Utilise un coût de 4 pour un équilibre entre sécurité et performance
}

/// Vérifie si un mot de passe correspond à un hachage
pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    match verify(password, hashed_password) {
        Ok(is_valid) => is_valid,
        Err(_) => false,
    }
}
