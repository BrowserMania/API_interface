mod db; 
mod models; 
mod routes; 
mod utils; 
mod config;

use actix_cors::Cors; 
use actix_web::{http, middleware, App, HttpServer, web}; 
use config::Config; 
use db::init_pool; 
use utils::extract_user::ExtractUser;

#[actix_web::main] 
async fn main() -> std::io::Result<()> {
    // Charger la configuration depuis le fichier .env
    let config = Config::from_env().expect("Erreur lors du chargement de la configuration");

    println!("Connexion à la base de données...");
    let pool = init_pool().await.expect("Impossible de se connecter à la base de données");

    // Garde le même message même si on change l'adresse d'écoute
    println!("Démarrage du serveur sur http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    // Autoriser multiple origines pour différents environnements
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("http://127.0.0.1:3000")
                    .allowed_origin("http://0.0.0.0:3000")
                    .allowed_origin("http://localhost")
                    .allowed_origin("http://127.0.0.1")
                    .allowed_origin("https://browsermania.fr")
                    .allowed_origin("http://browsermania.fr")
                    // Autoriser les requêtes depuis les conteneurs Docker
                    .allowed_origin_fn(|origin, _req_head| {
                        // Autoriser toutes les origines localhost et 127.0.0.1 sur tous les ports
                        origin.as_bytes().starts_with(b"http://localhost") ||
                        origin.as_bytes().starts_with(b"http://127.0.0.1") ||
                        origin.as_bytes().starts_with(b"http://0.0.0.0") ||
                        // Autoriser les origines de conteneurs Docker (généralement 172.x.x.x)
                        origin.as_bytes().starts_with(b"http://172.") ||
                        // Pour le développement, vous pouvez temporairement autoriser tout
                        true // ATTENTION: À supprimer en production !
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        http::header::CONTENT_TYPE,
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::ORIGIN,
                    ])
                    .supports_credentials() // Nécessaire pour les cookies et l'authentification
                    .max_age(3600),
            )
            // Appliquer le middleware pour extraire l'utilisateur
            .wrap(ExtractUser)
            .service(
                web::scope("/auth")
                    .configure(routes::auth::config),
            )
            .service(
                web::scope("/admin")
                    .configure(routes::admin::config),
            )
            .service(
                web::scope("/browser")
                    .configure(routes::browser::config),
            )
            .service(
                web::scope("/regle") // Nouvelles routes
                    .configure(routes::regle::config),
            )
    })
    .bind("0.0.0.0:8080")?  // Changez ceci pour 0.0.0.0 mais gardez l'URL 127.0.0.1 pour le frontend
    .run()
    .await
}