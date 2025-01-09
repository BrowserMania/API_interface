# Étape 1 : Construire l'application
FROM rust:1.72-slim as builder

# Définir le répertoire de travail
WORKDIR /usr/src/app

# Copier les fichiers Cargo.toml et Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Précharger les dépendances
RUN apt-get update && apt-get install -y libmariadb-dev && cargo fetch

# Copier le reste du projet
COPY . .

# Construire l'application en mode release
RUN cargo build --release

# Étape 2 : Préparer l'image finale
FROM debian:bullseye-slim

# Installer les dépendances nécessaires
RUN apt-get update && apt-get install -y \
    libssl-dev \
    libmariadb3 \
    && rm -rf /var/lib/apt/lists/*

# Définir le répertoire de travail
WORKDIR /usr/src/app

# Copier le fichier binaire depuis l'étape de build
COPY --from=builder /usr/src/app/target/release/api /usr/src/app/

# Copier le fichier `.env`
COPY .env /usr/src/app/.env

# Exposer le port d'écoute (par défaut 8080 pour votre application)
EXPOSE 8080

# Définir le point d'entrée
CMD ["./api"]
