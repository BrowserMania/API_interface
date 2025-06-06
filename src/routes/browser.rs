use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, http};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use crate::k8s;

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
    pub browser_url: Option<String>, // URL pour accéder au navigateur
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
            // Utiliser la nouvelle fonction k8s::deploy
            match k8s::deploy(&user.username).await {
                Ok(_) => {
                    println!("Déploiement réussi, attente de l'IP LoadBalancer...");
                    
                    // Attendre que le LoadBalancer soit prêt et récupérer l'IP
                    match k8s::wait_for_loadbalancer(&user.username, 12).await { // 12 tentatives = 1 minute
                        Ok(loadbalancer_ip) => {
                            let browser_url = Some(format!("http://{}:3000/", loadbalancer_ip));
                            
                            println!("LoadBalancer IP récupérée: {}", loadbalancer_ip);
                            
                            HttpResponse::Ok().json(serde_json::json!({
                                "message": "Navigateur lancé avec succès",
                                "pod_id": format!("{}-browser", user.username),
                                "browser_type": browser_request.browser_type,
                                "browser_url": browser_url,
                                "loadbalancer_ip": loadbalancer_ip,
                                "access_url": format!("/browser/access-browser/{}", user.username)
                            }))
                        },
                        Err(e) => {
                            eprintln!("Erreur lors de la récupération de l'IP LoadBalancer: {}", e);
                            
                            // Fallback sur l'ancienne URL si le LoadBalancer n'est pas prêt
                            let browser_url = Some("http://10.10.32.153:3000/".to_string());
                            
                            HttpResponse::Ok().json(serde_json::json!({
                                "message": "Navigateur lancé avec succès (LoadBalancer en cours de préparation)",
                                "pod_id": format!("{}-browser", user.username),
                                "browser_type": browser_request.browser_type,
                                "browser_url": browser_url,
                                "warning": "LoadBalancer IP non disponible, utilisation de l'IP par défaut",
                                "access_url": format!("/browser/access-browser/{}", user.username)
                            }))
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Erreur lors du déploiement Kubernetes: {}", e);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Erreur lors du déploiement du navigateur",
                        "details": e.to_string()
                    }))
                }
            }
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
    
    // Utiliser la nouvelle fonction k8s::cleanup au lieu de kubectl
    match k8s::cleanup(&username).await {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Navigateur arrêté avec succès"
            }))
        },
        Err(e) => {
            eprintln!("Erreur lors de la suppression Kubernetes: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Erreur lors de l'arrêt du navigateur",
                "details": e.to_string()
            }))
        }
    }
}

