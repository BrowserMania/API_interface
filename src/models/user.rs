use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

/// Représente un utilisateur dans la base de données
#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,         // Ajout du champ username
    pub email: String,
    pub password: String,
    pub role_id: i32,
    pub created_at: DateTime<Utc>, // Nouveau type
}

/// Structure pour les requêtes d'inscription d'utilisateur
#[derive(Serialize, Deserialize)]
pub struct RegisterForm {
    pub username: String,         // Ajout du champ username
    pub email: String,
    pub password: String,
    pub access_code: Option<String>, // Champ optionnel pour le code d'accès admin
}

/// Structure pour les requêtes de connexion d'utilisateur
#[derive(Serialize, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}
