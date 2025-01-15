use anyhow;
//use k8s_openapi::api::apps::v1::Deployment;
//use k8s_openapi::api::core::v1::{Namespace, Pod};
//use kube::{
//    //api::{Api, PostParams},
//    //Client,
//};

///partie à tester
//use k8s_openapi::api::networking::v1::{
//    NetworkPolicy, NetworkPolicyEgressRule, NetworkPolicyIngressRule, NetworkPolicyPeer,
//    NetworkPolicyPort, NetworkPolicySpec,
//};

pub async fn main() -> anyhow::Result<()> {
    println!("Toto");
    let user = "{user}-browser".to_string();
    //let client = Client::try_default().await?;
    println!("Votre user est {}", user);
    /*
        let policy = NetworkPolicy {
            metadata: kube::api::ObjectMeta {
                name: Some("mapolitique".to_string()),
                ..Default::default()
            },
            spec: Some(
                NetworkPolicySpec {
                egress: Some(
                    Vec::<NetworkPolicyEgressRule> = {
                        ports: Some(NetworkPolicyPort {
                            protocol: Some("icmp"),
                            ..Default::default()
                        }),
                        to: Some(NetworkPolicyPeer{
                            namespace_selector: Some("test"),
                            ..Default::default()
                        })
                    }
                ),
                ..Default::default()
            })
        }
    */
    Ok(())
}
