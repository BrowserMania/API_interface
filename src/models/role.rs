use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Représente un rôle dans la base de données
#[derive(Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: i32,
    pub name: String,
}

/// Structure pour créer un rôle
#[derive(Serialize, Deserialize)]
pub struct CreateRoleForm {
    pub name: String,
}