// Route pour lister tous les navigateurs actifs
#[get("/list-browsers")]
async fn list_browsers() -> impl Responder {
    println!("Requête pour lister les navigateurs actifs");
    
    // Utiliser la nouvelle fonction k8s::list_deployments
    match k8s::list_deployments().await {
        Ok(namespaces) => {
            let mut browser_infos = Vec::new();
            
            for ns in namespaces {
                if ns.ends_with("-browser") && !ns.starts_with("kube-") {
                    let username = ns.trim_end_matches("-browser");
                    println!("Namespace trouvé: {} (utilisateur: {})", ns, username);
                    
                    // Essayer de récupérer l'IP du LoadBalancer pour chaque navigateur actif
                    let browser_url = match k8s::get_loadbalancer_ip(username).await {
                        Ok(ip) => {
                            println!("LoadBalancer IP trouvée pour {}: {}", username, ip);
                            Some(format!("http://{}:3000/", ip))
                        },
                        Err(e) => {
                            println!("Pas d'IP LoadBalancer pour {}: {}, utilisation de l'IP par défaut", username, e);
                            Some("http://10.10.32.153:3000/".to_string())
                        }
                    };
                    
                    browser_infos.push(BrowserInfo {
                        pod_id: ns.clone(),
                        username: username.to_string(),
                        browser_type: "Chrome".to_string(),
                        browser_url,
                    });
                }
            }
            
            // Si aucun navigateur trouvé, utiliser des données fictives
            if browser_infos.is_empty() {
                browser_infos = vec![
                    BrowserInfo {
                        pod_id: "admin-browser".to_string(),
                        username: "admin".to_string(),
                        browser_type: "Chrome".to_string(),
                        browser_url: Some("http://10.10.32.153:3000/".to_string()),
                    },
                    BrowserInfo {
                        pod_id: "test-browser".to_string(),
                        username: "test".to_string(),
                        browser_type: "Chrome".to_string(),
                        browser_url: Some("http://10.10.32.153:3000/".to_string()),
                    }
                ];
                println!("Utilisation de données fictives");
            }
            
            println!("Renvoi de {} navigateurs", browser_infos.len());
            HttpResponse::Ok().json(browser_infos)
        },
        Err(e) => {
            eprintln!("Erreur lors de la récupération des namespaces: {}", e);
            
            // En cas d'erreur, renvoyer des données fictives
            let browser_infos = vec![
                BrowserInfo {
                    pod_id: "admin-browser".to_string(),
                    username: "admin".to_string(),
                    browser_type: "Chrome".to_string(),
                    browser_url: Some("http://10.10.32.153:3000/".to_string()),
                },
                BrowserInfo {
                    pod_id: "test-browser".to_string(),
                    username: "test".to_string(),
                    browser_type: "Chrome".to_string(),
                    browser_url: Some("http://10.10.32.153:3000/".to_string()),
                }
            ];
            
            HttpResponse::Ok().json(browser_infos)
        }
    }
}

// Route pour obtenir l'IP du LoadBalancer d'un utilisateur spécifique
#[get("/get-browser-ip/{username}")]
async fn get_browser_ip(
    path: web::Path<String>,
) -> impl Responder {
    let username = path.into_inner();
    
    match k8s::get_loadbalancer_ip(&username).await {
        Ok(ip) => {
            HttpResponse::Ok().json(serde_json::json!({
                "username": username,
                "loadbalancer_ip": ip,
                "browser_url": format!("http://{}:3000/", ip)
            }))
        },
        Err(e) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "LoadBalancer IP non trouvée",
                "username": username,
                "details": e.to_string()
            }))
        }
    }
}

// Route pour accéder à l'interface du navigateur (mise à jour pour utiliser le LoadBalancer)
#[get("/access-browser/{username}")]
async fn access_browser(
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let username = path.into_inner();
    
    // Essayer de récupérer l'IP du LoadBalancer
    let browser_url = match k8s::get_loadbalancer_ip(&username).await {
        Ok(ip) => {
            println!("Utilisation de l'IP LoadBalancer pour {}: {}", username, ip);
            format!("http://{}:3000/", ip)
        },
        Err(e) => {
            println!("LoadBalancer IP non disponible pour {}: {}, utilisation de l'IP par défaut", username, e);
            "http://10.10.32.153:3000/".to_string()
        }
    };
    
    // Créer un HTML qui intègre le navigateur dans un iframe
    let html = format!(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Navigateur Sécurisé - {}</title>
        <style>
            body, html {{
                margin: 0;
                padding: 0;
                height: 100%;
                overflow: hidden;
            }}
            .browser-container {{
                width: 100%;
                height: 100vh;
                border: none;
            }}
            .info-banner {{
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                background: #007bff;
                color: white;
                padding: 5px 10px;
                font-size: 12px;
                z-index: 1000;
            }}
        </style>
    </head>
    <body>
        <div class="info-banner">
            Navigateur sécurisé pour: {} | URL: {}
        </div>
        <iframe src="{}" class="browser-container" allowfullscreen="true" style="margin-top: 25px; height: calc(100vh - 25px);"></iframe>
    </body>
    </html>
    "#, username, username, browser_url, browser_url);
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

// Configuration des routes pour le module `browser`
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(start_browser);
    cfg.service(stop_browser);
    cfg.service(list_browsers);
    cfg.service(get_browser_ip); // Nouvelle route
    cfg.service(access_browser);
}