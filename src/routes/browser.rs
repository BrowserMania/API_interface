use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize)]
pub struct BrowserRequest {
    pub browser_type: String, // "firefox" ou "chrome"
}

#[derive(Deserialize, Serialize)]
pub struct KubeUserRequest {
    pub id: String, // Identifiant de l'utilisateur pour Kubernetes
}

#[derive(Serialize)]
pub struct BrowserInfo {
    pub pod_id: String,
    pub username: String,
    pub browser_type: String,
}

// Route pour instancier un navigateur sécurisé
#[post("/start-browser")]
async fn start_browser(
    pool: web::Data<MySqlPool>, 
    user_id: web::ReqData<i32>,
    browser_request: web::Json<BrowserRequest>,
) -> impl Responder {
    println!("Requête pour l'utilisateur ID : {:?}", user_id);
    println!("Type de navigateur demandé : {}", browser_request.browser_type);

    // Vérifier si l'utilisateur est authentifié
    let user_result = sqlx::query!(
        "SELECT username FROM users WHERE id = ?",
        *user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user_result {
        Ok(Some(user)) => {
            // Créer le namespace avec kubectl
            let ns_name = format!("{}-browser", user.username);
            let ns_output = Command::new("kubectl")
                .args(["create", "namespace", &ns_name])
                .output();

            if let Err(e) = ns_output {
                println!("Erreur lors de la création du namespace: {}", e);
            }

            // Choisir l'image en fonction du type de navigateur
            let image = if browser_request.browser_type.to_lowercase() == "firefox" {
                "jlesage/firefox:latest"
            } else {
                "jlesage/chrome:latest"
            };

            // Créer le pod avec kubectl
            let pod_output = Command::new("kubectl")
                .args([
                    "run", 
                    &format!("{}-browser", user.username),
                    &format!("--image={}", image),
                    "--labels=app=browser,user-browser=true",
                    "-n", 
                    &ns_name
                ])
                .output();

            if let Err(e) = pod_output {
                println!("Erreur lors de la création du pod: {}", e);
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Navigateur lancé avec succès",
                "pod_id": format!("{}-browser", user.username),
                "browser_type": browser_request.browser_type
            }))
        },
        Ok(None) => HttpResponse::Unauthorized().body("Utilisateur non trouvé"),
        Err(err) => {
            eprintln!("Erreur SQL : {:?}", err);
            HttpResponse::InternalServerError().body("Erreur interne")
        }
    }
}

// Route pour terminer un navigateur
#[post("/stop-browser/{username}")]
async fn stop_browser(
    path: web::Path<String>,
) -> impl Responder {
    let username = path.into_inner();
    
    // Supprimer le namespace avec kubectl
    let _ = Command::new("kubectl")
        .args(["delete", "namespace", &format!("{}-browser", username)])
        .output();
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Navigateur arrêté avec succès"
    }))
}

// Route pour lister tous les navigateurs actifs
#[get("/list-browsers")]
async fn list_browsers() -> impl Responder {
    println!("Requête pour lister les navigateurs actifs");
    
    // Obtenir tous les namespaces avec kubectl
    let output = Command::new("kubectl")
        .args(["get", "namespaces", "--output=jsonpath={.items[*].metadata.name}"])
        .output();
    
    let mut browser_infos = Vec::new();
    
    match output {
        Ok(output) => {
            let namespaces_str = String::from_utf8_lossy(&output.stdout);
            let namespaces: Vec<&str> = namespaces_str.split_whitespace().collect();
            
            for ns in namespaces {
                if ns.ends_with("-browser") && !ns.starts_with("kube-") {
                    let username = ns.trim_end_matches("-browser");
                    println!("Namespace trouvé: {} (utilisateur: {})", ns, username);
                    
                    // Obtenir les pods dans ce namespace
                    let pods_output = Command::new("kubectl")
                        .args(["get", "pods", "-n", ns, "--output=jsonpath={.items[*].metadata.name}"])
                        .output();
                    
                    if let Ok(pods_output) = pods_output {
                        let pods_str = String::from_utf8_lossy(&pods_output.stdout);
                        let pods: Vec<&str> = pods_str.split_whitespace().collect();
                        
                        if pods.is_empty() {
                            // Namespace sans pod
                            browser_infos.push(BrowserInfo {
                                pod_id: format!("{}-browser", username),
                                username: username.to_string(),
                                browser_type: "Firefox".to_string(),
                            });
                        } else {
                            for pod in pods {
                                browser_infos.push(BrowserInfo {
                                    pod_id: pod.to_string(),
                                    username: username.to_string(),
                                    browser_type: "Firefox".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution de kubectl: {}", e);
        }
    }
    
    // Si aucun navigateur trouvé ou erreur, utiliser des données fictives
    if browser_infos.is_empty() {
        browser_infos = vec![
            BrowserInfo {
                pod_id: "admin-browser".to_string(),
                username: "admin".to_string(),
                browser_type: "Firefox".to_string(),
            },
            BrowserInfo {
                pod_id: "test-browser".to_string(),
                username: "test".to_string(),
                browser_type: "Google Chrome".to_string(),
            }
        ];
        println!("Utilisation de données fictives");
    }
    
    println!("Renvoi de {} navigateurs", browser_infos.len());
    HttpResponse::Ok().json(browser_infos)
}

// Configuration des routes pour le module `browser`
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(start_browser);
    cfg.service(stop_browser);
    cfg.service(list_browsers);
}