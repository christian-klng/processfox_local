use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::{CoreError, CoreResult};
use super::storage::AppPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ModelRef {
    Local { id: String },
    Cloud { provider: String, id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SkillSetting {
    pub hitl: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub folder: Option<PathBuf>,
    pub system_prompt: String,
    pub model: Option<ModelRef>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub skill_settings: std::collections::BTreeMap<String, SkillSetting>,
    /// Per-agent escape hatch: when true, write tools execute without the
    /// HITL approve/reject gate. Default false — only flip this for agents
    /// the user trusts to act unattended.
    #[serde(default)]
    pub hitl_disabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Input shape from the frontend when creating a new agent.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDraft {
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub folder: Option<PathBuf>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub model: Option<ModelRef>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub hitl_disabled: bool,
}

/// Input shape when updating an existing agent. Every field is optional.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentUpdate {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub folder: Option<PathBuf>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub model: Option<ModelRef>,
    #[serde(default)]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub hitl_disabled: Option<bool>,
}

impl Agent {
    pub fn from_draft(draft: AgentDraft) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name: draft.name,
            icon: draft.icon.unwrap_or_else(|| "🦊".to_string()),
            folder: draft.folder,
            system_prompt: draft.system_prompt.unwrap_or_default(),
            model: draft.model,
            skills: draft.skills,
            skill_settings: Default::default(),
            hitl_disabled: draft.hitl_disabled,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, update: AgentUpdate) {
        if let Some(v) = update.name {
            self.name = v;
        }
        if let Some(v) = update.icon {
            self.icon = v;
        }
        if let Some(v) = update.folder {
            self.folder = Some(v);
        }
        if let Some(v) = update.system_prompt {
            self.system_prompt = v;
        }
        if let Some(v) = update.model {
            self.model = Some(v);
        }
        if let Some(v) = update.skills {
            self.skills = v;
        }
        if let Some(v) = update.hitl_disabled {
            self.hitl_disabled = v;
        }
        self.updated_at = Utc::now().to_rfc3339();
    }
}

/// Filesystem-backed agent repository.
#[derive(Debug, Clone)]
pub struct AgentRepo {
    dir: PathBuf,
}

impl AgentRepo {
    pub fn new(paths: &AppPaths) -> Self {
        Self {
            dir: paths.agents_dir(),
        }
    }

    fn file_for(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    pub fn list(&self) -> CoreResult<Vec<Agent>> {
        if !self.dir.exists() {
            return Ok(vec![]);
        }
        let mut agents = Vec::new();
        for entry in std::fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            match Self::read_file(&path) {
                Ok(agent) => agents.push(agent),
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "skipping unreadable agent file")
                }
            }
        }
        agents.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(agents)
    }

    pub fn get(&self, id: &str) -> CoreResult<Agent> {
        let path = self.file_for(id);
        if !path.exists() {
            return Err(CoreError::AgentNotFound(id.to_string()));
        }
        Self::read_file(&path)
    }

    pub fn save(&self, agent: &Agent) -> CoreResult<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.file_for(&agent.id);
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_vec_pretty(agent)?;
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> CoreResult<()> {
        let path = self.file_for(id);
        if !path.exists() {
            return Err(CoreError::AgentNotFound(id.to_string()));
        }
        std::fs::remove_file(&path)?;
        Ok(())
    }

    fn read_file(path: &Path) -> CoreResult<Agent> {
        let body = std::fs::read(path)?;
        let agent: Agent = serde_json::from_slice(&body)?;
        Ok(agent)
    }
}
