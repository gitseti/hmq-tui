use hivemq_openapi::models::{BehaviorPolicy, BehaviorPolicyBehavior, BehaviorPolicyMatching};
use hmq_tui::mode::Mode;
use hmq_tui::{action::Action, components::{
    tabs::behavior_policies::{BehaviorPoliciesTab},
    Component,
}, repository};
use indoc::indoc;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::sync::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};
use hmq_tui::repository::Repository;
use hmq_tui::services::behavior_policy_service::BehaviorPolicyService;

use crate::common::{assert_draw, create_item, Hivemq};

mod common;

#[tokio::test]
async fn test_behavior_policies_tab() {
    let hivemq = Hivemq::start();
    hivemq.enable_data_hub_trial().await;

    let sqlite_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
    let repository = Repository::<BehaviorPolicy>::init(&sqlite_pool, "behavior_policies", |val| {
        val.id.clone()
    }, "lastUpdatedAt").unwrap();
    let repository = Arc::new(repository);
    let service = BehaviorPolicyService::new(repository.clone(), &hivemq.host.clone());

    for i in 0..100 {
        let behavior_policy = BehaviorPolicy::new(
            BehaviorPolicyBehavior::new("Mqtt.events".to_owned()),
            format!("behavior-policy-{i}"),
            BehaviorPolicyMatching::new(".*".to_owned()),
        );
        service.create_behavior_policy(&serde_json::to_string(&behavior_policy).unwrap())
            .await
            .unwrap();
    }

    let (tx, mut rx): (UnboundedSender<Action>, UnboundedReceiver<Action>) =
        mpsc::unbounded_channel();
    let mode = Rc::new(RefCell::new(Mode::Home));
    let mut tab = BehaviorPoliciesTab::new(tx, hivemq.host, mode.clone(), &sqlite_pool);
    tab.activate().unwrap();

    tab.update(Action::LoadAllItems).unwrap();
    let action = rx.recv().await.unwrap();
    let Action::ItemsLoadingFinished { item_name, result } = &action else {
        panic!("'Received wrong action {:?}", action.clone());
    };
    tab.update(action).unwrap();
    let behavior_policy= repository.find_by_id("behavior-policy-0").unwrap();
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
        }.replace("1***********************", &behavior_policy.created_at.unwrap())
        .replace("2***********************", &behavior_policy.last_updated_at.unwrap()),
    );

    let behavior_policy = BehaviorPolicy::new(
        BehaviorPolicyBehavior::new("Mqtt.events".to_owned()),
        "new-behavior-policy".to_owned(),
        BehaviorPolicyMatching::new(".*".to_owned()),
    );
    let behavior_policy =
        create_item(&mut tab, &mut rx, behavior_policy, &repository).await;
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
        .replace("2***********************", &behavior_policy.last_updated_at.unwrap()),
    );
}
