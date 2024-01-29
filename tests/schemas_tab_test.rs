use base64::{prelude::BASE64_STANDARD, Engine};
use hivemq_openapi::models::Schema;
use hmq_tui::mode::Mode;
use hmq_tui::{
    action::Action,
    components::{
        item_features::ItemSelector,
        tabs::schemas::{SchemaSelector, SchemasTab},
        Component,
    },
    hivemq_rest_client::create_schema,
};
use indoc::indoc;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::common::{assert_draw, create_item, Hivemq};

mod common;

#[tokio::test]
async fn test_schemas_tab() {
    let hivemq = Hivemq::start();

    for i in 0..100 {
        let schema = Schema::new(
            format!("schema-{i}"),
            BASE64_STANDARD.encode("{}"),
            "JSON".to_owned(),
        );
        create_schema(hivemq.host.clone(), serde_json::to_string(&schema).unwrap())
            .await
            .unwrap();
    }

    let (tx, mut rx): (UnboundedSender<Action>, UnboundedReceiver<Action>) =
        mpsc::unbounded_channel();
    let mode = Rc::new(RefCell::new(Mode::Home));
    let mut tab = SchemasTab::new(tx, hivemq.host, mode.clone());
    tab.activate().unwrap();

    tab.update(Action::LoadAllItems).unwrap();
    let action = rx.recv().await.unwrap();
    let Action::ItemsLoadingFinished { result } = &action else {
        panic!("'Received wrong action {:?}", action.clone());
    };
    let schemas = result.clone().unwrap();
    let schema0 = SchemaSelector.select(schemas[0].clone().1).unwrap();

    tab.update(action).unwrap();
    assert_draw(
        &mut tab,
        indoc! {"
            ┌Schemas (0/100)────────────────┐┌Schema───────────────────────────────────────────────────────────┐
            │schema-0                       ││                                                                 │
            │schema-1                       ││                                                                 │
            │schema-2                       ││                                                                 │
            │schema-3                       ││                                                                 │
            │schema-4                       ││                                                                 │
            │schema-5                       ││                                                                 │
            │schema-6                       ││                                                                 │
            │schema-7                       ││                                                                 │
            │schema-8                       ││                                                                 │
            │schema-9                       ││                                                                 │
            │schema-10                      ││                                                                 │
            │schema-11                      ││                                                                 │
            │schema-12                      ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
            "},
    );

    tab.update(Action::NextItem).unwrap();
    assert_draw(&mut tab, &indoc! { r#"
            ┌Schemas (1/100)────────────────┐┌Schema───────────────────────────────────────────────────────────┐
            │schema-0                       ││ 1 {                                                             │
            │schema-1                       ││ 2   "arguments": {},                                            │
            │schema-2                       ││ 3   "createdAt": "************************",                    │
            │schema-3                       ││ 4   "id": "schema-0",                                           │
            │schema-4                       ││ 5   "schemaDefinition": "e30=",                                 │
            │schema-5                       ││ 6   "type": "JSON",                                             │
            │schema-6                       ││ 7   "version": 1                                                │
            │schema-7                       ││ 8 }                                                             │
            │schema-8                       ││                                                                 │
            │schema-9                       ││                                                                 │
            │schema-10                      ││                                                                 │
            │schema-11                      ││                                                                 │
            │schema-12                      ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#
        }.replace("************************", &schema0.created_at.unwrap()),
    );

    let schema = Schema::new(
        "new-schema".to_owned(),
        BASE64_STANDARD.encode("{}"),
        "JSON".to_owned(),
    );
    let schema = create_item(&mut tab, &mut rx, schema, &SchemaSelector).await;
    assert_draw(&mut tab, &indoc! {r#"
            ┌Schemas (101/101)──────────────┐┌Schema───────────────────────────────────────────────────────────┐
            │schema-88                      ││ 1 {                                                             │
            │schema-89                      ││ 2   "arguments": {},                                            │
            │schema-90                      ││ 3   "createdAt": "************************",                    │
            │schema-91                      ││ 4   "id": "new-schema",                                         │
            │schema-92                      ││ 5   "schemaDefinition": "e30=",                                 │
            │schema-93                      ││ 6   "type": "JSON",                                             │
            │schema-94                      ││ 7   "version": 1                                                │
            │schema-95                      ││ 8 }                                                             │
            │schema-96                      ││                                                                 │
            │schema-97                      ││                                                                 │
            │schema-98                      ││                                                                 │
            │schema-99                      ││                                                                 │
            │new-schema                     ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#}.replace("************************", &schema.created_at.unwrap()),
    );
}
