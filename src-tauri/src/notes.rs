use serde::Serialize;
use std::fs;
use std::path::PathBuf;

use crate::store;

/// 文件树节点（前端渲染用）
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    /// 相对路径（相对于 notes/ 目录）
    pub path: String,
    pub is_dir: bool,
    /// 目录的子节点（仅目录有，递归填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileEntry>>,
}

/// 获取 notes/ 目录的绝对路径
fn notes_dir() -> PathBuf {
    store::get_workspace_dir().join("notes")
}

/// 递归读取目录结构
fn read_dir_recursive(base: &PathBuf, rel: &str) -> Vec<FileEntry> {
    let dir = base.join(rel);
    let mut entries = Vec::new();

    let read = match fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let entry_rel = if rel.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", rel, name)
        };
        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        if is_dir {
            entries.push(FileEntry {
                name,
                path: entry_rel.clone(),
                is_dir: true,
                children: Some(read_dir_recursive(base, &entry_rel)),
            });
        } else if entry_rel.ends_with(".md") {
            entries.push(FileEntry {
                name,
                path: entry_rel,
                is_dir: false,
                children: None,
            });
        }
    }

    // 目录在前，文件在后；均按名称排序
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    entries
}

/// 列出 notes/ 目录的完整文件树
#[tauri::command]
pub fn list_note_tree() -> Vec<FileEntry> {
    let base = notes_dir();
    fs::create_dir_all(base.join("inbox")).ok();
    read_dir_recursive(&base, "")
}

/// 读取笔记内容
#[tauri::command]
pub fn read_note(path: String) -> Result<String, String> {
    let full = notes_dir().join(&path);
    // 安全检查：防止路径穿越
    let canonical = full.canonicalize().map_err(|e| e.to_string())?;
    let notes_root = notes_dir().canonicalize().unwrap_or_default();
    if !canonical.starts_with(&notes_root) {
        return Err("非法路径".into());
    }
    fs::read_to_string(&canonical).map_err(|e| e.to_string())
}

/// 写入笔记内容（自动创建父目录）
#[tauri::command]
pub fn write_note(path: String, content: String) -> Result<(), String> {
    let full = notes_dir().join(&path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&full, &content).map_err(|e| e.to_string())
}

/// 创建文件夹
#[tauri::command]
pub fn create_note_dir(path: String) -> Result<(), String> {
    let full = notes_dir().join(&path);
    fs::create_dir_all(&full).map_err(|e| e.to_string())
}

/// 删除文件或文件夹
#[tauri::command]
pub fn delete_note_entry(path: String) -> Result<(), String> {
    let full = notes_dir().join(&path);
    if full.is_dir() {
        fs::remove_dir_all(&full).map_err(|e| e.to_string())
    } else {
        fs::remove_file(&full).map_err(|e| e.to_string())
    }
}

/// 重命名文件或文件夹
#[tauri::command]
pub fn rename_note_entry(path: String, new_name: String) -> Result<(), String> {
    let full = notes_dir().join(&path);
    let parent = full.parent().unwrap_or(&full);
    let new_path = parent.join(&new_name);
    fs::rename(&full, &new_path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_serialization() {
        let entry = FileEntry {
            name: "test.md".to_string(),
            path: "inbox/test.md".to_string(),
            is_dir: false,
            children: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test.md"));
        assert!(json.contains("inbox/test.md"));
        assert!(!json.contains("children"));
    }

    #[test]
    fn test_dir_entry_serialization() {
        let entry = FileEntry {
            name: "inbox".to_string(),
            path: "inbox".to_string(),
            is_dir: true,
            children: Some(vec![]),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("inbox"));
        assert!(json.contains("children"));
    }
}
