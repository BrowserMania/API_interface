//use kube::{Client, Api};
//use anyhow;
use k8s_openapi::api::core::v1::{Namespace, Pod};
use kube::{
    api::DeleteParams,
    Api,
    Client,
    //core_methods::Api,
};

#[tokio::main]
pub async fn del_namespace(mut user: String) -> anyhow::Result<()> {
    user = format!("{user}-browser");
    let client = Client::try_default().await?;
    println!("Votre user est {}", user);
    let namespace: Api<Namespace> = Api::all(client);
    let delete_params = DeleteParams::default();
    let result = namespace.delete("{user}", &delete_params).await?;
    println!("namespace {:?}  supprimé", result);
    Ok(())
    /*Ok(result.metadata.name.unwrap())
     */
}

#[tokio::main]
pub async fn del_pod(mut user: String) -> anyhow::Result<()> {
    user = format!("{user}-browser");
    let client = Client::try_default().await?;
    println!("Votre user est {}", user);

    let pods: Api<Pod> = Api::default_namespaced(client);

    let delete_params = DeleteParams::default();
    let result = pods.delete("{user}", &delete_params).await?;

    println!("deleted pod {:?}", result);

    Ok(())
}
