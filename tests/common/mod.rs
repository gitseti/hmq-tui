use crossterm::event::{KeyCode, KeyEvent};
use hmq_tui::{
    action::Action,
    components::{item_features::ItemSelector, tabs::TabComponent},
};
use pretty_assertions::assert_str_eq;
use ratatui::{backend::TestBackend, Terminal};
use serde::Serialize;
use testcontainers::{clients::Cli, core::WaitFor, Container, GenericImage};
use tokio::sync::mpsc::UnboundedReceiver;

pub struct Hivemq<'a> {
    container: Container<'a, GenericImage>,
    pub host: String,
}

impl<'a> Hivemq<'a> {
    pub fn start(docker: &'a Cli) -> Self {
        let hivemq_image = GenericImage::new("hivemq/hivemq4", "latest")
            .with_env_var("HIVEMQ_REST_API_ENABLED", "true")
            .with_wait_for(WaitFor::StdOutMessage {
                message: "Started HiveMQ in".to_owned(),
            })
            .with_wait_for(WaitFor::StdOutMessage {
                message: "Started HiveMQ REST API".to_owned(),
            })
            .with_exposed_port(1883)
            .with_exposed_port(8888);
        let container = docker.run(hivemq_image);
        let rest_api_port = container.get_host_port_ipv4(8888);
        let host = format!("http://localhost:{rest_api_port}");
        Hivemq { container, host }
    }
}

pub async fn create_item<T: TabComponent, I: Serialize>(
    tab: &mut T,
    rx: &mut UnboundedReceiver<Action>,
    item: I,
    selector: &dyn ItemSelector<I>,
) -> I {
    let schema_create_json = serde_json::to_string_pretty(&item).unwrap();
    tab.update(Action::NewItem).unwrap();
    for c in schema_create_json.chars() {
        tab.handle_key_events(KeyEvent::from(KeyCode::Char(c)))
            .unwrap();
    }
    tab.update(Action::Submit).unwrap();
    let action = rx.recv().await.unwrap();
    let Action::ItemCreated { result } = &action else {
        panic!("Received wrong action {:?}", action);
    };
    tab.update(action.clone()).unwrap();
    selector.select(result.clone().unwrap()).unwrap()
}

pub fn assert_draw<T: TabComponent>(tab: &mut T, expected: &str) {
    let width = expected.lines().next().unwrap().chars().count() as u16;
    let height = expected.lines().count() as u16;
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            tab.draw(f, f.size()).unwrap();
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let actual: String = (0..height)
        .map(|y| {
            let line = (0..width)
                .map(|x| buffer.get(x, y).symbol())
                .collect::<String>();
            format!("{line}\n")
        })
        .collect();

    assert_str_eq!(expected, actual);
}