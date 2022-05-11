# DDS policy module
## Introduction
In DDS, when a user wants to start a task with other users, they need to send them task invitations and wait for their decisions. When a user receives a task invitation, they need to make the decision manually by default. So they will be benefited if there exists a tool to help them do this, and the policy module is such a tool to help users make their decisions automatically based on rules.

The policy module runs as a protocol operator(protocol=policy_module), and the user can use a local task to start it.

Here are some common scenarios:
1. A user wants to automatically approve all tasks with greetings protocol.
2. A user wants to automatically approve all tasks if all participants of these tasks are in a trust list.
3. In federated learning scenarios, a user may be invited as different roles in different tasks. And a user wants to automatically approve all federated learning tasks if their roles in these tasks are aggregator or client.
4. A user wants to reject all tasks without the require_agreement property.
5. A task wants to approve all its subtasks with greetings protocol.
6. ...

## Rule
The policy module will follow the rules to make decisions automatically. A user's rules will be stored in the user's storage space and the user can manage their rules through the policy module interface(SDK-?).

#### Each rule must consist of three properties:
- `task_filter` - a filter that determines when a rule takes effect.
- `action` - a string (`approve`/`reject`/`ignore`) that describes the action that will be taken.
- `priority` - a positive integer, and the smaller integer represents higher priority.

#### Each task_filter can consist of the following properties:
- `task_id` - a UUID.
- `protocol_name` - a string.
- `participants` - a list of `user_id`, a task will be matched if all participants of it except the user itself are on the list.
- `role` - a regex, a task will be matched if the user's role in the task matches the regex.
- `parent_task_filter` - a task_filter, but it can not include a `parent_task_filter` property.
- `require_agreement` - a bool.
If one property is not included in task_filter, then the filter will ignore it and use other properties to match. If a task_filter does not contain any property, it can match any task.

#### About priority:
##### What will happen if a task matches more than one rule?
If there exists a high priority rule the policy module will take it. And if there exist some rules having the same priority, the policy module will check whether their actions are the same. If their actions conflict, the policy module will skip this task.
##### What priority should I set?
We recommend users set a priority lower than 100, and the protocol operator should set `priority=100` if they want to set a policy for their subtask.


## How to config rules
TODO


## Syntax(deprecated)
The rules are in JSON formats. There are some examples.

1.
```
{
    "task_filter": {
        "protocol_name": "greetings"
    },
    "action": "approve",
    "priority": 1
}
```
2.
```
{
    "task_filter": {
        "protocol_name": "proxy_greetings",
        "participants": [
            "user_id_1",
            "user_id_2"
        ]
    },
    "action": "approve",
    "priority": 1
}
```
3.
```
{
    "task_filter": {
        "protocol_name": "fl",
        "role": "aggregator|client"
    },
    "action": "approve",
    "priority": 1
}
```
4.
```
{
    "task_filter": {
        "require_agreement": false
    },
    "action": "reject",
    "priority": 1
}
```
5.
```
{
    "task_filter": {
        "protocol_name": "greetings",
        "parent_task_filter": {
            "task_id": "2beef808-e7ff-425c-bb89-4761826c0771"
        }
    },
    "action": "approve",
    "priority": 100
}
```