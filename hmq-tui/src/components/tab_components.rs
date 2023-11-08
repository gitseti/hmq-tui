use crate::components::Component;

pub trait TabComponent: Component {

    fn get_name(&self) -> String;
}
