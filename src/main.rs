use colink_protocol_policy_module::init::Init;
use colink_protocol_policy_module::policy_module::PolicyModuleLauncher;

colink::protocol_start!(
    ("policy_module:@init", Init),
    ("policy_module:local", PolicyModuleLauncher)
);
