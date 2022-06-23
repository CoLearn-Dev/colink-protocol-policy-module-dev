use colink_policy_module::policy_module::PolicyModuleLauncher;
use colink_policy_module::user_interface::*;

colink_sdk_p::protocol_start!(
    ("policy_module:local", PolicyModuleLauncher),
    ("policy_module.start:local", UserStart),
    ("policy_module.stop:local", UserStop),
    ("policy_module.rule.add_protocol:local", UserAddProtocol),
    (
        "policy_module.rule.remove_protocol:local",
        UserRemoveProtocol
    )
);
