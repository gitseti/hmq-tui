use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
    EditorReadOnly,
    Editor,
    Tab,
    ReadTab,
    ReadDeleteTab,
    BackupTab,
    FullTab,
    ErrorPopup,
    ConfirmPopup,
}
