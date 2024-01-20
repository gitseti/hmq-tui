use std::collections::HashMap;

use crate::components::Component;

pub mod backups;
pub mod behavior_policies;
pub mod clients;
pub mod data_policies;
pub mod schemas;
pub mod scripts;
pub mod trace_recordings;

pub trait TabComponent: Component {
    fn get_name(&self) -> &str;

    fn get_key_hints(&self) -> Vec<(&str, &str)>;
}
