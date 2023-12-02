use hivemq_openapi::models::{
    Backup, BehaviorPolicy, ClientDetails, DataPolicy, Schema, TraceRecording,
};
use std::fmt;

use crate::mode::Mode;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,

    SwitchMode(Mode),

    // Key Events
    PrevItem,
    NextItem,
    NewItem,
    Left,
    FocusDetails,
    Enter,
    Escape,
    NextTab,
    PrevTab,
    SelectTab(usize),
    LoadAllItems,
    Copy,

    // Clients view
    ClientIdsLoadingFinished(Result<Vec<String>, String>),
    ClientDetailsLoadingFinished(Result<(String, ClientDetails), String>),

    // Data Policies
    DataPoliciesLoadingFinished(Result<Vec<(String, DataPolicy)>, String>),

    // Behavior Policies
    BehaviorPoliciesLoadingFinished(Result<Vec<(String, BehaviorPolicy)>, String>),

    // Schemas
    SchemasLoadingFinished(Result<Vec<(String, Schema)>, String>),
    SchemaCreated(Result<Schema, String>),

    // Backups
    BackupsLoadingFinished(Result<Vec<(String, Backup)>, String>),

    // Trace Recordings
    TraceRecordingsLoadingFinished(Result<Vec<(String, TraceRecording)>, String>),

    SelectedItem(String),
    Submit,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionVisitor;

        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid string representation of Action")
            }

            fn visit_str<E>(self, value: &str) -> Result<Action, E>
            where
                E: de::Error,
            {
                match value {
                    "Tick" => Ok(Action::Tick),
                    "Render" => Ok(Action::Render),
                    "Suspend" => Ok(Action::Suspend),
                    "Resume" => Ok(Action::Resume),
                    "Quit" => Ok(Action::Quit),
                    "Refresh" => Ok(Action::Refresh),
                    "Help" => Ok(Action::Help),
                    "LoadAllItems" => Ok(Action::LoadAllItems),
                    "Copy" => Ok(Action::Copy),
                    "PrevItem" => Ok(Action::PrevItem),
                    "NextItem" => Ok(Action::NextItem),
                    "NewItem" => Ok(Action::NewItem),
                    "FocusDetails" => Ok(Action::FocusDetails),
                    "Enter" => Ok(Action::Enter),
                    "Submit" => Ok(Action::Submit),
                    "Escape" => Ok(Action::Escape),
                    "NextTab" => Ok(Action::NextTab),
                    "PrevTab" => Ok(Action::PrevTab),
                    "Tab1" => Ok(Action::SelectTab(0)),
                    "Tab2" => Ok(Action::SelectTab(1)),
                    "Tab3" => Ok(Action::SelectTab(2)),
                    "Tab4" => Ok(Action::SelectTab(3)),
                    "Tab5" => Ok(Action::SelectTab(4)),
                    "Tab6" => Ok(Action::SelectTab(5)),
                    data if data.starts_with("Error(") => {
                        let error_msg = data.trim_start_matches("Error(").trim_end_matches(")");
                        Ok(Action::Error(error_msg.to_string()))
                    }
                    data if data.starts_with("Resize(") => {
                        let parts: Vec<&str> = data
                            .trim_start_matches("Resize(")
                            .trim_end_matches(")")
                            .split(',')
                            .collect();
                        if parts.len() == 2 {
                            let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
                            let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
                            Ok(Action::Resize(width, height))
                        } else {
                            Err(E::custom(format!("Invalid Resize format: {}", value)))
                        }
                    }
                    _ => Err(E::custom(format!("Unknown Action variant: {}", value))),
                }
            }
        }

        deserializer.deserialize_str(ActionVisitor)
    }
}
