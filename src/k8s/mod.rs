use std::env;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Container, EnvVar, Namespace, PodSpec, PodTemplateSpec, Service, ServicePort, ServiceSpec, Volume, VolumeMount};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::api::{Api, PostParams, DeleteParams};
use kube::{Client, Error};
use std::collections::BTreeMap;

pub async fn deploy(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let deployment_name = format!("{}-browser", name);
    let service_name = format!("{}-service", name);
    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), name.to_string());

    // Créer le namespace
    let namespace = Namespace {
        metadata: ObjectMeta {
            name: Some(deployment_name.clone()),
            ..Default::default()
        },
        ..Default::default()
    };

    let namespaces: Api<Namespace> = Api::all(client.clone());
    namespaces.create(&PostParams::default(), &namespace).await?;

    // Créer le deployment
    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(deployment_name.clone()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: Some(1),
            selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                match_labels: Some(labels.clone()),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels.clone()),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    // runtime_class_name: Some("kata".to_string()), // Commenté comme dans votre version
                    containers: vec![Container {
                        name: "chromium".to_string(),
                        image: Some("dilanek/docker_chromiun:test".to_string()),
                        env: Some(vec![
                            EnvVar {
                                name: "PUID".to_string(),
                                value: Some("1000".to_string()),
                                ..Default::default()
                            },
                            EnvVar {
                                name: "PGID".to_string(),
                                value: Some("1000".to_string()),
                                ..Default::default()
                            },
                            EnvVar {
                                name: "TZ".to_string(),
                                value: Some("Etc/UTC".to_string()),
                                ..Default::default()
                            },
                            EnvVar {
                                name: "CHROME_CLI".to_string(),
                                value: Some("https://isen-mediterranee.fr/en/".to_string()),
                                ..Default::default()
                            },
                        ]),
                        ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                            container_port: 3000,
                            protocol: Some("TCP".to_string()),
                            ..Default::default()
                        }]),
                        security_context: Some(k8s_openapi::api::core::v1::SecurityContext {
                            seccomp_profile: Some(k8s_openapi::api::core::v1::SeccompProfile {
                                type_: "Unconfined".to_string(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        volume_mounts: Some(vec![VolumeMount {
                            name: "shm".to_string(),
                            mount_path: "/dev/shm".to_string(),
                            ..Default::default()
                        }]),
                        resources: Some(k8s_openapi::api::core::v1::ResourceRequirements {
                            limits: Some({
                                let mut limits = BTreeMap::new();
                                limits.insert("memory".to_string(), Quantity("1Gi".to_string()));
                                limits
                            }),
                            requests: Some({
                                let mut requests = BTreeMap::new();
                                requests.insert("memory".to_string(), Quantity("512Mi".to_string()));
                                requests
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    volumes: Some(vec![Volume {
                        name: "shm".to_string(),
                        empty_dir: Some(k8s_openapi::api::core::v1::EmptyDirVolumeSource {
                            medium: Some("Memory".to_string()),
                            size_limit: Some(Quantity("1Gi".to_string())),
                        }),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let namespace_name = deployment_name.clone();
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), &namespace_name);
    deployments.create(&PostParams::default(), &deployment).await?;

    // Créer le service avec type LoadBalancer au lieu de ClusterIP
    let service = Service {
        metadata: ObjectMeta {
            name: Some(service_name.clone()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            selector: Some(labels),
            ports: Some(vec![ServicePort {
                protocol: Some("TCP".to_string()),
                port: 3000,
                target_port: Some(IntOrString::Int(3000)),
                ..Default::default()
            }]),
            type_: Some("LoadBalancer".to_string()), // Changé de ClusterIP à LoadBalancer
            ..Default::default()
        }),
        ..Default::default()
    };

    let services: Api<Service> = Api::namespaced(client, &namespace_name);
    services.create(&PostParams::default(), &service).await?;

    Ok(())
}

// Nouvelle fonction pour récupérer l'IP du LoadBalancer
pub async fn get_loadbalancer_ip(service_prefix: &str) -> Result<String, Error> {
    let client = Client::try_default().await?;
    let namespace = format!("{}-browser", service_prefix);
    let service_name = format!("{}-service", service_prefix);
    
    let services: Api<Service> = Api::namespaced(client, &namespace);
    let svc = services.get(&service_name).await?;
    
    if let Some(status) = svc.status {
        if let Some(lb) = status.load_balancer {
            if let Some(ingresses) = lb.ingress {
                if let Some(ingress) = ingresses.first() {
                    if let Some(ip) = &ingress.ip {
                        return Ok(ip.clone());
                    }
                }
            }
        }
    }
    
    Err(Error::Service("No LoadBalancer IP found".to_string().into()))
}

// Fonction pour supprimer un déploiement (équivalent de votre stop_browser)
pub async fn cleanup(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let namespace_name = format!("{}-browser", name);
    
    // Supprimer tout le namespace (cela supprime automatiquement tous les objets à l'intérieur)
    let namespaces: Api<Namespace> = Api::all(client);
    match namespaces.delete(&namespace_name, &DeleteParams::default()).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Erreur lors de la suppression du namespace: {}", e);
            Err(Box::new(e))
        }
    }
}

// Fonction pour lister les déploiements actifs
pub async fn list_deployments() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = Client::try_default().await?;
    let namespaces: Api<Namespace> = Api::all(client);
    
    let mut browser_namespaces = Vec::new();
    
    match namespaces.list(&Default::default()).await {
        Ok(namespace_list) => {
            for namespace in namespace_list.items {
                if let Some(name) = namespace.metadata.name {
                    if name.ends_with("-browser") && !name.starts_with("kube-") {
                        browser_namespaces.push(name);
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("Erreur lors de la récupération des namespaces: {}", e);
            return Err(Box::new(e));
        }
    }
    
    Ok(browser_namespaces)
}

// Fonction pour attendre que le LoadBalancer soit prêt et retourner l'IP
pub async fn wait_for_loadbalancer(service_prefix: &str, max_attempts: u32) -> Result<String, Error> {
    use tokio::time::{sleep, Duration};
    
    for attempt in 1..=max_attempts {
        println!("Tentative {} de récupération de l'IP LoadBalancer pour {}", attempt, service_prefix);
        
        match get_loadbalancer_ip(service_prefix).await {
            Ok(ip) => {
                println!("LoadBalancer IP trouvée: {}", ip);
                return Ok(ip);
            },
            Err(e) => {
                if attempt == max_attempts {
                    eprintln!("Impossible de récupérer l'IP LoadBalancer après {} tentatives: {}", max_attempts, e);
                    return Err(e);
                } else {
                    println!("LoadBalancer pas encore prêt, nouvelle tentative dans 5 secondes...");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    Err(Error::Service("LoadBalancer timeout".to_string().into()))
}