# Étape 1 : Construction du binaire Rust
FROM rust:1.72 AS builder

# Crée un utilisateur non-root
RUN useradd -m -s /bin/bash appuser

# Définit le répertoire de travail
WORKDIR /app

# Copie les fichiers du projet dans l'image
COPY . .

# Compile le projet en mode release
RUN cargo build --release

# Étape 2 : Création de l'image finale
FROM debian:bullseye-slim

# Installation des dépendances nécessaires pour exécuter le binaire
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copie du binaire compilé depuis l'étape précédente
COPY --from=builder /app/target/release/api /usr/local/bin/api

# Définit un utilisateur non-root pour exécuter le binaire
USER appuser

# Définit le répertoire de travail
WORKDIR /home/appuser

# Expose le port utilisé par l'application
EXPOSE 8080

# Commande par défaut pour démarrer l'application
CMD ["api"]
