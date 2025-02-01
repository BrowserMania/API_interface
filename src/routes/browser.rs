use actix_web::{post, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use serde::Deserialize; // Pour décoder la requête JSON

#[derive(Deserialize)]
pub struct BrowserRequest {
    pub id: String, // Identifiant unique pour le navigateur
}

/// Route pour instancier un navigateur sécurisé
#[post("/start-browser")]
async fn start_browser(
    pool: web::Data<MySqlPool>, 
    user_id: web::ReqData<i32>, // Injecté par le middleware pour récupérer l'utilisateur connecté
    browser_request: web::Json<BrowserRequest>, // Décoder la requête JSON
) -> impl Responder {
    println!("Requête pour l'utilisateur ID : {:?}", user_id);
    println!("ID unique reçu pour le navigateur : {}", browser_request.id);

    // Vérifiez si l'utilisateur est authentifié dans la base de données
    let user_exists = sqlx::query!(
        "SELECT id FROM users WHERE id = ?",
        *user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user_exists {
        Ok(Some(_)) => {
            // Simule le démarrage du navigateur
            if deploy_browser_instance(&browser_request.id).await {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Navigateur lancé"
                }))
            } else {
                HttpResponse::InternalServerError().body("Erreur lors du démarrage du navigateur")
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
    true // Simule un succès
}

/// Configuration des routes pour le module `browser`
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(start_browser);
}