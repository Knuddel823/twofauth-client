use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const CONFIG_DIR_NAME: &str = "twofauth-client";
const UI_STATE_FILE_NAME: &str = "ui-state.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiState {
    #[serde(default)]
    pub collapsed_groups: Vec<u64>,

    #[serde(default)]
    pub ungrouped_collapsed: bool,
}

impl UiState {
    pub fn is_group_collapsed(&self, group_id: u64) -> bool {
        self.collapsed_groups.contains(&group_id)
    }

    pub fn set_group_collapsed(&mut self, group_id: u64, collapsed: bool) {
        if collapsed {
            if !self.collapsed_groups.contains(&group_id) {
                self.collapsed_groups.push(group_id);
            }
        } else {
            self.collapsed_groups.retain(|id| *id != group_id);
        }
    }

    pub fn set_ungrouped_collapsed(&mut self, collapsed: bool) {
        self.ungrouped_collapsed = collapsed;
    }
}

fn ui_state_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not determine the user configuration directory")?
        .join(CONFIG_DIR_NAME);

    Ok(config_dir.join(UI_STATE_FILE_NAME))
}

pub fn load_ui_state() -> Result<UiState> {
    let path = ui_state_path()?;

    if !path.exists() {
        return Ok(UiState::default());
    }

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Could not read UI state from {}", path.display()))?;

    serde_json::from_str(&contents)
        .with_context(|| format!("Could not parse UI state from {}", path.display()))
}

pub fn save_ui_state(state: &UiState) -> Result<()> {
    let path = ui_state_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Could not create config directory {}", parent.display()))?;
    }

    let contents = serde_json::to_string_pretty(state).context("Could not serialize UI state")?;

    fs::write(&path, contents)
        .with_context(|| format!("Could not write UI state to {}", path.display()))?;

    Ok(())
}
