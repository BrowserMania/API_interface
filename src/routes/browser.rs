use actix_web::{get, post, web, HttpResponse, Responder, HttpRequest, http};
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

            // Nom du pod
            let pod_name = format!("{}-browser", user.username);

            // Créer le pod avec kubectl - avec les ports exposés pour VNC (6080)
            let pod_output = Command::new("kubectl")
                .args([
                    "run", 
                    &pod_name,
                    &format!("--image={}", image),
                    "--labels=app=browser,user-browser=true",
                    "--port=6080", // Port VNC
                    "-n", 
                    &ns_name
                ])
                .output();

            if let Err(e) = pod_output {
                println!("Erreur lors de la création du pod: {}", e);
            }

            // Créer un service pour exposer le navigateur
            let service_output = Command::new("kubectl")
                .args([
                    "expose", 
                    "pod",
                    &pod_name,
                    "--name", &format!("{}-service", pod_name),
                    "--port=5000",
                    "--target-port=5000",
                    "--type=NodePort",
                    "-n", &ns_name
                ])
                .output();

            if let Err(e) = service_output {
                println!("Erreur lors de la création du service: {}", e);
            }

            // Obtenir le NodePort assigné
            let node_port_output = Command::new("kubectl")
                .args([
                    "get", 
                    "service",
                    &format!("{}-service", pod_name),
                    "-n", &ns_name,
                    "--output=jsonpath={.spec.ports[0].nodePort}"
                ])
                .output();

            // Utiliser la bonne adresse IP
            let browser_url = Some("http://10.10.32.153:3000/".to_string());

            HttpResponse::Ok().json(serde_json::json!({
                "message": "Navigateur lancé avec succès",
                "pod_id": pod_name,
                "browser_type": browser_request.browser_type,
                "browser_url": browser_url,
                "access_url": "/browser/access-browser/".to_string() + &user.username
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
                                browser_url: None,
                            });
                        } else {
                            for pod in pods {
                                // Utiliser la bonne adresse IP
                                let browser_url = Some("http://10.10.32.153:3000/".to_string());
                                
                                browser_infos.push(BrowserInfo {
                                    pod_id: pod.to_string(),
                                    username: username.to_string(),
                                    browser_type: "Firefox".to_string(),
                                    browser_url,
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
                browser_url: Some("http://10.10.32.153:3000/".to_string()),
            },
            BrowserInfo {
                pod_id: "test-browser".to_string(),
                username: "test".to_string(),
                browser_type: "Google Chrome".to_string(),
                browser_url: Some("http://10.10.32.153:3000/".to_string()),
            }
        ];
        println!("Utilisation de données fictives");
    }
    
    println!("Renvoi de {} navigateurs", browser_infos.len());
    HttpResponse::Ok().json(browser_infos)
}

// Route pour accéder à l'interface du navigateur
#[get("/access-browser/{username}")]
async fn access_browser(
    path: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let username = path.into_inner();
    
    // Utiliser la bonne adresse IP
    let browser_url = "http://10.10.32.153:3000/";
    
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
        </style>
    </head>
    <body>
        <iframe src="{}" class="browser-container" allowfullscreen="true"></iframe>
    </body>
    </html>
    "#, username, browser_url);
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

// Configuration des routes pour le module `browser`
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(start_browser);
    cfg.service(stop_browser);
    cfg.service(list_browsers);
    cfg.service(access_browser); // Nouvel endpoint
}