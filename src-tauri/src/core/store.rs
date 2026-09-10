use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub model: String,
    pub thinking_enabled: bool,
    pub thinking_effort: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub autotitle_llm_done: bool,
    #[serde(default)]
    pub title_user_edited: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_call_id: Option<String>,
    pub seq: i64,
    pub created_at: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments_json: Option<String>,
    #[serde(skip)]
    pub provider_state_json: Option<String>,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("id", &self.id)
            .field("session_id", &self.session_id)
            .field("role", &self.role)
            .field("content", &self.content)
            .field("reasoning_content", &self.reasoning_content)
            .field("tool_call_id", &self.tool_call_id)
            .field("seq", &self.seq)
            .field("created_at", &self.created_at)
            .field("archived", &self.archived)
            .field("attachments_json", &self.attachments_json)
            .field(
                "provider_state_json",
                &self.provider_state_json.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: String,
    pub message_id: String,
    pub name: String,
    pub args_json: String,
    pub result_json: Option<String>,
    pub status: String,
    pub duration_ms: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyPending {
    pub session_id: String,
    pub turn_id: String,
    pub tool_call_id: String,
    pub question_json: String,
    pub created_at: String,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: PathBuf) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| StoreError::Message(e.to_string()))?;
        }
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                hidden INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                model TEXT NOT NULL,
                thinking_enabled INTEGER NOT NULL,
                thinking_effort TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT,
                reasoning_content TEXT,
                tool_call_id TEXT,
                seq INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );
            CREATE TABLE IF NOT EXISTS tool_calls (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                name TEXT NOT NULL,
                args_json TEXT NOT NULL,
                result_json TEXT,
                status TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(message_id) REFERENCES messages(id)
            );
            CREATE TABLE IF NOT EXISTS clarify_pending (
                session_id TEXT PRIMARY KEY,
                turn_id TEXT NOT NULL,
                tool_call_id TEXT NOT NULL,
                question_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id),
                FOREIGN KEY(tool_call_id) REFERENCES tool_calls(id)
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        // 兼容旧库：列已存在则忽略
        let _ = self.conn.execute(
            "ALTER TABLE projects ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE messages ADD COLUMN archived INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE sessions ADD COLUMN last_token_count INTEGER",
            [],
        );
        let _ = self
            .conn
            .execute("ALTER TABLE messages ADD COLUMN attachments_json TEXT", []);
        let _ = self.conn.execute(
            "ALTER TABLE sessions ADD COLUMN autotitle_llm_done INTEGER NOT NULL DEFAULT 0",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE sessions ADD COLUMN title_user_edited INTEGER NOT NULL DEFAULT 0",
            [],
        );
        self.ensure_column(
            "messages",
            "provider_state_json",
            "ALTER TABLE messages ADD COLUMN provider_state_json TEXT",
        )?;
        let _ = self.conn.execute_batch(
            "UPDATE sessions SET autotitle_llm_done = 1
             WHERE id IN (
               SELECT session_id FROM messages WHERE role = 'user'
               GROUP BY session_id HAVING COUNT(*) >= 2
             );",
        );
        Ok(())
    }

    fn table_has_column(&self, table: &str, column: &str) -> Result<bool, StoreError> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .map_err(StoreError::from)?;
        let mut rows = stmt.query([]).map_err(StoreError::from)?;
        while let Some(row) = rows.next().map_err(StoreError::from)? {
            let name: String = row.get(1).map_err(StoreError::from)?;
            if name == column {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn ensure_column(&self, table: &str, column: &str, ddl: &str) -> Result<(), StoreError> {
        if self.table_has_column(table, column)? {
            return Ok(());
        }
        self.conn.execute(ddl, []).map_err(StoreError::from)?;
        Ok(())
    }

    fn map_session_row(row: &rusqlite::Row<'_>) -> Result<Session, rusqlite::Error> {
        Ok(Session {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            model: row.get(3)?,
            thinking_enabled: row.get::<_, i32>(4)? != 0,
            thinking_effort: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            autotitle_llm_done: row.get::<_, i32>(8)? != 0,
            title_user_edited: row.get::<_, i32>(9)? != 0,
        })
    }

    fn map_message_row(row: &rusqlite::Row<'_>) -> Result<Message, rusqlite::Error> {
        Ok(Message {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            reasoning_content: row.get(4)?,
            tool_call_id: row.get(5)?,
            seq: row.get(6)?,
            created_at: row.get(7)?,
            archived: row.get::<_, i32>(8)? != 0,
            attachments_json: row.get(9).ok(),
            provider_state_json: row.get(10).ok(),
        })
    }

    pub fn create_project(&self, name: &str, root_path: &str) -> Result<Project, StoreError> {
        if let Some(existing) = self.get_project_by_root_path(root_path)? {
            self.conn.execute(
                "UPDATE projects SET hidden = 0, name = ?1 WHERE id = ?2",
                params![name, existing.id],
            )?;
            return Ok(Project {
                id: existing.id,
                name: name.to_string(),
                root_path: existing.root_path,
                created_at: existing.created_at,
            });
        }
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, created_at, hidden) VALUES (?1, ?2, ?3, ?4, 0)",
            params![id, name, root_path, created_at],
        )?;
        Ok(Project {
            id,
            name: name.to_string(),
            root_path: root_path.to_string(),
            created_at,
        })
    }

    pub fn hide_project(&self, id: &str) -> Result<(), StoreError> {
        let updated = self
            .conn
            .execute("UPDATE projects SET hidden = 1 WHERE id = ?1", params![id])?;
        if updated == 0 {
            return Err(StoreError::Message("project not found".into()));
        }
        Ok(())
    }

    pub fn get_project_by_root_path(&self, root_path: &str) -> Result<Option<Project>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, root_path, created_at FROM projects WHERE root_path = ?1")?;
        let mut rows = stmt.query(params![root_path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, created_at FROM projects WHERE hidden = 0 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, root_path, created_at FROM projects WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                created_at: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn create_session(
        &self,
        project_id: &str,
        title: &str,
        model: &str,
        thinking_enabled: bool,
        thinking_effort: &str,
    ) -> Result<Session, StoreError> {
        let id = Uuid::new_v4().to_string();
        let ts = now();
        self.conn.execute(
            "INSERT INTO sessions (id, project_id, title, model, thinking_enabled, thinking_effort, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                project_id,
                title,
                model,
                thinking_enabled as i32,
                thinking_effort,
                ts,
                ts
            ],
        )?;
        Ok(Session {
            id,
            project_id: project_id.to_string(),
            title: title.to_string(),
            model: model.to_string(),
            thinking_enabled,
            thinking_effort: thinking_effort.to_string(),
            created_at: ts.clone(),
            updated_at: ts,
            autotitle_llm_done: false,
            title_user_edited: false,
        })
    }

    pub fn list_sessions(&self, project_id: &str) -> Result<Vec<Session>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, title, model, thinking_enabled, thinking_effort, created_at, updated_at,
                    autotitle_llm_done, title_user_edited
             FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], Self::map_session_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, title, model, thinking_enabled, thinking_effort, created_at, updated_at,
                    autotitle_llm_done, title_user_edited
             FROM sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::map_session_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn session_has_chat_messages(&self, session_id: &str) -> Result<bool, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT 1 FROM messages WHERE session_id = ?1 AND role IN ('user', 'assistant') LIMIT 1",
        )?;
        let mut rows = stmt.query(params![session_id])?;
        Ok(rows.next()?.is_some())
    }

    pub fn update_session(
        &self,
        id: &str,
        title: Option<&str>,
        model: Option<&str>,
        thinking_enabled: Option<bool>,
        thinking_effort: Option<&str>,
    ) -> Result<Session, StoreError> {
        let current = self
            .get_session(id)?
            .ok_or_else(|| StoreError::Message("session not found".into()))?;
        let has_chat = self.session_has_chat_messages(id)?;
        if has_chat {
            let model_locked = model.is_some_and(|m| m != current.model);
            let thinking_locked = thinking_enabled.is_some_and(|v| v != current.thinking_enabled);
            let effort_locked =
                thinking_effort.is_some_and(|e| e != current.thinking_effort.as_str());
            if model_locked || thinking_locked || effort_locked {
                return Err(StoreError::Message("session model is locked".into()));
            }
        }
        let new_title = title.unwrap_or(&current.title);
        let model = model.unwrap_or(&current.model);
        let thinking_enabled = thinking_enabled.unwrap_or(current.thinking_enabled);
        let thinking_effort = thinking_effort.unwrap_or(&current.thinking_effort);
        let title_user_edited =
            current.title_user_edited || title.is_some_and(|t| t != current.title.as_str());
        let updated_at = now();
        self.conn.execute(
            "UPDATE sessions SET title = ?1, model = ?2, thinking_enabled = ?3, thinking_effort = ?4, updated_at = ?5, title_user_edited = ?6 WHERE id = ?7",
            params![new_title, model, thinking_enabled as i32, thinking_effort, updated_at, title_user_edited as i32, id],
        )?;
        Ok(Session {
            id: current.id,
            project_id: current.project_id,
            title: new_title.to_string(),
            model: model.to_string(),
            thinking_enabled,
            thinking_effort: thinking_effort.to_string(),
            created_at: current.created_at,
            updated_at,
            autotitle_llm_done: current.autotitle_llm_done,
            title_user_edited,
        })
    }

    pub fn set_session_title_autogen(&self, id: &str, title: &str) -> Result<(), StoreError> {
        let updated_at = now();
        let updated = self.conn.execute(
            "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, updated_at, id],
        )?;
        if updated == 0 {
            return Err(StoreError::Message("session not found".into()));
        }
        Ok(())
    }

    pub fn claim_autotitle_llm(&self, id: &str) -> Result<bool, StoreError> {
        let updated = self.conn.execute(
            "UPDATE sessions SET autotitle_llm_done = 1
             WHERE id = ?1 AND autotitle_llm_done = 0 AND title_user_edited = 0",
            params![id],
        )?;
        Ok(updated > 0)
    }

    pub fn finish_llm_autotitle(&self, id: &str, title: &str) -> Result<bool, StoreError> {
        let updated_at = now();
        let updated = self.conn.execute(
            "UPDATE sessions SET title = ?1, updated_at = ?2
             WHERE id = ?3 AND autotitle_llm_done = 1 AND title_user_edited = 0",
            params![title, updated_at, id],
        )?;
        Ok(updated > 0)
    }

    pub fn delete_session(&mut self, id: &str) -> Result<(), StoreError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM tool_calls WHERE message_id IN (SELECT id FROM messages WHERE session_id = ?1)",
            params![id],
        )?;
        tx.execute("DELETE FROM messages WHERE session_id = ?1", params![id])?;
        let deleted = tx.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        if deleted == 0 {
            return Err(StoreError::Message("session not found".into()));
        }
        tx.commit()?;
        Ok(())
    }

    pub fn next_seq(&self, session_id: &str) -> Result<i64, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(MAX(seq), 0) FROM messages WHERE session_id = ?1")?;
        let max: i64 = stmt.query_row(params![session_id], |row| row.get(0))?;
        Ok(max + 1)
    }

    /// Inserts a compaction summary before preserved messages (summary seq < preserved seq).
    pub fn add_compaction_summary(
        &self,
        session_id: &str,
        content: &str,
        insert_before_seq: i64,
    ) -> Result<Message, StoreError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        self.conn.execute(
            "UPDATE messages SET seq = seq + 1 WHERE session_id = ?1 AND seq >= ?2",
            params![session_id, insert_before_seq],
        )?;
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at)
             VALUES (?1, ?2, 'user', ?3, NULL, NULL, ?4, ?5)",
            params![id, session_id, content, insert_before_seq, created_at],
        )?;
        self.conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![created_at, session_id],
        )?;
        Ok(Message {
            id,
            session_id: session_id.to_string(),
            role: "user".into(),
            content: Some(content.to_string()),
            reasoning_content: None,
            tool_call_id: None,
            seq: insert_before_seq,
            created_at,
            archived: false,
            attachments_json: None,
            provider_state_json: None,
        })
    }

    pub fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: Option<&str>,
        reasoning_content: Option<&str>,
        tool_call_id: Option<&str>,
        attachments_json: Option<&str>,
    ) -> Result<Message, StoreError> {
        self.add_message_with_provider_state(
            session_id,
            role,
            content,
            reasoning_content,
            tool_call_id,
            attachments_json,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_message_with_provider_state(
        &self,
        session_id: &str,
        role: &str,
        content: Option<&str>,
        reasoning_content: Option<&str>,
        tool_call_id: Option<&str>,
        attachments_json: Option<&str>,
        provider_state_json: Option<&str>,
    ) -> Result<Message, StoreError> {
        let id = Uuid::new_v4().to_string();
        let seq = self.next_seq(session_id)?;
        let created_at = now();
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at, attachments_json, provider_state_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                session_id,
                role,
                content,
                reasoning_content,
                tool_call_id,
                seq,
                created_at,
                attachments_json,
                provider_state_json
            ],
        )?;
        self.conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![created_at, session_id],
        )?;
        Ok(Message {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.map(str::to_string),
            reasoning_content: reasoning_content.map(str::to_string),
            tool_call_id: tool_call_id.map(str::to_string),
            seq,
            created_at,
            attachments_json: attachments_json.map(str::to_string),
            archived: false,
            provider_state_json: provider_state_json.map(str::to_string),
        })
    }

    pub fn list_active_messages(&self, session_id: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at, archived, attachments_json, provider_state_json
             FROM messages WHERE session_id = ?1 AND archived = 0 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(params![session_id], Self::map_message_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn mark_messages_archived(&self, ids: &[String]) -> Result<(), StoreError> {
        for id in ids {
            self.conn.execute(
                "UPDATE messages SET archived = 1 WHERE id = ?1",
                params![id],
            )?;
        }
        Ok(())
    }

    pub fn set_session_token_count(
        &self,
        session_id: &str,
        token_count: u32,
    ) -> Result<(), StoreError> {
        let updated = self.conn.execute(
            "UPDATE sessions SET last_token_count = ?1 WHERE id = ?2",
            params![token_count as i64, session_id],
        )?;
        if updated == 0 {
            return Err(StoreError::Message("session not found".into()));
        }
        Ok(())
    }

    pub fn get_session_token_count(&self, session_id: &str) -> Result<Option<u32>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT last_token_count FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let value: Option<i64> = row.get(0)?;
            Ok(value.map(|v| v as u32))
        } else {
            Ok(None)
        }
    }

    pub fn list_messages(&self, session_id: &str) -> Result<Vec<Message>, StoreError> {
        self.list_active_messages(session_id)
    }

    pub fn list_all_messages(&self, session_id: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at, archived, attachments_json, provider_state_json
             FROM messages WHERE session_id = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(params![session_id], Self::map_message_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn add_tool_call(
        &self,
        message_id: &str,
        id: &str,
        name: &str,
        args_json: &str,
    ) -> Result<ToolCallRecord, StoreError> {
        let created_at = now();
        self.conn.execute(
            "INSERT INTO tool_calls (id, message_id, name, args_json, result_json, status, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, NULL, 'running', 0, ?5)",
            params![id, message_id, name, args_json, created_at],
        )?;
        Ok(ToolCallRecord {
            id: id.to_string(),
            message_id: message_id.to_string(),
            name: name.to_string(),
            args_json: args_json.to_string(),
            result_json: None,
            status: "running".into(),
            duration_ms: 0,
            created_at,
        })
    }

    pub fn tool_call_exists(&self, id: &str) -> Result<bool, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM tool_calls WHERE id = ?1 LIMIT 1")?;
        let mut rows = stmt.query(params![id])?;
        Ok(rows.next()?.is_some())
    }

    pub fn finish_tool_call(
        &self,
        id: &str,
        result_json: &str,
        status: &str,
        duration_ms: i64,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE tool_calls SET result_json = ?1, status = ?2, duration_ms = ?3 WHERE id = ?4",
            params![result_json, status, duration_ms, id],
        )?;
        Ok(())
    }

    pub fn update_tool_call_status(&self, id: &str, status: &str) -> Result<(), StoreError> {
        let updated = self.conn.execute(
            "UPDATE tool_calls SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
        if updated == 0 {
            return Err(StoreError::Message("tool call not found".into()));
        }
        Ok(())
    }

    pub fn update_tool_call_args(&self, id: &str, args_json: &str) -> Result<(), StoreError> {
        let updated = self.conn.execute(
            "UPDATE tool_calls SET args_json = ?1 WHERE id = ?2",
            params![args_json, id],
        )?;
        if updated == 0 {
            return Err(StoreError::Message("tool call not found".into()));
        }
        Ok(())
    }

    pub fn save_clarify_pending(
        &self,
        session_id: &str,
        turn_id: &str,
        tool_call_id: &str,
        question_json: &str,
    ) -> Result<ClarifyPending, StoreError> {
        let created_at = now();
        self.conn.execute(
            "INSERT INTO clarify_pending (session_id, turn_id, tool_call_id, question_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, turn_id, tool_call_id, question_json, created_at],
        )?;
        Ok(ClarifyPending {
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
            tool_call_id: tool_call_id.to_string(),
            question_json: question_json.to_string(),
            created_at,
        })
    }

    pub fn get_clarify_pending(
        &self,
        session_id: &str,
    ) -> Result<Option<ClarifyPending>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, turn_id, tool_call_id, question_json, created_at
             FROM clarify_pending WHERE session_id = ?1",
        )?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ClarifyPending {
                session_id: row.get(0)?,
                turn_id: row.get(1)?,
                tool_call_id: row.get(2)?,
                question_json: row.get(3)?,
                created_at: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete_clarify_pending(&self, session_id: &str) -> Result<usize, StoreError> {
        Ok(self.conn.execute(
            "DELETE FROM clarify_pending WHERE session_id = ?1",
            params![session_id],
        )?)
    }

    pub fn list_tool_calls_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<ToolCallRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tc.id, tc.message_id, tc.name, tc.args_json, tc.result_json, tc.status, tc.duration_ms, tc.created_at
             FROM tool_calls tc
             JOIN messages m ON m.id = tc.message_id
             WHERE m.session_id = ?1
             ORDER BY m.seq ASC, tc.rowid ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(ToolCallRecord {
                id: row.get(0)?,
                message_id: row.get(1)?,
                name: row.get(2)?,
                args_json: row.get(3)?,
                result_json: row.get(4)?,
                status: row.get(5)?,
                duration_ms: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

pub fn project_root_from_path(root: &str) -> PathBuf {
    Path::new(root).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn project_session_message_crud() {
        let dir = tempdir().unwrap();
        let mut store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();

        store
            .add_message(&session.id, "user", Some("hello"), None, None, None)
            .unwrap();
        let assistant = store
            .add_message(
                &session.id,
                "assistant",
                Some("hi"),
                Some("thinking"),
                None,
                None,
            )
            .unwrap();

        let messages = store.list_messages(&session.id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].reasoning_content.as_deref(), Some("thinking"));

        let renamed = store
            .update_session(&session.id, Some("renamed"), None, None, None)
            .unwrap();
        assert_eq!(renamed.title, "renamed");
        assert_eq!(renamed.model, "mock");

        let locked = store.update_session(&session.id, None, Some("kimi-k2.6"), None, None);
        assert!(locked.is_err());
        assert!(locked
            .unwrap_err()
            .to_string()
            .contains("session model is locked"));

        let tc = store
            .add_tool_call(&assistant.id, "call_1", "fs_list", r#"{"path":"."}"#)
            .unwrap();
        store
            .finish_tool_call(&tc.id, r#"{"entries":[]}"#, "done", 12)
            .unwrap();
        let tool_calls = store.list_tool_calls_for_session(&session.id).unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].status, "done");
        assert_eq!(tool_calls[0].duration_ms, 12);

        store.delete_session(&session.id).unwrap();
        assert!(store.get_session(&session.id).unwrap().is_none());
        assert!(store.list_messages(&session.id).unwrap().is_empty());
        assert!(store
            .list_tool_calls_for_session(&session.id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn sessions_are_isolated_by_project() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let p1 = store.create_project("a", "/tmp/a").unwrap();
        let p2 = store.create_project("b", "/tmp/b").unwrap();
        store
            .create_session(&p1.id, "s1", "mock", true, "high")
            .unwrap();
        store
            .create_session(&p2.id, "s2", "mock", true, "high")
            .unwrap();

        assert_eq!(store.list_sessions(&p1.id).unwrap().len(), 1);
        assert_eq!(store.list_sessions(&p2.id).unwrap().len(), 1);
        assert_ne!(
            store.list_sessions(&p1.id).unwrap()[0].id,
            store.list_sessions(&p2.id).unwrap()[0].id
        );
    }

    #[test]
    fn clarify_pending_crud_and_status() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();
        let assistant = store
            .add_message(&session.id, "assistant", None, Some("thinking"), None, None)
            .unwrap();
        store
            .add_tool_call(
                &assistant.id,
                "call_clarify",
                "clarify_ask",
                r#"{"id":"q1","kind":"text","prompt":"主题？"}"#,
            )
            .unwrap();

        store
            .update_tool_call_status("call_clarify", "awaiting_user")
            .unwrap();
        store
            .save_clarify_pending(
                &session.id,
                "turn-1",
                "call_clarify",
                r#"{"id":"q1","kind":"text","prompt":"主题？"}"#,
            )
            .unwrap();

        let pending = store.get_clarify_pending(&session.id).unwrap().unwrap();
        assert_eq!(pending.turn_id, "turn-1");
        assert_eq!(pending.tool_call_id, "call_clarify");
        let calls = store.list_tool_calls_for_session(&session.id).unwrap();
        assert_eq!(calls[0].status, "awaiting_user");

        assert_eq!(store.delete_clarify_pending(&session.id).unwrap(), 1);
        assert_eq!(store.delete_clarify_pending(&session.id).unwrap(), 0);
    }

    #[test]
    fn tool_call_exists_detects_primary_key_collision() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();
        let assistant = store
            .add_message(&session.id, "assistant", None, None, None, None)
            .unwrap();
        store
            .add_tool_call(&assistant.id, "dup_id", "pdf_read", r#"{"path":"a.pdf"}"#)
            .unwrap();
        assert!(store.tool_call_exists("dup_id").unwrap());
        assert!(!store.tool_call_exists("missing").unwrap());
    }

    #[test]
    fn hide_and_restore_project_preserves_sessions() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store
            .create_project("demo", dir.path().to_str().unwrap())
            .unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();

        store.hide_project(&project.id).unwrap();
        assert!(store.list_projects().unwrap().is_empty());

        let restored = store
            .create_project("demo-restored", dir.path().to_str().unwrap())
            .unwrap();
        assert_eq!(restored.id, project.id);
        assert_eq!(store.list_projects().unwrap().len(), 1);
        assert_eq!(store.list_sessions(&project.id).unwrap().len(), 1);
        assert_eq!(store.list_sessions(&project.id).unwrap()[0].id, session.id);
    }

    #[test]
    fn settings_roundtrip() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        assert!(store.get_setting("theme").unwrap().is_none());
        store.set_setting("theme", "dark").unwrap();
        assert_eq!(store.get_setting("theme").unwrap().as_deref(), Some("dark"));
    }

    #[test]
    fn archived_messages_are_hidden_from_active_list() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();
        let old = store
            .add_message(&session.id, "user", Some("old"), None, None, None)
            .unwrap();
        store
            .add_message(&session.id, "user", Some("new"), None, None, None)
            .unwrap();
        store.mark_messages_archived(&[old.id]).unwrap();
        let active = store.list_active_messages(&session.id).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].content.as_deref(), Some("new"));
    }

    #[test]
    fn session_token_count_roundtrip() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();
        store.set_session_token_count(&session.id, 42_000).unwrap();
        assert_eq!(
            store.get_session_token_count(&session.id).unwrap(),
            Some(42_000)
        );
    }

    #[test]
    fn compaction_summary_is_inserted_before_preserved_messages() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "mock", true, "high")
            .unwrap();
        let old = store
            .add_message(&session.id, "user", Some("old"), None, None, None)
            .unwrap();
        let preserved = store
            .add_message(&session.id, "user", Some("recent"), None, None, None)
            .unwrap();
        store.mark_messages_archived(&[old.id]).unwrap();
        store
            .add_compaction_summary(
                &session.id,
                "Previous context has been compacted. Continue from this summary:\n\nsummary",
                preserved.seq,
            )
            .unwrap();
        let active = store.list_active_messages(&session.id).unwrap();
        assert_eq!(active.len(), 2);
        assert!(active[0]
            .content
            .as_deref()
            .unwrap_or("")
            .starts_with("Previous context has been compacted."));
        assert_eq!(active[1].content.as_deref(), Some("recent"));
    }

    #[test]
    fn autotitle_migration_backfills_two_user_messages() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        {
            let store = Store::open(db_path.clone()).unwrap();
            let project = store.create_project("demo", "/tmp/demo").unwrap();
            let session = store
                .create_session(&project.id, "新会话", "mock", true, "high")
                .unwrap();
            store
                .add_message(&session.id, "user", Some("a"), None, None, None)
                .unwrap();
            store
                .add_message(&session.id, "user", Some("b"), None, None, None)
                .unwrap();
        }
        let store = Store::open(db_path).unwrap();
        let session = store
            .list_sessions(&store.list_projects().unwrap()[0].id)
            .unwrap()[0]
            .clone();
        assert!(session.autotitle_llm_done);
    }

    #[test]
    fn manual_title_edit_sets_title_user_edited() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "新会话", "mock", true, "high")
            .unwrap();
        let updated = store
            .update_session(&session.id, Some("自定义标题"), None, None, None)
            .unwrap();
        assert!(updated.title_user_edited);
        assert!(!store.claim_autotitle_llm(&session.id).unwrap());
    }

    fn column_names(conn: &rusqlite::Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        let rows = stmt.query_map([], |row| row.get::<_, String>(1)).unwrap();
        rows.collect::<Result<_, _>>().unwrap()
    }

    #[test]
    fn provider_state_column_migrates_from_legacy_schema() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("legacy.db");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "
                CREATE TABLE projects (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    root_path TEXT NOT NULL UNIQUE,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE sessions (
                    id TEXT PRIMARY KEY,
                    project_id TEXT NOT NULL,
                    title TEXT NOT NULL,
                    model TEXT NOT NULL,
                    thinking_enabled INTEGER NOT NULL,
                    thinking_effort TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE TABLE messages (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT,
                    reasoning_content TEXT,
                    tool_call_id TEXT,
                    seq INTEGER NOT NULL,
                    created_at TEXT NOT NULL,
                    archived INTEGER NOT NULL DEFAULT 0,
                    attachments_json TEXT
                );
                CREATE TABLE tool_calls (
                    id TEXT PRIMARY KEY,
                    message_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    args_json TEXT NOT NULL,
                    result_json TEXT,
                    status TEXT NOT NULL,
                    duration_ms INTEGER NOT NULL,
                    created_at TEXT NOT NULL
                );
                ",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO projects (id, name, root_path, created_at) VALUES ('p1', 'demo', '/tmp/demo', 't')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO sessions (id, project_id, title, model, thinking_enabled, thinking_effort, created_at, updated_at)
                 VALUES ('s1', 'p1', 'old', 'kimi-k2.6', 0, 'high', 't', 't')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO messages (id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at, archived, attachments_json)
                 VALUES ('m0', 's1', 'user', 'see image', NULL, NULL, 1, 't', 0, '[{\"path\":\".cache/attachments/a.png\",\"mime\":\"image/png\"}]')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO messages (id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at, archived, attachments_json)
                 VALUES ('m1', 's1', 'assistant', 'hello', 'thought', NULL, 2, 't', 0, NULL)",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO tool_calls (id, message_id, name, args_json, result_json, status, duration_ms, created_at)
                 VALUES ('c1', 'm1', 'fs_list', '{}', '[]', 'done', 1, 't')",
                [],
            )
            .unwrap();
            let message_cols = column_names(&conn, "messages");
            assert!(!message_cols.contains(&"provider_state_json".into()));
        }

        let store = Store::open(db_path.clone()).unwrap();
        let message_cols = column_names(&store.conn, "messages");
        assert_eq!(
            message_cols
                .iter()
                .filter(|c| *c == "provider_state_json")
                .count(),
            1
        );
        let session_cols = column_names(&store.conn, "sessions");
        assert!(!session_cols.contains(&"provider_state_json".into()));
        let tool_cols = column_names(&store.conn, "tool_calls");
        assert!(!tool_cols.contains(&"provider_state_json".into()));

        let session = store.get_session("s1").unwrap().unwrap();
        assert_eq!(session.model, "kimi-k2.6");
        assert!(!session.thinking_enabled);
        assert_eq!(session.thinking_effort, "high");
        let messages = store.list_all_messages("s1").unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert!(messages[0]
            .attachments_json
            .as_deref()
            .unwrap()
            .contains(".cache/attachments/a.png"));
        assert_eq!(messages[1].content.as_deref(), Some("hello"));
        assert_eq!(messages[1].reasoning_content.as_deref(), Some("thought"));
        assert!(messages.iter().all(|m| m.provider_state_json.is_none()));
        let tools = store.list_tool_calls_for_session("s1").unwrap();
        assert_eq!(tools[0].name, "fs_list");
        assert_eq!(tools[0].result_json.as_deref(), Some("[]"));
        let encoded = serde_json::to_value(&messages[0]).unwrap();
        assert!(encoded.get("provider_state_json").is_none());

        let store_again = Store::open(db_path).unwrap();
        let again = column_names(&store_again.conn, "messages");
        assert_eq!(
            again.iter().filter(|c| *c == "provider_state_json").count(),
            1
        );
    }

    #[test]
    fn add_message_defaults_provider_state_to_none() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        let project = store.create_project("demo", "/tmp/demo").unwrap();
        let session = store
            .create_session(&project.id, "s1", "deepseek-v4-flash", true, "high")
            .unwrap();
        let msg = store
            .add_message(&session.id, "user", Some("hi"), None, None, None)
            .unwrap();
        assert!(msg.provider_state_json.is_none());
        let loaded = store.list_messages(&session.id).unwrap();
        assert!(loaded[0].provider_state_json.is_none());
        let with_state = store
            .add_message_with_provider_state(
                &session.id,
                "assistant",
                Some("ok"),
                None,
                None,
                None,
                Some(r#"{"protocol":"google_openai","version":1}"#),
            )
            .unwrap();
        assert!(with_state.provider_state_json.is_some());
        let encoded = serde_json::to_value(&with_state).unwrap();
        assert!(encoded.get("provider_state_json").is_none());
        assert_eq!(encoded["content"], "ok");
    }
}
