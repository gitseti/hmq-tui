use std::collections::HashMap;
use crate::components::Component;

pub trait TabComponent: Component {
    fn get_name(&self) -> String;

    fn get_key_hints(&self) -> Vec<(&str, &str)>;
}
