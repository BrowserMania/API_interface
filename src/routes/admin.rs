use actix_web::{get, post, put, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::user::{User, RegisterForm};
use crate::utils::hash;

/// Route pour obtenir la liste des utilisateurs (accessible uniquement aux administrateurs)
#[get("/users")]
async fn list_users(pool: web::Data<MySqlPool>) -> impl Responder {
    match sqlx::query_as::<_, User>("SELECT id, username, email, password, role_id, created_at FROM users")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => {
            eprintln!("Erreur SQL : {:?}", err); // Ajout des logs pour l'erreur SQL
            HttpResponse::InternalServerError().body("Erreur lors de la récupération des utilisateurs")
        }
    }
}


/// Route pour ajouter un utilisateur en tant qu'administrateur
#[post("/create-user")]
async fn create_user_as_admin(
    pool: web::Data<MySqlPool>,
    form: web::Json<RegisterForm>,
) -> impl Responder {
    // Vérifier si l'email est déjà utilisé
    if let Ok(Some(_)) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&form.email)
        .fetch_optional(pool.get_ref())
        .await
    {
        return HttpResponse::BadRequest().body("Email déjà utilisé");
    }

    // Hacher le mot de passe
    let hashed_password = match hash::hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Erreur lors du hachage du mot de passe"),
    };

    // Déterminer le rôle en fonction du code d'accès fourni
    let role_id = if let Some(ref code) = form.access_code {
        
        if code == &std::env::var("ADMIN_SECRET_CODE").expect("ADMIN_SECRET_CODE doit être défini") {

            1 // Rôle admin
        } else {
            2 // Rôle utilisateur
        }
    } else {
        2 // Rôle utilisateur par défaut
    };

    // Insérer l'utilisateur dans la base de données
    match sqlx::query!(
        "INSERT INTO users (username, email, password, role_id) VALUES (?, ?, ?, ?)",
        form.username,
        form.email,
        hashed_password,
        role_id
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Utilisateur créé avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la création de l'utilisateur"),
    }
}

/// Route pour mettre à jour les informations d'un utilisateur
#[put("/update-user/{id}")]
async fn update_user_as_admin(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    form: web::Json<RegisterForm>,
) -> impl Responder {
    let user_id = path.into_inner();

    // Hacher le nouveau mot de passe (si fourni)
    let hashed_password = match hash::hash_password(&form.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().body("Erreur lors du hachage du mot de passe"),
    };

    // Mettre à jour les informations de l'utilisateur
    match sqlx::query!(
        "UPDATE users SET username = ?, email = ?, password = ?, role_id = ? WHERE id = ?",
        form.username,
        form.email,
        hashed_password,
        if let Some(ref code) = form.access_code {
            if code == &std::env::var("ADMIN_SECRET_CODE").expect("ADMIN_SECRET_CODE doit être défini") {

                1 // Rôle admin
            } else {
                2 // Rôle utilisateur
            }
        } else {
            2 // Rôle utilisateur par défaut
        },
        user_id
        
    )
    
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().body("Utilisateur mis à jour avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la mise à jour de l'utilisateur"),
    }
    
}




/// Route pour supprimer un utilisateur (accessible uniquement aux administrateurs)
#[post("/delete-user/{id}")]
async fn delete_user_as_admin(
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> impl Responder {
    let user_id = path.into_inner();

    // Supprimer l'utilisateur de la base de données
    match sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(pool.get_ref())
        .await
    {
        Ok(_) => HttpResponse::Ok().body("Utilisateur supprimé avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la suppression de l'utilisateur"),
    }
}

/// Configuration des routes spécifiques aux administrateurs
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users);
    cfg.service(create_user_as_admin);
    cfg.service(update_user_as_admin); // Ajouter la route de mise à jour
    cfg.service(delete_user_as_admin);
}
