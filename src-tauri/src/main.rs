// 仅在 Release 模式下隐藏 Windows 控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashSet;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_notification::NotificationExt;

pub(crate) mod ai;
pub(crate) mod notes;
pub(crate) mod prompt;
pub(crate) mod store;

mod commands;

use commands::{resolve_ai_settings, AppState};

// ── 窗口管理命令 ──────────────────────────────

/// 切换到悬浮小窗模式（隐藏主窗口）
#[tauri::command]
fn show_floating_window(app: tauri::AppHandle) -> Result<(), String> {
    let float_win = app
        .get_webview_window("floating")
        .ok_or("floating window not found")?;
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.hide().map_err(|e| e.to_string())?;
    }
    float_win.show().map_err(|e| e.to_string())?;
    float_win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

/// 切换回主窗口模式（隐藏悬浮窗）
#[tauri::command]
fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    let main_win = app
        .get_webview_window("main")
        .ok_or("main window not found")?;
    if let Some(float_win) = app.get_webview_window("floating") {
        float_win.hide().map_err(|e| e.to_string())?;
    }
    main_win.show().map_err(|e| e.to_string())?;
    main_win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

// ── 提醒设置命令 ──────────────────────────────

/// 设置任务到期提醒的提前分钟数（0 表示关闭提醒）
#[tauri::command]
fn set_reminder_minutes(state: tauri::State<AppState>, minutes: u32) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.reminder_minutes = minutes;
    store::save_config(&config)
}

/// 获取当前提醒设置（分钟数）
#[tauri::command]
fn get_reminder_minutes(state: tauri::State<AppState>) -> u32 {
    state.config.lock().unwrap().reminder_minutes
}

// ── AI 设置命令 ──────────────────────────────

/// 获取 AI 配置状态（是否已配置可用供应商）
#[tauri::command]
fn get_ai_settings_all(state: tauri::State<AppState>) -> serde_json::Value {
    let config = state.config.lock().unwrap();
    serde_json::json!({
        "active_vendor_id": config.active_vendor_id,
        "has_enabled_vendor": config.vendors.iter().any(|v| v.enabled),
    })
}

// ── AI 供应商命令 ──────────────────────────────

/// 获取所有供应商
#[tauri::command]
fn get_vendors(state: tauri::State<AppState>) -> Vec<store::Vendor> {
    state.config.lock().unwrap().vendors.clone()
}

/// 添加供应商
#[tauri::command]
fn add_vendor(
    state: tauri::State<AppState>,
    vendor: store::Vendor,
) -> Result<store::Vendor, String> {
    let mut config = state.config.lock().unwrap();
    config.vendors.push(vendor.clone());
    store::save_config(&config)?;
    Ok(vendor)
}

/// 更新供应商
#[tauri::command]
fn update_vendor(state: tauri::State<AppState>, vendor: store::Vendor) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    if let Some(v) = config.vendors.iter_mut().find(|v| v.id == vendor.id) {
        *v = vendor;
    }
    store::save_config(&config)
}

/// 删除供应商
#[tauri::command]
fn delete_vendor(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.vendors.retain(|v| v.id != id);
    // 如果删除的是当前激活的供应商，清除激活状态
    if config.active_vendor_id.as_deref() == Some(&id) {
        config.active_vendor_id = None;
    }
    store::save_config(&config)
}

/// 设置激活的供应商
#[tauri::command]
fn set_active_vendor(state: tauri::State<AppState>, id: Option<String>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.active_vendor_id = id;
    store::save_config(&config)
}

// resolve_ai_settings 已迁移到 commands 模块

// ── AI 功能命令 ──────────────────────────────

/// 自然语言解析输入
#[tauri::command]
async fn ai_parse_input(
    state: tauri::State<'_, AppState>,
    text: String,
) -> Result<ai::ParsedTask, String> {
    let (settings, existing_tags) = {
        let config = state.config.lock().unwrap();
        let settings = resolve_ai_settings(&config)?;
        let data = state.data.lock().unwrap();
        let existing_tags: Vec<String> = data.tasks.iter().flat_map(|t| t.tags.clone()).collect();
        (settings, existing_tags)
    };
    ai::parse_input(&settings, &text, &existing_tags).await
}

