use std::fs;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_log::log::info;

mod db;

/// 获取所有轮次
#[tauri::command]
async fn get_routines(state: State<'_, db::Database>) -> Result<Vec<db::Routine>, String> {
    state.get_routines().await.map_err(|e| e.to_string())
}

/// 创建轮次
#[tauri::command]
async fn create_routine(
    state: State<'_, db::Database>,
    name: String,
    desc: String,
) -> Result<i64, String> {
    state
        .create_routine(&name, &desc)
        .await
        .map_err(|e| e.to_string())
}

/// 删除轮次
#[tauri::command]
async fn delete_routine(state: State<'_, db::Database>, routine_id: i64) -> Result<(), String> {
    state
        .delete_routine(routine_id)
        .await
        .map_err(|e| e.to_string())
}

/// 更新轮次
#[tauri::command]
async fn update_routine(
    state: State<'_, db::Database>,
    routine_id: i64,
    name: String,
    desc: String,
) -> Result<(), String> {
    state
        .update_routine(routine_id, &name, &desc)
        .await
        .map_err(|e| e.to_string())
}

/// 获取轮次下的动作
#[tauri::command]
async fn get_exercises(
    state: State<'_, db::Database>,
    routine_id: i64,
) -> Result<Vec<db::Exercise>, String> {
    state
        .get_exercises(routine_id)
        .await
        .map_err(|e| e.to_string())
}

/// 添加动作
#[tauri::command]
async fn add_exercise(
    state: State<'_, db::Database>,
    routine_id: i64,
    name: String,
    sets: i64,
    reps: String,
    note: String,
    unit: String,
) -> Result<i64, String> {
    state
        .add_exercise(routine_id, &name, sets, &reps, &note, &unit)
        .await
        .map_err(|e| e.to_string())
}

/// 删除动作
#[tauri::command]
async fn delete_exercise(state: State<'_, db::Database>, exercise_id: i64) -> Result<(), String> {
    state
        .delete_exercise(exercise_id)
        .await
        .map_err(|e| e.to_string())
}

/// 更新动作
#[tauri::command]
async fn update_exercise(
    state: State<'_, db::Database>,
    exercise_id: i64,
    name: String,
    sets: i64,
    reps: String,
    note: String,
    unit: String,
) -> Result<(), String> {
    state
        .update_exercise(exercise_id, &name, sets, &reps, &note, &unit)
        .await
        .map_err(|e| e.to_string())
}

/// 记录一次最大重量
#[tauri::command]
async fn add_record(
    state: State<'_, db::Database>,
    exercise_id: i64,
    weight: f64,
    reps: Option<i64>,
) -> Result<i64, String> {
    state
        .add_record(exercise_id, weight, reps)
        .await
        .map_err(|e| e.to_string())
}

/// 删除记录
#[tauri::command]
async fn delete_record(state: State<'_, db::Database>, record_id: i64) -> Result<(), String> {
    state
        .delete_record(record_id)
        .await
        .map_err(|e| e.to_string())
}

/// 分页获取记录
#[tauri::command]
async fn get_all_records(
    state: State<'_, db::Database>,
    exercise_id: i64,
) -> Result<Vec<db::Record>, String> {
    state
        .get_all_records(exercise_id)
        .await
        .map_err(|e| e.to_string())
}

/// 更新记录
#[tauri::command]
async fn update_record(
    state: State<'_, db::Database>,
    record_id: i64,
    weight: f64,
    reps: Option<i64>,
) -> Result<(), String> {
    state
        .update_record(record_id, weight, reps)
        .await
        .map_err(|e| e.to_string())
}

/// 获取单个动作的统计信息
#[tauri::command]
async fn get_exercise_stats(
    state: State<'_, db::Database>,
    exercise_id: i64,
) -> Result<db::ExerciseStats, String> {
    state
        .get_exercise_stats(exercise_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_database(
    state: State<'_, db::Database>,
    destination: String,
) -> Result<(), String> {
    let source = state.get_db_path();
    fs::copy(source, &destination).map_err(|e| format!("复制文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn import_database(
    state: State<'_, db::Database>,
    source: String,
) -> Result<(), String> {
    let destination = state.get_db_path();

    // 关闭数据库连接，释放文件锁
    state
        .close()
        .await
        .map_err(|e| format!("关闭数据库失败: {}", e))?;

    // 复制备份文件覆盖原文件
    fs::copy(&source, destination).map_err(|e| format!("复制文件失败: {}", e))?;

    // 提示用户重启（可立即重启）
    Ok(())
}

#[tauri::command]
async fn restart_app(app_handle: AppHandle) {
    app_handle.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            let app_data_dir_str = app_data_dir.to_str().expect("路径转换失败");

            info!("数据库路径: {}", app_data_dir_str);

            // 初始化数据库
            let db =
                tauri::async_runtime::block_on(async { db::Database::new(app_data_dir_str).await })
                    .expect("数据库初始化失败");

            app.manage(db);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_routines,
            create_routine,
            delete_routine,
            update_routine,
            get_exercises,
            add_exercise,
            delete_exercise,
            update_exercise,
            add_record,
            delete_record,
            get_all_records,
            update_record,
            get_exercise_stats,
            export_database,
            import_database,
            restart_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
