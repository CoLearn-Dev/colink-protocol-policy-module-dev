use colink_protocol_policy_module::init::Init;
use colink_protocol_policy_module::policy_module::PolicyModuleLauncher;
use colink_protocol_policy_module::user_interface::*;

colink::protocol_start!(
    ("policy_module:@init", Init),
    ("policy_module:local", PolicyModuleLauncher),
    ("policy_module.start:local", UserStart),
    ("policy_module.stop:local", UserStop),
    ("policy_module.rule.add_protocol:local", UserAddProtocol),
    ("policy_module.rule.remove:local", UserRemoveRule),
    ("policy_module.rule.reset:local", UserReset)
);
