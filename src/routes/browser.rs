use actix_web::{post, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use crate::models::user::User;

/// Route pour instancier un navigateur sécurisé
#[post("/start-browser")]
async fn start_browser(
    pool: web::Data<MySqlPool>, 
    user_id: web::ReqData<i32>, // Injecté par le middleware
) -> impl Responder {
    println!("Requête pour l'utilisateur ID : {:?}", user_id);

    // Vérifiez si l'utilisateur existe
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, password, role_id, created_at FROM users WHERE id = ?",
        *user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            let browser_id = format!("browser-{}", user.id);

            if deploy_browser_instance(&browser_id).await {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Navigateur sécurisé lancé avec succès",
                    "browser_id": browser_id,
                    "webrtc_url": format!("https://Moussaoui.com/{}", browser_id)
                }))
            } else {
                HttpResponse::InternalServerError().body("Erreur lors du déploiement du navigateur")
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Utilisateur non trouvé"),
        Err(err) => {
            eprintln!("Erreur SQL : {:?}", err);
            HttpResponse::InternalServerError().body("Erreur interne")
        }
    }
}

/// Simule le déploiement d'un navigateur sécurisé
async fn deploy_browser_instance(browser_id: &str) -> bool {
    println!("Déploiement d'une instance de navigateur sécurisé : {}", browser_id);
    true
}

/// Configuration des routes pour le module `browser`
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(start_browser);
}
