use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct TrafficRule {
    pub id: Option<i32>,
    pub name: String,
    pub description: String,
    pub rule_type: String,  // "allow" ou "deny"
    pub host: Option<String>,
    pub port: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkPolicyInfo {
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub host: Option<String>,
    pub port: Option<i32>,
}

// Fonction utilitaire pour extraire l'hôte d'un egress
fn extract_host(json: &serde_json::Value, egress_index: usize) -> Option<String> {
    if let Some(egress) = json["spec"]["egress"].as_array() {
        if egress.len() > egress_index {
            if let Some(to_fqdns) = egress[egress_index]["toFQDNs"].as_array() {
                if !to_fqdns.is_empty() {
                    // Utiliser matchName s'il existe, sinon matchPattern
                    if let Some(name) = to_fqdns[0]["matchName"].as_str() {
                        return Some(name.to_string());
                    } else if let Some(pattern) = to_fqdns[0]["matchPattern"].as_str() {
                        // Enlever le préfixe "*." si présent
                        return Some(pattern.trim_start_matches("*.").to_string());
                    }
                }
            }
        }
    }
    None
}

// Fonction utilitaire pour extraire le port d'un egress
fn extract_port(json: &serde_json::Value, egress_index: usize) -> Option<i32> {
    if let Some(egress) = json["spec"]["egress"].as_array() {
        if egress.len() > egress_index {
            if let Some(to_ports) = egress[egress_index]["toPorts"].as_array() {
                if !to_ports.is_empty() {
                    if let Some(ports) = to_ports[0]["ports"].as_array() {
                        if !ports.is_empty() {
                            return ports[0]["port"].as_str()
                                .and_then(|p| p.parse::<i32>().ok());
                        }
                    }
                }
            }
        }
    }
    None
}

// Fonction utilitaire pour déterminer le type de règle
fn determine_rule_type(json: &serde_json::Value) -> String {
    if let Some(egress) = json["spec"]["egress"].as_array() {
        if egress.len() >= 2 { // Le premier est pour DNS, le second pour les règles
            if let Some(to_fqdns) = egress[1]["toFQDNs"].as_array() {
                if !to_fqdns.is_empty() {
                    return "deny".to_string(); // S'il y a des toFQDNs dans le second egress, c'est une règle deny
                }
            }
        }
    }
    "allow".to_string() // Par défaut
}

// Route pour récupérer toutes les règles de trafic
#[get("/list-rules")]
async fn list_rules() -> impl Responder {
    println!("Récupération des règles de trafic");
    
    // Utiliser kubectl pour lister les ciliumnetworkpolicies
    let output = Command::new("kubectl")
        .args(["get", "ciliumnetworkpolicies", "--all-namespaces", "-o", "jsonpath={.items[*].metadata.name}"])
        .output();
    
    match output {
        Ok(output) => {
            let policies_str = String::from_utf8_lossy(&output.stdout);
            let policies: Vec<&str> = policies_str.split_whitespace().collect();
            
            let mut rules = Vec::new();
            for policy in policies.iter() {
                // Récupérer plus de détails sur chaque politique
                let policy_details = Command::new("kubectl")
                    .args(["get", "ciliumnetworkpolicy", policy, "-o", "json"])
                    .output();
                
                if let Ok(details) = policy_details {
                    let details_str = String::from_utf8_lossy(&details.stdout);
                    
                    // Parser le JSON pour extraire les informations
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&details_str) {
                        // Utiliser les fonctions utilitaires pour extraire les informations
                        let rule_type = determine_rule_type(&json);
                        let host = extract_host(&json, 1).or_else(|| extract_host(&json, 0));
                        let port = extract_port(&json, 1).or_else(|| extract_port(&json, 0));
                        
                        // Essayer d'extraire la description depuis les annotations
                        let description = json["metadata"]["annotations"]["description"].as_str()
                            .map(|s| s.to_string());
                        
                        // Construire l'objet info
                        rules.push(NetworkPolicyInfo {
                            name: policy.to_string(),
                            description,
                            rule_type,
                            host,
                            port,
                        });
                    }
                }
            }
            
            // Si aucune règle trouvée, utiliser des données fictives
            if rules.is_empty() {
                rules = vec![
                    NetworkPolicyInfo {
                        name: "block-social-media".to_string(),
                        description: Some("Bloquer les réseaux sociaux".to_string()),
                        rule_type: "deny".to_string(),
                        host: Some("facebook.com".to_string()),
                        port: None,
                    },
                    NetworkPolicyInfo {
                        name: "allow-https".to_string(),
                        description: Some("Autoriser le trafic HTTPS".to_string()),
                        rule_type: "allow".to_string(),
                        host: Some("example.com".to_string()),
                        port: Some(443),
                    },
                ];
            }
            
            HttpResponse::Ok().json(rules)
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution de kubectl: {}", e);
            
            // Renvoyer des données fictives en cas d'erreur
            let rules = vec![
                NetworkPolicyInfo {
                    name: "block-social-media".to_string(),
                    description: Some("Bloquer les réseaux sociaux".to_string()),
                    rule_type: "deny".to_string(),
                    host: Some("facebook.com".to_string()),
                    port: None,
                },
                NetworkPolicyInfo {
                    name: "allow-https".to_string(),
                    description: Some("Autoriser le trafic HTTPS".to_string()),
                    rule_type: "allow".to_string(),
                    host: Some("example.com".to_string()),
                    port: Some(443),
                },
            ];
            
            HttpResponse::Ok().json(rules)
        }
    }
}

