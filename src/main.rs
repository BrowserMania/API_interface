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

    println!("Démarrage du serveur sur http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        http::header::CONTENT_TYPE,
                        http::header::AUTHORIZATION,
                    ])
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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}