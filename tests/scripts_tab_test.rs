use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use base64::{Engine, prelude::BASE64_STANDARD};
use hivemq_openapi::models::{Script, script::FunctionType};
use indoc::indoc;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::sync::{
    mpsc,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use hmq_tui::{
    action::Action,
    components::{
        Component,
        tabs::scripts::ScriptsTab,
    },
};
use hmq_tui::mode::Mode;
use hmq_tui::repository::Repository;
use hmq_tui::services::scripts_service::ScriptService;

use crate::common::{assert_draw, create_item, Hivemq};

mod common;

#[tokio::test]
async fn test_scripts_tab() {
    let hivemq = Hivemq::start();

    let sqlite_pool = Pool::new(SqliteConnectionManager::memory()).unwrap();
    let repository = Repository::<Script>::init(&sqlite_pool, "scripts", |val| {
        val.id.clone()
    }, "createdAt").unwrap();
    let repository = Arc::new(repository);
    let service = ScriptService::new(repository.clone(), &hivemq.host.clone());

    for i in 0..100 {
        let script = Script::new(
            FunctionType::Transformation,
            format!("script-{i}"),
            BASE64_STANDARD.encode("function transform(publish, context) { return publish; }"),
        );
        service.create_script(&serde_json::to_string(&script).unwrap())
            .await
            .unwrap();
    }

    let (tx, mut rx): (UnboundedSender<Action>, UnboundedReceiver<Action>) =
        mpsc::unbounded_channel();
    let mode = Rc::new(RefCell::new(Mode::Home));
    let mut tab = ScriptsTab::new(tx, hivemq.host, mode.clone(), &sqlite_pool);
    tab.activate().unwrap();

    tab.update(Action::LoadAllItems).unwrap();
    let action = rx.recv().await.unwrap();
    let Action::ItemsLoadingFinished { result, .. } = &action else {
        panic!("'Received wrong action {:?}", action.clone());
    };
    let scripts = result.clone().unwrap();
    let script0 = repository.find_by_id("script-0").unwrap();

    tab.update(action).unwrap();
    assert_draw(
        &mut tab,
        indoc! {"
            ┌Scripts (0/100)────────────────┐┌Script───────────────────────────────────────────────────────────┐
            │script-0                       ││                                                                 │
            │script-1                       ││                                                                 │
            │script-2                       ││                                                                 │
            │script-3                       ││                                                                 │
            │script-4                       ││                                                                 │
            │script-5                       ││                                                                 │
            │script-6                       ││                                                                 │
            │script-7                       ││                                                                 │
            │script-8                       ││                                                                 │
            │script-9                       ││                                                                 │
            │script-10                      ││                                                                 │
            │script-11                      ││                                                                 │
            │script-12                      ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
            "},
    );

    tab.update(Action::NextItem).unwrap();
    assert_draw(&mut tab, &indoc! { r#"
            ┌Scripts (1/100)────────────────┐┌Script───────────────────────────────────────────────────────────┐
            │script-0                       ││ 1 {                                                             │
            │script-1                       ││ 2   "createdAt": "************************",                    │
            │script-2                       ││ 3   "functionType": "TRANSFORMATION",                           │
            │script-3                       ││ 4   "id": "script-0",                                           │
            │script-4                       ││ 5   "source": "ZnVuY3Rpb24gdHJhbnNmb3JtKHB1Ymxpc2gsIGNvbnRleHQpI│
            │script-5                       ││ 6   "version": 1                                                │
            │script-6                       ││ 7 }                                                             │
            │script-7                       ││                                                                 │
            │script-8                       ││                                                                 │
            │script-9                       ││                                                                 │
            │script-10                      ││                                                                 │
            │script-11                      ││                                                                 │
            │script-12                      ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#
        }.replace("************************", &script0.created_at.unwrap()),
    );

    let script = Script::new(
        FunctionType::Transformation,
        "new-script".to_owned(),
        BASE64_STANDARD.encode("function transform(publish, context) { return publish; }"),
    );
    let script = create_item(&mut tab, &mut rx, script, &repository).await;
    assert_draw(&mut tab, &indoc! {r#"
            ┌Scripts (101/101)──────────────┐┌Script───────────────────────────────────────────────────────────┐
            │script-88                      ││ 1 {                                                             │
            │script-89                      ││ 2   "createdAt": "************************",                    │
            │script-90                      ││ 3   "functionType": "TRANSFORMATION",                           │
            │script-91                      ││ 4   "id": "new-script",                                         │
            │script-92                      ││ 5   "source": "ZnVuY3Rpb24gdHJhbnNmb3JtKHB1Ymxpc2gsIGNvbnRleHQpI│
            │script-93                      ││ 6   "version": 1                                                │
            │script-94                      ││ 7 }                                                             │
            │script-95                      ││                                                                 │
            │script-96                      ││                                                                 │
            │script-97                      ││                                                                 │
            │script-98                      ││                                                                 │
            │script-99                      ││                                                                 │
            │new-script                     ││                                                                 │
            └───────────────────────────────┘└─────────────────────────────────────────────────────────────────┘
        "#}.replace("************************", &script.created_at.unwrap()),
    );
}
