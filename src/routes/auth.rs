use actix_web::{post, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::user::{RegisterForm, LoginForm, User};
use crate::utils::{hash, jwt};


/// Route pour l'inscription des utilisateurs
#[post("/register")]
async fn register(
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

    // Déterminer le rôle en fonction du code d'accès
    let role_id = if let Some(ref code) = form.access_code {
        if code.trim() == std::env::var("ADMIN_SECRET_CODE").expect("ADMIN_SECRET_CODE doit être défini").trim() {

            1 // Rôle admin
        } else {
            2 // Rôle utilisateur
        }
    } else {
        2 // Rôle utilisateur par défaut
    };

    // Insérer le nouvel utilisateur dans la base de données
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
        Ok(_) => HttpResponse::Ok().body("Inscription réussie"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de l'inscription"),
    }
}

/// Route pour la connexion des utilisateurs
#[post("/login")]
async fn login(
    pool: web::Data<MySqlPool>,
    form: web::Json<LoginForm>,
) -> impl Responder {
    // Rechercher l'utilisateur par email
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&form.email)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().body("Identifiants invalides"),
        Err(_) => return HttpResponse::InternalServerError().body("Erreur lors de la récupération de l'utilisateur"),
    };

    // Vérifier le mot de passe
    if !hash::verify_password(&form.password, &user.password) {
        return HttpResponse::Unauthorized().body("Identifiants invalides");
    }

    // Rechercher le nom du rôle dans la table `roles` en fonction de `role_id`
    let role = match sqlx::query_scalar::<_, String>("SELECT name FROM roles WHERE id = ?")
        .bind(user.role_id)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(role_name)) => role_name,
        Ok(None) => return HttpResponse::InternalServerError().body("Rôle non trouvé"),
        Err(_) => return HttpResponse::InternalServerError().body("Erreur lors de la récupération du rôle"),
    };

    // Générer un token JWT
    let token = match jwt::create_token(user.id) { // Passer l'ID utilisateur ici
    Ok(token) => token,
    Err(_) => return HttpResponse::InternalServerError().body("Erreur lors de la génération du token"),
};


    // Retourner à la fois le token, le username et le rôle
    HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "username": user.username,
        "role": role
    }))
}


/// Configuration des routes d'authentification
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
    cfg.service(login);
}
