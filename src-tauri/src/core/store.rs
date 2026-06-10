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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_call_id: Option<String>,
    pub seq: i64,
    pub created_at: String,
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
                created_at TEXT NOT NULL
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
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn create_project(&self, name: &str, root_path: &str) -> Result<Project, StoreError> {
        let id = Uuid::new_v4().to_string();
        let created_at = now();
        self.conn.execute(
            "INSERT INTO projects (id, name, root_path, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, root_path, created_at],
        )?;
        Ok(Project {
            id,
            name: name.to_string(),
            root_path: root_path.to_string(),
            created_at,
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, root_path, created_at FROM projects ORDER BY created_at DESC",
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
        })
    }

    pub fn list_sessions(&self, project_id: &str) -> Result<Vec<Session>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, title, model, thinking_enabled, thinking_effort, created_at, updated_at
             FROM sessions WHERE project_id = ?1 ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                model: row.get(3)?,
                thinking_enabled: row.get::<_, i32>(4)? != 0,
                thinking_effort: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, title, model, thinking_enabled, thinking_effort, created_at, updated_at
             FROM sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                model: row.get(3)?,
                thinking_enabled: row.get::<_, i32>(4)? != 0,
                thinking_effort: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
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
        let title = title.unwrap_or(&current.title);
        let model = model.unwrap_or(&current.model);
        let thinking_enabled = thinking_enabled.unwrap_or(current.thinking_enabled);
        let thinking_effort = thinking_effort.unwrap_or(&current.thinking_effort);
        let updated_at = now();
        self.conn.execute(
            "UPDATE sessions SET title = ?1, model = ?2, thinking_enabled = ?3, thinking_effort = ?4, updated_at = ?5 WHERE id = ?6",
            params![title, model, thinking_enabled as i32, thinking_effort, updated_at, id],
        )?;
        Ok(Session {
            id: current.id,
            project_id: current.project_id,
            title: title.to_string(),
            model: model.to_string(),
            thinking_enabled,
            thinking_effort: thinking_effort.to_string(),
            created_at: current.created_at,
            updated_at,
        })
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

    pub fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: Option<&str>,
        reasoning_content: Option<&str>,
        tool_call_id: Option<&str>,
    ) -> Result<Message, StoreError> {
        let id = Uuid::new_v4().to_string();
        let seq = self.next_seq(session_id)?;
        let created_at = now();
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                session_id,
                role,
                content,
                reasoning_content,
                tool_call_id,
                seq,
                created_at
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
        })
    }

    pub fn list_messages(&self, session_id: &str) -> Result<Vec<Message>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, role, content, reasoning_content, tool_call_id, seq, created_at
             FROM messages WHERE session_id = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                reasoning_content: row.get(4)?,
                tool_call_id: row.get(5)?,
                seq: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
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

    pub fn list_tool_calls_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<ToolCallRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT tc.id, tc.message_id, tc.name, tc.args_json, tc.result_json, tc.status, tc.duration_ms, tc.created_at
             FROM tool_calls tc
             JOIN messages m ON m.id = tc.message_id
             WHERE m.session_id = ?1
             ORDER BY tc.created_at ASC",
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
            .add_message(&session.id, "user", Some("hello"), None, None)
            .unwrap();
        let assistant = store
            .add_message(&session.id, "assistant", Some("hi"), Some("thinking"), None)
            .unwrap();

        let messages = store.list_messages(&session.id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].reasoning_content.as_deref(), Some("thinking"));

        let updated = store
            .update_session(&session.id, Some("renamed"), Some("kimi-k2.6"), None, None)
            .unwrap();
        assert_eq!(updated.title, "renamed");
        assert_eq!(updated.model, "kimi-k2.6");

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
    fn settings_roundtrip() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        assert!(store.get_setting("theme").unwrap().is_none());
        store.set_setting("theme", "dark").unwrap();
        assert_eq!(store.get_setting("theme").unwrap().as_deref(), Some("dark"));
    }
}
