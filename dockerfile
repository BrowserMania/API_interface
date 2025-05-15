# Étape 1: Construction avec Rust
FROM rust:slim AS builder

# Installation des dépendances de build
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    default-libmysqlclient-dev \
    && rm -rf /var/lib/apt/lists/*

# Configuration du build
WORKDIR /app
ENV SQLX_OFFLINE=true

# Copie des fichiers de dépendances
COPY Cargo.toml Cargo.lock ./
COPY .sqlx/ ./.sqlx/

# Copie du code source
COPY src/ ./src/
COPY scripts/ ./scripts/

# Compilation en mode release
RUN cargo build --release

# Étape 2: Une approche plus simple - utiliser la même image que l'étape de build
FROM rust:slim

# Installation des dépendances minimales
RUN apt-get update && apt-get install -y \
    libssl3 \
    default-mysql-client-core \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Installation de kubectl
RUN curl -LO "https://dl.k8s.io/release/stable.txt" && \
    KUBECTL_VERSION=$(cat stable.txt) && \
    curl -LO "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl" && \
    chmod +x kubectl && \
    mv kubectl /usr/local/bin/ && \
    rm stable.txt

# Préparation de l'environnement d'exécution
WORKDIR /app
RUN mkdir -p temp /root/.kube

# Copie des fichiers nécessaires uniquement
COPY --from=builder /app/target/release/api /app/api
COPY .env /app/.env
COPY --from=builder /app/scripts/ /app/scripts/

# Configuration
EXPOSE 8080
ENV KUBECONFIG=/root/.kube/config
ENV SQLX_OFFLINE=true

# Commande de démarrage
CMD ["/app/api"]