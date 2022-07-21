use colink_policy_module::colink_policy_module_proto::*;
use colink_sdk_a::CoLink;
use prost::Message;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let addr = &args[0];
    let jwt = &args[1];

    let cl = CoLink::new(addr, jwt);
    let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
        Ok(res) => prost::Message::decode(&*res)?,
        Err(_) => Default::default(),
    };
    let rule_id = uuid::Uuid::new_v4().to_string();
    let rule = Rule {
        rule_id: rule_id.clone(),
        task_filter: Some(TaskFilter::default()),
        action: "approve".to_string(),
        priority: 1,
    };
    settings.rules.push(rule);
    let mut payload = vec![];
    settings.encode(&mut payload).unwrap();
    cl.update_entry("_policy_module:settings", &payload).await?;

    Ok(())
}
