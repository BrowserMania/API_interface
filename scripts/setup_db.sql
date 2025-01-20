-- Créer la base de données si elle n'existe pas
CREATE DATABASE IF NOT EXISTS brow2025;

-- Utiliser la base de données
USE brow2025;

-- Créer la table des rôles
CREATE TABLE IF NOT EXISTS roles (
    id INT AUTO_INCREMENT PRIMARY KEY, -- Identifiant unique pour chaque rôle
    name VARCHAR(255) NOT NULL UNIQUE -- Nom unique du rôle (par exemple, 'admin', 'user')
);

-- Créer la table des utilisateurs
CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY, -- Identifiant unique de l'utilisateur
    username VARCHAR(255) NOT NULL, -- Nom d'utilisateur
    email VARCHAR(255) NOT NULL UNIQUE, -- Adresse e-mail unique
    password VARCHAR(255) NOT NULL, -- Mot de passe haché
    role_id INT NOT NULL, -- ID du rôle (clé étrangère vers la table roles)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP, -- Date de création
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE -- Relation avec la table roles
);

-- Insérer les rôles par défaut
INSERT IGNORE INTO roles (id, name) VALUES 
(1, 'admin'), 
(2, 'user');

-- Insérer un utilisateur administrateur par défaut (mot de passe haché à ajuster)
INSERT IGNORE INTO users (username, email, password, role_id) VALUES 
('adminuser', 'admin8@example.com', '$2b$04$s4urY49Pr3RtiJxL6eBQ.ey/ssPxhxq/ciXiHHokr8AFWmnoQZYLO', 1),
('badr', 'badr.moussaoui2@gmail.com', '$2b$04$c.G3L4yuM2inFVjchU1UKOe6tHrsp2.og8R8Hwx/6Yr/ZYJeVkwBq', 2);

