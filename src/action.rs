use std::{fmt, fmt::Debug};

use hivemq_openapi::models::{
    Backup, BehaviorPolicy, ClientDetails, DataPolicy, Schema, Script, TraceRecording,
};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};

use crate::mode::Mode;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Item {
    ClientItem(ClientDetails),
    SchemaItem(Schema),
    ScriptItem(Script),
    DataPolicyItem(DataPolicy),
    BehaviorPolicyItem(BehaviorPolicy),
    BackupItem(Backup),
    TraceRecordingItem(TraceRecording),
}

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

    ClosePopup,
    ConfirmPopup,

    // Key Events
    PrevItem,
    NextItem,
    NewItem,
    Delete,
    Left,
    FocusDetails,
    Enter,
    Escape,
    NextTab,
    PrevTab,
    SelectTab(usize),
    LoadAllItems,
    Copy,

    Submit,
    SelectedItem(String),

    // ListWithDetails
    ItemDelete {
        item_type: String,
        item_id: String,
    },
    ItemDeleted {
        item_type: String,
        result: Result<String, String>,
    },
    ItemsLoadingFinished {
        result: Result<Vec<(String, Item)>, String>,
    },
    ItemCreated {
        result: Result<Item, String>,
    },

    // Clients view
    ClientIdsLoadingFinished(Result<Vec<String>, String>),
    ClientDetailsLoadingFinished(Result<(String, ClientDetails), String>),
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
                    "DeleteItem" => Ok(Action::Delete),
                    "FocusDetails" => Ok(Action::FocusDetails),
                    "Enter" => Ok(Action::Enter),
                    "Submit" => Ok(Action::Submit),
                    "Escape" => Ok(Action::Escape),
                    "NextTab" => Ok(Action::NextTab),
                    "PrevTab" => Ok(Action::PrevTab),
                    "ClosePopup" => Ok(Action::ClosePopup),
                    "ConfirmPopup" => Ok(Action::ConfirmPopup),
                    "Tab1" => Ok(Action::SelectTab(0)),
                    "Tab2" => Ok(Action::SelectTab(1)),
                    "Tab3" => Ok(Action::SelectTab(2)),
                    "Tab4" => Ok(Action::SelectTab(3)),
                    "Tab5" => Ok(Action::SelectTab(4)),
                    "Tab6" => Ok(Action::SelectTab(5)),
                    "Tab7" => Ok(Action::SelectTab(6)),
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
