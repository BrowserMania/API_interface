mod db;
mod models;
mod routes;
mod utils;
mod config;

use actix_cors::Cors; // Importation pour CORS
use actix_web::{http, App, HttpServer, web};
use config::Config;
use db::init_pool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Charger la configuration depuis .env
    let config = Config::from_env().expect("Erreur lors du chargement de la configuration");

    println!("Connexion à la base de données...");
    // Initialiser le pool de connexions
    let pool = init_pool().await.expect("Impossible de se connecter à la base de données");

    println!("Démarrage du serveur sur http://127.0.0.1:8080");
    // Démarrer le serveur Actix-web
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Partager le pool de connexions avec les handlers
            .wrap(
                Cors::default() // Configuration de CORS
                    .allowed_origin("http://localhost:3000") // Autoriser les requêtes du frontend React
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]) // Méthodes HTTP autorisées
                    .allowed_headers(vec![http::header::CONTENT_TYPE, http::header::AUTHORIZATION]) // En-têtes autorisés
                    .max_age(3600), // Durée de validité des pré-requêtes
            )
            .service(
                web::scope("/auth") // Ajoute un préfixe pour toutes les routes du module auth
                    .configure(routes::auth::config),
            )
            .configure(routes::admin::config) // Routes pour l'administration
    })
    .bind("127.0.0.1:8080")? // Démarrer le serveur sur le port 8080
    .run()
    .await
}