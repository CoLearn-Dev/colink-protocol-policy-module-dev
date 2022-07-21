use colink_sdk_a::{CoLink, Participant};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let addr = &args[0];
    let jwt = &args[1];

    let cl = CoLink::new(addr, jwt);
    let participants = vec![Participant {
        user_id: cl.get_user_id()?,
        role: "local".to_string(),
    }];
    let task_id = cl
        .run_task(
            "policy_module.start",
            Default::default(),
            &participants,
            false,
        )
        .await?;
    println!(
        "Local task {} has been created, it will remain in started status until the protocol starts.",
        task_id
    );

    Ok(())
}