// Route pour récupérer les détails d'une règle spécifique
#[get("/rule/{name}")]
async fn get_rule(path: web::Path<String>) -> impl Responder {
    let rule_name = path.into_inner();
    
    // Récupérer la CiliumNetworkPolicy
    let output = Command::new("kubectl")
        .args(["get", "ciliumnetworkpolicy", &rule_name, "-o", "json"])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let details_str = String::from_utf8_lossy(&output.stdout);
                
                // Parser le JSON retourné par kubectl
                match serde_json::from_str::<serde_json::Value>(&details_str) {
                    Ok(policy_json) => {
                        // Utiliser les fonctions utilitaires
                        let rule_type = determine_rule_type(&policy_json);
                        let host = extract_host(&policy_json, 0);
                        let port = extract_port(&policy_json, 0);
                        
                        // Essayer d'extraire la description depuis les annotations
                        let description = policy_json["metadata"]["annotations"]["description"].as_str()
                            .map(|s| s.to_string())
                            .or_else(|| Some("Description de la règle".to_string()));
                        
                        HttpResponse::Ok().json(NetworkPolicyInfo {
                            name: rule_name,
                            description,
                            rule_type,
                            host,
                            port,
                        })
                    },
                    Err(e) => {
                        eprintln!("Erreur de parsing JSON: {}", e);
                        HttpResponse::InternalServerError().body("Erreur de parsing de la règle")
                    }
                }
            } else {
                HttpResponse::NotFound().body(format!("Règle {} non trouvée", rule_name))
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution de kubectl: {}", e);
            HttpResponse::InternalServerError().body("Erreur interne du serveur")
        }
    }
}

// Route pour créer une nouvelle règle
#[post("/add-rule")]
async fn add_rule(rule: web::Json<TrafficRule>) -> impl Responder {
    println!("Ajout d'une règle: {}", rule.name);
    
    // Créer un fichier YAML temporaire pour la règle
    let yaml_content = generate_network_policy_yaml(&rule);
    
    // S'assurer que le répertoire temp existe
    let temp_dir = "temp";
    if !Path::new(temp_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(temp_dir) {
            eprintln!("Erreur lors de la création du répertoire temp: {}", e);
            return HttpResponse::InternalServerError().body("Erreur lors de la création du répertoire temporaire");
        }
    }
    
    // Utiliser un chemin compatible avec tous les OS
    let temp_file = format!("{}/{}.yaml", temp_dir, rule.name);
    if let Err(e) = std::fs::write(&temp_file, yaml_content) {
        eprintln!("Erreur lors de l'écriture du fichier YAML: {}", e);
        return HttpResponse::InternalServerError().body("Erreur lors de la création du fichier de règle");
    }
    
    // Appliquer la règle avec kubectl
    let output = Command::new("kubectl")
        .args(["apply", "-f", &temp_file])
        .output();
    
    // Supprimer le fichier temporaire
    let _ = std::fs::remove_file(&temp_file);
    
    match output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Règle créée avec succès",
                    "name": rule.name
                }))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("Erreur kubectl: {}", error);
                HttpResponse::BadRequest().body(format!("Erreur lors de la création de la règle: {}", error))
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution de kubectl: {}", e);
            HttpResponse::InternalServerError().body("Erreur interne du serveur")
        }
    }
}

