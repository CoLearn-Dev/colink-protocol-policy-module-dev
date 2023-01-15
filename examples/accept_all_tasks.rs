use colink::CoLink;
use colink_protocol_policy_module::colink_policy_module_proto::*;
use prost::Message;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let addr = &args[0];
    let jwt = &args[1];

    let cl = CoLink::new(addr, jwt);
    let lock = cl.lock("_policy_module:settings").await?;
    let mut settings: Settings = match cl.read_entry("_policy_module:settings").await {
        Ok(res) => prost::Message::decode(&*res)?,
        Err(_) => Default::default(),
    };
    let rule_id = uuid::Uuid::new_v4().to_string();
    let rule = Rule {
        rule_id: rule_id.clone(),
        task_filter: Some(TaskFilter::default()),
        action: Some(Action {
            r#type: "approve".to_string(),
            ..Default::default()
        }),
        priority: 1,
    };
    settings.rules.push(rule);
    let mut payload = vec![];
    settings.encode(&mut payload).unwrap();
    let timestamp = get_timestamp(&cl.update_entry("_policy_module:settings", &payload).await?);
    cl.unlock(lock).await?;
    loop {
        let applied_settings_timestamp = cl
            .read_or_wait("_policy_module:applied_settings_timestamp")
            .await?;
        let applied_settings_timestamp =
            i64::from_le_bytes(<[u8; 8]>::try_from(applied_settings_timestamp).unwrap());
        if applied_settings_timestamp >= timestamp {
            return Ok(());
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn get_timestamp(key_path: &str) -> i64 {
    let pos = key_path.rfind('@').unwrap();
    key_path[pos + 1..].parse().unwrap()
}
