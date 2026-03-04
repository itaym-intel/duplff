use crate::state::{AppState, TauriProgress};
use duplff_core::models::ScanConfig;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
pub struct TrashResult {
    pub count: usize,
    pub bytes_reclaimed: u64,
}

#[derive(Serialize)]
pub struct UndoResult {
    pub restored: usize,
    pub not_found: usize,
}

#[tauri::command]
pub async fn start_scan(
    config: ScanConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.scan_running.load(Ordering::Relaxed) {
        return Err("scan already running".into());
    }
    state.scan_running.store(true, Ordering::Relaxed);
    *state.report.lock().unwrap() = None;

    let report_store = state.report.clone();
    let scan_flag = state.scan_running.clone();

    std::thread::spawn(move || {
        let progress = TauriProgress::new(app.clone());
        match duplff_core::find_duplicates(&config, &progress) {
            Ok(report) => {
                *report_store.lock().unwrap() = Some(report.clone());
                let _ = app.emit("scan-complete", &report);
            }
            Err(e) => {
                let _ = app.emit("scan-error", e.to_string());
            }
        }
        scan_flag.store(false, Ordering::Relaxed);
    });

    Ok(())
}

#[tauri::command]
pub fn get_results(state: State<'_, AppState>) -> Option<duplff_core::models::DuplicateReport> {
    state.report.lock().unwrap().clone()
}

#[tauri::command]
pub fn trash_files(
    paths: Vec<std::path::PathBuf>,
    state: State<'_, AppState>,
) -> Result<TrashResult, String> {
    let mut trashed = Vec::new();
    let mut bytes_reclaimed = 0u64;

    for path in &paths {
        if let Ok(meta) = std::fs::metadata(path) {
            bytes_reclaimed += meta.len();
        }
        trash::delete(path).map_err(|e| format!("{}: {e}", path.display()))?;
        trashed.push(path.clone());
    }

    let log = duplff_core::actions::ActionLog {
        actions: trashed
            .iter()
            .map(|p| duplff_core::actions::ActionRecord {
                path: p.clone(),
                action: duplff_core::actions::ActionType::Trashed,
            })
            .collect(),
        bytes_reclaimed,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string(),
    };

    let _ = duplff_core::log_store::save_action_log(&log);
    *state.last_action_log.lock().unwrap() = Some(log);

    // Remove trashed files from the stored report
    if let Some(ref mut report) = *state.report.lock().unwrap() {
        for group in &mut report.groups {
            group.duplicates.retain(|d| !trashed.contains(&d.entry.path));
        }
        report.groups.retain(|g| !g.duplicates.is_empty());
        report.total_duplicates = report.groups.iter().map(|g| g.duplicates.len()).sum();
        report.total_wasted_bytes = report.groups.iter().map(|g| g.wasted_bytes()).sum();
    }

    Ok(TrashResult {
        count: trashed.len(),
        bytes_reclaimed,
    })
}

#[tauri::command]
pub fn undo_last(state: State<'_, AppState>) -> Result<UndoResult, String> {
    let log = state.last_action_log.lock().unwrap();
    let log = log.as_ref().ok_or("no action to undo")?;
    let result = duplff_core::actions::undo(log).map_err(|e| e.to_string())?;
    Ok(UndoResult {
        restored: result.restored.len(),
        not_found: result.not_found.len(),
    })
}

#[tauri::command]
pub fn export_json(state: State<'_, AppState>) -> Result<String, String> {
    let report = state.report.lock().unwrap();
    let report = report.as_ref().ok_or("no results to export")?;
    serde_json::to_string_pretty(report).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_csv(state: State<'_, AppState>) -> Result<String, String> {
    let report = state.report.lock().unwrap();
    let report = report.as_ref().ok_or("no results to export")?;
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["Group", "Status", "Path", "Size", "Hash"])
        .map_err(|e| e.to_string())?;
    for (i, group) in report.groups.iter().enumerate() {
        let hash = hex::encode(group.hash);
        wtr.write_record([
            &(i + 1).to_string(),
            "keep",
            &group.keep.entry.path.display().to_string(),
            &group.size.to_string(),
            &hash,
        ])
        .map_err(|e| e.to_string())?;
        for dup in &group.duplicates {
            wtr.write_record([
                &(i + 1).to_string(),
                "duplicate",
                &dup.entry.path.display().to_string(),
                &group.size.to_string(),
                &hash,
            ])
            .map_err(|e| e.to_string())?;
        }
    }
    String::from_utf8(wtr.into_inner().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_in_file_manager(path: std::path::PathBuf) -> Result<(), String> {
    let dir = if path.is_file() {
        path.parent().unwrap_or(&path).to_path_buf()
    } else {
        path
    };
    open::that(&dir).map_err(|e| e.to_string())
}