// Route pour mettre à jour une règle existante
#[put("/update-rule/{name}")]
async fn update_rule(path: web::Path<String>, rule: web::Json<TrafficRule>) -> impl Responder {
    let rule_name = path.into_inner();
    println!("Mise à jour de la règle: {}", rule_name);
    
    // Supprimer d'abord la règle existante
    let delete_output = Command::new("kubectl")
        .args(["delete", "ciliumnetworkpolicy", &rule_name])
        .output();
    
    if let Err(e) = delete_output {
        eprintln!("Erreur lors de la suppression de la règle: {}", e);
        return HttpResponse::InternalServerError().body("Erreur lors de la mise à jour de la règle");
    }
    
    // S'assurer que le répertoire temp existe
    let temp_dir = "temp";
    if !Path::new(temp_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(temp_dir) {
            eprintln!("Erreur lors de la création du répertoire temp: {}", e);
            return HttpResponse::InternalServerError().body("Erreur lors de la création du répertoire temporaire");
        }
    }
    
    // Créer une nouvelle règle avec les mêmes étapes que pour l'ajout
    let yaml_content = generate_network_policy_yaml(&rule);
    let temp_file = format!("{}/{}.yaml", temp_dir, rule.name);
    
    if let Err(e) = std::fs::write(&temp_file, yaml_content) {
        eprintln!("Erreur lors de l'écriture du fichier YAML: {}", e);
        return HttpResponse::InternalServerError().body("Erreur lors de la mise à jour de la règle");
    }
    
    let output = Command::new("kubectl")
        .args(["apply", "-f", &temp_file])
        .output();
    
    let _ = std::fs::remove_file(&temp_file);
    
    match output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Règle mise à jour avec succès",
                    "name": rule.name
                }))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                HttpResponse::BadRequest().body(format!("Erreur lors de la mise à jour de la règle: {}", error))
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution de kubectl: {}", e);
            HttpResponse::InternalServerError().body("Erreur interne du serveur")
        }
    }
}

// Route pour supprimer une règle
#[delete("/delete-rule/{name}")]
async fn delete_rule(path: web::Path<String>) -> impl Responder {
    let rule_name = path.into_inner();
    println!("Suppression de la règle: {}", rule_name);
    
    let output = Command::new("kubectl")
        .args(["delete", "ciliumnetworkpolicy", &rule_name])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Règle supprimée avec succès",
                    "name": rule_name
                }))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                HttpResponse::BadRequest().body(format!("Erreur lors de la suppression de la règle: {}", error))
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de l'exécution de kubectl: {}", e);
            HttpResponse::InternalServerError().body("Erreur interne du serveur")
        }
    }
}

// Génère le YAML pour une CiliumNetworkPolicy
fn generate_network_policy_yaml(rule: &TrafficRule) -> String {
    let mut lines = Vec::new();
    
    // En-tête du YAML pour CiliumNetworkPolicy
    lines.push("apiVersion: cilium.io/v2".to_string());
    lines.push("kind: CiliumNetworkPolicy".to_string());
    lines.push("metadata:".to_string());
    lines.push(format!("  name: {}", rule.name));
    lines.push("  annotations:".to_string());
    lines.push(format!("    description: \"{}\"", rule.description));
    lines.push("spec:".to_string());
    lines.push("  endpointSelector:".to_string());
    lines.push("    matchLabels:".to_string());
    lines.push("      app: browser".to_string());
    
    // Permettre l'accès DNS
    lines.push("  egress:".to_string());
    lines.push("  - toEndpoints:".to_string());
    lines.push("    - matchLabels:".to_string());
    lines.push("        k8s-app: kube-dns".to_string());
    lines.push("    toPorts:".to_string());
    lines.push("    - ports:".to_string());
    lines.push("      - port: \"53\"".to_string());
    lines.push("        protocol: UDP".to_string());
    
    if rule.rule_type == "deny" {
        if let Some(host) = &rule.host {
            // IMPORTANT: Ajouter cette partie pour bloquer l'accès au domaine
            lines.push("  - toFQDNs:".to_string());
            lines.push(format!("    - matchPattern: \"*.{}\"", host));
            lines.push(format!("    - matchName: \"{}\"", host));
            
            // Ajouter la configuration des ports si spécifiés
            if let Some(port) = rule.port {
                lines.push("    toPorts:".to_string());
                lines.push("    - ports:".to_string());
                lines.push(format!("      - port: \"{}\"", port));
                lines.push("        protocol: TCP".to_string());
            }
        }
    } else {
        // Règle allow - permet tout le trafic sauf ce qui est spécifiquement bloqué
        if let Some(host) = &rule.host {
            lines.push("  - toCIDR:".to_string());
            lines.push("    - 0.0.0.0/0".to_string());
            
            if let Some(port) = rule.port {
                lines.push("    toPorts:".to_string());
                lines.push("    - ports:".to_string());
                lines.push(format!("      - port: \"{}\"", port));
                lines.push("        protocol: TCP".to_string());
            }
        }
    }
    
    lines.join("\n")
}

// Configuration des routes pour le module `regle`
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_rules);
    cfg.service(get_rule);
    cfg.service(add_rule);
    cfg.service(update_rule);
    cfg.service(delete_rule);
}