/// 今日聚焦建议
#[tauri::command]
async fn ai_daily_focus(state: tauri::State<'_, AppState>) -> Result<ai::FocusSuggestion, String> {
    let (settings, tasks) = {
        let config = state.config.lock().unwrap();
        let settings = resolve_ai_settings(&config)?;
        let data = state.data.lock().unwrap();
        let tasks = data.tasks.clone();
        (settings, tasks)
    };
    ai::daily_focus(&settings, &tasks).await
}

/// 任务智能拆解
#[tauri::command]
async fn ai_decompose(
    state: tauri::State<'_, AppState>,
    task_id: String,
) -> Result<Vec<ai::SubTask>, String> {
    let (settings, task_title, existing_subtasks) = {
        let config = state.config.lock().unwrap();
        let settings = resolve_ai_settings(&config)?;
        let data = state.data.lock().unwrap();
        let task_title = data
            .tasks
            .iter()
            .find(|t| t.id == task_id)
            .map(|t| t.title.clone())
            .ok_or("任务不存在")?;
        let existing_subtasks: Vec<String> = data
            .tasks
            .iter()
            .filter(|t| t.parent_id.as_deref() == Some(&task_id))
            .map(|t| t.title.clone())
            .collect();
        (settings, task_title, existing_subtasks)
    };
    ai::decompose(&settings, &task_title, &existing_subtasks).await
}

/// 过期任务处理建议
#[tauri::command]
async fn ai_overdue_suggest(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ai::OverdueSuggestion>, String> {
    let (settings, overdue) = {
        let config = state.config.lock().unwrap();
        let settings = resolve_ai_settings(&config)?;
        let data = state.data.lock().unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let overdue: Vec<store::Task> = data
            .tasks
            .iter()
            .filter(|t| !t.completed && t.due_date.as_deref().is_some_and(|d| d < today.as_str()))
            .cloned()
            .collect();
        (settings, overdue)
    };
    ai::overdue_suggest(&settings, &overdue).await
}

/// AI 助手自由对话
#[tauri::command]
async fn ai_chat(state: tauri::State<'_, AppState>, message: String) -> Result<String, String> {
    let (settings, tasks) = {
        let config = state.config.lock().unwrap();
        let settings = resolve_ai_settings(&config)?;
        let data = state.data.lock().unwrap();
        let tasks = data.tasks.clone();
        (settings, tasks)
    };
    ai::chat(&settings, &message, &tasks).await
}

/// AI 解释 JSON 结构
#[tauri::command]
async fn ai_json_explain(
    state: tauri::State<'_, AppState>,
    json_text: String,
) -> Result<String, String> {
    let settings = {
        let config = state.config.lock().unwrap();
        resolve_ai_settings(&config)?
    };
    ai::json_explain(&settings, &json_text).await
}

/// AI 生成正则表达式
#[tauri::command]
async fn ai_regex_generate(
    state: tauri::State<'_, AppState>,
    description: String,
) -> Result<String, String> {
    let settings = {
        let config = state.config.lock().unwrap();
        resolve_ai_settings(&config)?
    };
    ai::regex_generate(&settings, &description).await
}

/// 获取当前主题设置
#[tauri::command]
fn get_theme(state: tauri::State<AppState>) -> String {
    state.config.lock().unwrap().theme.clone()
}

/// 设置主题模式并持久化
#[tauri::command]
fn set_theme(state: tauri::State<AppState>, theme: String) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.theme = theme;
    store::save_config(&config)
}

// ── 模块配置命令 ──────────────────────────────

/// 获取所有模块的启用状态
#[tauri::command]
fn get_module_enabled(state: tauri::State<AppState>) -> std::collections::HashMap<String, bool> {
    state.config.lock().unwrap().module_enabled.clone()
}

/// 设置单个模块的启用状态
#[tauri::command]
fn set_module_enabled(
    state: tauri::State<AppState>,
    module_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    config.module_enabled.insert(module_id, enabled);
    store::save_config(&config)
}

