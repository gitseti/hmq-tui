use hivemq_openapi::models::{DataPolicy, DataPolicyMatching};
use hmq_tui::{
    action::Action,
    components::{
        item_features::ItemSelector,
        tabs::data_policies::{DataPoliciesTab, DataPolicySelector},
        Component,
    },
    hivemq_rest_client::create_data_policy,
};
use indoc::indoc;
use tokio::sync::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::common::{assert_draw, create_item, Hivemq};

mod common;

#[tokio::test]
async fn test_data_policies_tab() {
    let hivemq = Hivemq::start();
    hivemq.enable_data_hub_trial().await;

    for i in 0..100 {
        let data_policy = DataPolicy::new(
            format!("data-policy-{i}"),
            DataPolicyMatching::new(format!("topic-{i}")),
        );
        create_data_policy(
            hivemq.host.clone(),
            serde_json::to_string(&data_policy).unwrap(),
        )
        .await
        .unwrap();
    }

    let mut tab = DataPoliciesTab::new(hivemq.host);
    let (tx, mut rx): (UnboundedSender<Action>, UnboundedReceiver<Action>) =
        mpsc::unbounded_channel();
    tab.register_action_handler(tx.clone()).unwrap();

    tab.update(Action::LoadAllItems).unwrap();
    let action = rx.recv().await.unwrap();
    let Action::ItemsLoadingFinished { result } = &action else {
        panic!("'Received wrong action {:?}", action.clone());
    };
    let data_policies = result.clone().unwrap();
    let data_policy0 = DataPolicySelector
        .select(data_policies[0].clone().1)
        .unwrap();

    tab.update(action).unwrap();
    assert_draw(
        &mut tab,
        indoc! {"
            ┌Data Policies (0/100)──────────┐┌Data Policy──────────────────────────────────────────────────────┐
            │data-policy-0                  ││                                                                 │
            │data-policy-1                  ││                                                                 │
            │data-policy-2                  ││                                                                 │
            │data-policy-3                  ││                                                                 │
            │data-policy-4                  ││                                                                 │
            │data-policy-5                  ││                                                                 │
            │data-policy-6                  ││                                                                 │
            │data-policy-7                  ││                                                                 │
            │data-policy-8                  ││                                                                 │
            │data-policy-9                  ││                                                                 │
            │data-policy-10                 ││                                                                 │
            │data-policy-11                 ││                                                                 │
            │data-policy-12                 ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
            "},
    );

    tab.update(Action::NextItem).unwrap();
    assert_draw(&mut tab, &indoc! { r#"
            ┌Data Policies (1/100)──────────┐┌Data Policy──────────────────────────────────────────────────────┐
            │data-policy-0                  ││  1 {                                                            │
            │data-policy-1                  ││  2   "createdAt": "1***********************",                   │
            │data-policy-2                  ││  3   "id": "data-policy-0",                                     │
            │data-policy-3                  ││  4   "lastUpdatedAt": "2***********************",               │
            │data-policy-4                  ││  5   "matching": {                                              │
            │data-policy-5                  ││  6     "topicFilter": "topic-0"                                 │
            │data-policy-6                  ││  7   },                                                         │
            │data-policy-7                  ││  8   "onFailure": {                                             │
            │data-policy-8                  ││  9     "pipeline": []                                           │
            │data-policy-9                  ││ 10   },                                                         │
            │data-policy-10                 ││ 11   "onSuccess": {                                             │
            │data-policy-11                 ││ 12     "pipeline": []                                           │
            │data-policy-12                 ││ 13   },                                                         │
            │data-policy-13                 ││ 14   "validation": {                                            │
            │data-policy-14                 ││ 15     "validators": []                                         │
            │data-policy-15                 ││ 16   }                                                          │
            │data-policy-16                 ││ 17 }                                                            │
            │data-policy-17                 ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#
        }.replace("1***********************", &data_policy0.created_at.unwrap())
        .replace("2***********************", &data_policy0.last_updated_at.unwrap()),
    );

    let data_policy = DataPolicy::new(
        "new-data_policy".to_owned(),
        DataPolicyMatching::new("new-topic-filter".to_owned()),
    );
    let data_policy = create_item(&mut tab, &mut rx, data_policy, &DataPolicySelector).await;
    assert_draw(&mut tab, &indoc! {r#"
            ┌Data Policies (101/101)────────┐┌Data Policy──────────────────────────────────────────────────────┐
            │data-policy-83                 ││  1 {                                                            │
            │data-policy-84                 ││  2   "createdAt": "1***********************",                   │
            │data-policy-85                 ││  3   "id": "new-data_policy",                                   │
            │data-policy-86                 ││  4   "lastUpdatedAt": "2***********************",               │
            │data-policy-87                 ││  5   "matching": {                                              │
            │data-policy-88                 ││  6     "topicFilter": "new-topic-filter"                        │
            │data-policy-89                 ││  7   },                                                         │
            │data-policy-90                 ││  8   "onFailure": {                                             │
            │data-policy-91                 ││  9     "pipeline": []                                           │
            │data-policy-92                 ││ 10   },                                                         │
            │data-policy-93                 ││ 11   "onSuccess": {                                             │
            │data-policy-94                 ││ 12     "pipeline": []                                           │
            │data-policy-95                 ││ 13   },                                                         │
            │data-policy-96                 ││ 14   "validation": {                                            │
            │data-policy-97                 ││ 15     "validators": []                                         │
            │data-policy-98                 ││ 16   }                                                          │
            │data-policy-99                 ││ 17 }                                                            │
            │new-data_policy                ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#}.replace("1***********************", &data_policy.created_at.unwrap())
        .replace("2***********************", &data_policy.last_updated_at.unwrap())
    );
}
