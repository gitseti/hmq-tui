use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
    EditorReadOnly,
    CreateEditor,
    UpdateEditor,
    Tab,
    ReadTab,
    ReadDeleteTab,
    BackupTab,
    FullTab,
    ErrorPopup,
    ConfirmPopup,
    FilterPopup,
}
