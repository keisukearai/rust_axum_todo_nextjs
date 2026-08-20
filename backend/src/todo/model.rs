use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const TITLE_MAX_LEN: usize = 500;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub completed: Option<bool>,
}

/// タイトルを整形して検証する。前後の空白は落とす。
pub fn normalize_title(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("title を空にはできません".to_string());
    }
    if trimmed.chars().count() > TITLE_MAX_LEN {
        return Err(format!("title は {TITLE_MAX_LEN} 文字以内にしてください"));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(normalize_title("  買い物  ").unwrap(), "買い物");
    }

    #[test]
    fn rejects_blank_title() {
        assert!(normalize_title("   ").is_err());
        assert!(normalize_title("").is_err());
    }

    #[test]
    fn counts_length_in_characters_not_bytes() {
        let multibyte = "あ".repeat(TITLE_MAX_LEN);
        assert!(normalize_title(&multibyte).is_ok());
        assert!(normalize_title(&"あ".repeat(TITLE_MAX_LEN + 1)).is_err());
    }
}