/// 应用入口：初始化存储、注册命令、启动后台提醒线程
fn main() {
    let (data, config) = store::initialize();
    prompt::create_defaults();
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            data: Mutex::new(data),
            config: Mutex::new(config),
            notified_today: Mutex::new(HashSet::new()),
        })
        // 注册所有前端可调用的命令
        .invoke_handler(tauri::generate_handler![
            commands::tasks::get_tasks,
            commands::tasks::add_task,
            commands::tasks::toggle_task,
            commands::tasks::toggle_daily_task,
            commands::tasks::update_task,
            commands::tasks::delete_task,
            commands::tasks::clear_completed,
            commands::tasks::get_tasks_by_date,
            commands::tasks::get_all_tags,
            commands::tasks::delete_tag,
            commands::tasks::get_daily_completions,
            set_reminder_minutes,
            get_reminder_minutes,
            get_ai_settings_all,
            get_vendors,
            add_vendor,
            update_vendor,
            delete_vendor,
            set_active_vendor,
            get_theme,
            set_theme,
            get_module_enabled,
            set_module_enabled,
            notes::list_note_tree,
            notes::read_note,
            notes::write_note,
            notes::create_note_dir,
            notes::delete_note_entry,
            notes::rename_note_entry,
            notes::get_notes_directory,
            notes::set_notes_directory,
            show_floating_window,
            show_main_window,
            ai_parse_input,
            ai_daily_focus,
            ai_decompose,
            ai_overdue_suggest,
            ai_chat,
            ai_json_explain,
            ai_regex_generate,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            // 后台线程：每分钟检查一次到期任务并推送系统通知
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(60));

                    let reminder;
                    let tasks_snapshot: Vec<(String, String, Option<String>)>;
                    {
                        let state = handle.state::<AppState>();
                        let config = state.config.lock().unwrap();
                        reminder = config.reminder_minutes;
                        // 提醒已关闭，跳过本轮检查
                        if reminder == 0 {
                            continue;
                        }
                        let data = state.data.lock().unwrap();
                        // 快照当前未完成且有截止日期的任务（避免长时间持有锁）
                        tasks_snapshot = data
                            .tasks
                            .iter()
                            .filter(|t| !t.completed && t.due_date.is_some())
                            .map(|t| (t.id.clone(), t.title.clone(), t.due_date.clone()))
                            .collect();
                    }

                    let local_now = chrono::Local::now();
                    let today = local_now.format("%Y-%m-%d").to_string();

                    // 日期变更时重置已通知集合
                    {
                        let state = handle.state::<AppState>();
                        let mut notified = state.notified_today.lock().unwrap();
                        if !notified.contains(&today) {
                            notified.clear();
                            notified.insert(today.clone());
                        }
                    }

                    let local_offset = local_now.offset();

                    for (task_id, title, due_date_opt) in &tasks_snapshot {
                        let due_date = match due_date_opt {
                            Some(d) => d,
                            None => continue,
                        };
                        // 使用本地时区构建截止日 23:59:59，而非硬编码 UTC
                        let due_str = format!("{}T23:59:59{}", due_date, local_offset);
                        let due_time = match chrono::DateTime::parse_from_rfc3339(&due_str) {
                            Ok(t) => t,
                            Err(_) => continue,
                        };
                        let diff_secs = due_time.timestamp() - local_now.timestamp();
                        let diff_min = diff_secs / 60;
                        // 仅在距离截止时间还有剩余分钟数且小于等于提醒阈值时通知
                        if diff_min > 0 && diff_min <= reminder as i64 {
                            let state = handle.state::<AppState>();
                            let mut notified = state.notified_today.lock().unwrap();
                            // 当天已通知过的任务不再重复提醒
                            if !notified.contains(task_id) {
                                notified.insert(task_id.clone());
                                drop(notified);
                                let minutes_left = diff_min;
                                let _ = handle
                                    .notification()
                                    .builder()
                                    .title("⏰ 任务即将到期")
                                    .body(format!("\"{}\" 将在 {} 分钟后到期", title, minutes_left))
                                    .show();
                            }
                        }
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
