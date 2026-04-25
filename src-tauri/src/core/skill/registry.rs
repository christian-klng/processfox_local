use include_dir::{include_dir, Dir};

use super::Skill;
use crate::core::error::{CoreError, CoreResult};

static BUILTIN: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/skills_builtin");

/// Parsed collection of available skills. Cloneable; usually shared via
/// `AppState`.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl SkillRegistry {
    /// Parse every `SKILL.md` bundled in the binary at compile time. Fails
    /// if any SKILL.md has a broken frontmatter — these are authored by us,
    /// so a break should be caught before release.
    pub fn load_builtin() -> CoreResult<Self> {
        let mut skills = Vec::new();
        collect_skills(&BUILTIN, &mut skills)?;
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self { skills })
    }

    pub fn all(&self) -> &[Skill] {
        &self.skills
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.name == name)
    }
}

fn collect_skills(dir: &Dir<'_>, into: &mut Vec<Skill>) -> CoreResult<()> {
    for file in dir.files() {
        if file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case("SKILL.md"))
            .unwrap_or(false)
        {
            let text = file.contents_utf8().ok_or_else(|| {
                CoreError::Llm(format!("SKILL.md nicht UTF-8: {}", file.path().display()))
            })?;
            let skill = parse_skill_md(text).map_err(|e| {
                CoreError::Llm(format!("Parse-Fehler in {}: {e}", file.path().display()))
            })?;
            into.push(skill);
        }
    }
    for sub in dir.dirs() {
        collect_skills(sub, into)?;
    }
    Ok(())
}

/// Split a `SKILL.md` file into YAML frontmatter and Markdown body. Requires
/// the frontmatter to be delimited by `---\n` at start and `---\n` on its own
/// line further down (standard Jekyll/Obsidian format).
fn parse_skill_md(input: &str) -> CoreResult<Skill> {
    let stripped = input
        .strip_prefix("---\n")
        .ok_or_else(|| CoreError::Llm("SKILL.md muss mit '---' beginnen".to_string()))?;
    let end = stripped
        .find("\n---\n")
        .or_else(|| stripped.find("\n---\r\n"))
        .ok_or_else(|| CoreError::Llm("Frontmatter nicht geschlossen".to_string()))?;
    let (frontmatter, rest) = stripped.split_at(end);
    let body = rest.trim_start_matches("\n---\n").trim_start().to_string();

    let mut skill: Skill = serde_yaml::from_str(frontmatter)
        .map_err(|e| CoreError::Llm(format!("YAML-Frontmatter ungültig: {e}")))?;
    skill.body = body;
    Ok(skill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_skills_parse() {
        let reg = SkillRegistry::load_builtin().expect("builtin skills must parse");
        assert!(!reg.all().is_empty(), "expected at least one builtin skill");
        let fs = reg.get("folder-search").expect("folder-search missing");
        assert_eq!(fs.title, "Ordner durchsuchen");
        assert!(fs.tools.contains(&"list_folder".to_string()));
        assert!(fs.body.contains("list_folder"));
    }

    #[test]
    fn parse_splits_frontmatter_from_body() {
        let input =
            "---\nname: demo\ntitle: Demo\ndescription: D\ntools:\n  - foo\n---\n\nBody here.\n";
        let skill = parse_skill_md(input).unwrap();
        assert_eq!(skill.name, "demo");
        assert_eq!(skill.tools, vec!["foo".to_string()]);
        assert_eq!(skill.body.trim(), "Body here.");
    }
}
