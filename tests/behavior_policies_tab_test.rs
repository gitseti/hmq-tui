use hivemq_openapi::models::{BehaviorPolicy, BehaviorPolicyBehavior, BehaviorPolicyMatching};
use hmq_tui::{
    action::Action,
    components::{
        item_features::ItemSelector,
        tabs::behavior_policies::{BehaviorPoliciesTab, BehaviorPolicySelector},
        Component,
    },
    hivemq_rest_client::create_behavior_policy,
};
use indoc::indoc;
use tokio::sync::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::common::{assert_draw, create_item, Hivemq};

mod common;

#[tokio::test]
async fn test_behavior_policies_tab() {
    let hivemq = Hivemq::start();
    hivemq.enable_data_hub_trial().await;

    for i in 0..100 {
        let behavior_policy = BehaviorPolicy::new(
            BehaviorPolicyBehavior::new("Mqtt.events".to_owned()),
            format!("behavior-policy-{i}"),
            BehaviorPolicyMatching::new(".*".to_owned()),
        );
        create_behavior_policy(
            hivemq.host.clone(),
            serde_json::to_string(&behavior_policy).unwrap(),
        )
            .await
            .unwrap();
    }

    let mut tab = BehaviorPoliciesTab::new(hivemq.host);
    let (tx, mut rx): (UnboundedSender<Action>, UnboundedReceiver<Action>) =
        mpsc::unbounded_channel();
    tab.register_action_handler(tx.clone()).unwrap();

    tab.update(Action::LoadAllItems).unwrap();
    let action = rx.recv().await.unwrap();
    let Action::ItemsLoadingFinished { result } = &action else {
        panic!("'Received wrong action {:?}", action.clone());
    };
    let behavior_policies = result.clone().unwrap();
    let behavior_policy0 = BehaviorPolicySelector
        .select(behavior_policies[0].clone().1)
        .unwrap();

    tab.update(action).unwrap();
    assert_draw(
        &mut tab,
        indoc! {"
            ┌Behavior Policies (0/100)──────┐┌Behavior Policy──────────────────────────────────────────────────┐
            │behavior-policy-0              ││                                                                 │
            │behavior-policy-1              ││                                                                 │
            │behavior-policy-2              ││                                                                 │
            │behavior-policy-3              ││                                                                 │
            │behavior-policy-4              ││                                                                 │
            │behavior-policy-5              ││                                                                 │
            │behavior-policy-6              ││                                                                 │
            │behavior-policy-7              ││                                                                 │
            │behavior-policy-8              ││                                                                 │
            │behavior-policy-9              ││                                                                 │
            │behavior-policy-10             ││                                                                 │
            │behavior-policy-11             ││                                                                 │
            │behavior-policy-12             ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
            "},
    );

    tab.update(Action::NextItem).unwrap();
    assert_draw(&mut tab, &indoc! { r#"
            ┌Behavior Policies (1/100)──────┐┌Behavior Policy──────────────────────────────────────────────────┐
            │behavior-policy-0              ││  1 {                                                            │
            │behavior-policy-1              ││  2   "behavior": {                                              │
            │behavior-policy-2              ││  3     "arguments": {},                                         │
            │behavior-policy-3              ││  4     "id": "Mqtt.events"                                      │
            │behavior-policy-4              ││  5   },                                                         │
            │behavior-policy-5              ││  6   "createdAt": "1***********************",                   │
            │behavior-policy-6              ││  7   "id": "behavior-policy-0",                                 │
            │behavior-policy-7              ││  8   "lastUpdatedAt": "2***********************",               │
            │behavior-policy-8              ││  9   "matching": {                                              │
            │behavior-policy-9              ││ 10     "clientIdRegex": ".*"                                    │
            │behavior-policy-10             ││ 11   },                                                         │
            │behavior-policy-11             ││ 12   "onTransitions": []                                        │
            │behavior-policy-12             ││ 13 }                                                            │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#
        }.replace("1***********************", &behavior_policy0.created_at.unwrap())
        .replace("2***********************", &behavior_policy0.last_updated_at.unwrap()),
    );

    let behavior_policy = BehaviorPolicy::new(
        BehaviorPolicyBehavior::new("Mqtt.events".to_owned()),
        "new-behavior-policy".to_owned(),
        BehaviorPolicyMatching::new(".*".to_owned()),
    );
    let behavior_policy = create_item(&mut tab, &mut rx, behavior_policy, &BehaviorPolicySelector).await;
    assert_draw(&mut tab, &indoc! {r#"
            ┌Behavior Policies (101/101)────┐┌Behavior Policy──────────────────────────────────────────────────┐
            │behavior-policy-88             ││  1 {                                                            │
            │behavior-policy-89             ││  2   "behavior": {                                              │
            │behavior-policy-90             ││  3     "arguments": {},                                         │
            │behavior-policy-91             ││  4     "id": "Mqtt.events"                                      │
            │behavior-policy-92             ││  5   },                                                         │
            │behavior-policy-93             ││  6   "createdAt": "1***********************",                   │
            │behavior-policy-94             ││  7   "id": "new-behavior-policy",                               │
            │behavior-policy-95             ││  8   "lastUpdatedAt": "2***********************",               │
            │behavior-policy-96             ││  9   "matching": {                                              │
            │behavior-policy-97             ││ 10     "clientIdRegex": ".*"                                    │
            │behavior-policy-98             ││ 11   },                                                         │
            │behavior-policy-99             ││ 12   "onTransitions": []                                        │
            │new-behavior-policy            ││ 13 }                                                            │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#}.replace("1***********************", &behavior_policy.created_at.unwrap())
        .replace("2***********************", &behavior_policy.last_updated_at.unwrap())
    );
}
