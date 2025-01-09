use sqlx::MySqlPool;
use std::env;

/// Initialise une connexion à la base de données
pub async fn init_pool() -> Result<MySqlPool, sqlx::Error> {
    // Charger l'URL de la base de données depuis les variables d'environnement
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL doit être défini");
    MySqlPool::connect(&database_url).await
}
