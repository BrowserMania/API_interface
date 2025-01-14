use std::env;
use dotenvy::dotenv;


/// Structure pour stocker la configuration de l'application
#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub admin_secret_code: String,
}

impl Config {
    /// Charge les variables d'environnement et initialise la configuration
    pub fn from_env() -> Result<Self, String> {
        // Charge le fichier `.env`
        dotenv().ok();

        // Charge les variables ou retourne une erreur explicite si elles ne sont pas définies
        let database_url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL n'est pas défini dans .env")?;
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET n'est pas défini dans .env")?;
        let admin_secret_code = env::var("ADMIN_SECRET_CODE").map_err(|_| "ADMIN_SECRET_CODE n'est pas défini dans .env")?;

        // Retourne une instance de Config
        Ok(Self {
            database_url,
            jwt_secret,
            admin_secret_code,
        })
    }
}
