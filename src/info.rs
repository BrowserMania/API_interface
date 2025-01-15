use k8s_openapi::api::core::v1::{Namespace, Pod};
use kube::api::ListParams;
use kube::{Api, Client};
//use std::error::Error;

#[tokio::main]
pub async fn pods() -> anyhow::Result<()> {
    // Load the kubeconfig file.
    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;
    let client = Client::try_from(config)?;

    // Work with Kubernetes API.
    let pods: Api<Pod> = Api::default_namespaced(client);
    let lp = ListParams::default();

    for p in pods.list(&lp).await? {
        println!("Found Pod: {}", p.metadata.name.unwrap_or_default());
    }

    Ok(())
}

#[tokio::main]
pub async fn namespace() -> anyhow::Result<()> {
    let config = kube::Config::from_kubeconfig(&kube::config::KubeConfigOptions::default()).await?;
    let client = Client::try_from(config)?;

    let namespaces: Api<Namespace> = Api::all(client);
    match namespaces.list(&Default::default()).await {
        Ok(list) => {
            println!("Namespaces dans le cluster:");
            for ns in list.items {
                if let Some(name) = ns.metadata.name {
                    println!("- {}", name);
                }
            }
        }
        Err(_e) => todo!(),
    }
    Ok(())
}
