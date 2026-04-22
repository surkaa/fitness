use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};
use std::fs;
use std::path::Path;

// 轮次 (比如: "一轮次: 胸肩")
#[derive(Debug, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Routine {
    pub id: i32,
    /// 轮次名称
    pub name: String,
    /// 可选描述
    pub description: Option<String>,
}

// 动作 (比如: "杠铃卧推", 包含单位配置)
#[derive(Debug, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub id: i32,
    pub routine_id: i32,
    /// 动作名称
    pub name: String,
    /// 计划做几组
    pub target_sets: i32,
    /// 计划做几个 (用String是为了支持 "8-12" 这种写法)
    pub target_reps: String,
    /// 详细描述 (离心要慢...)
    pub note: Option<String>,
    /// 记录时的单位 'kg', 'lb', 'plate' (多少片)
    pub unit: String,
}

// 记录 (具体每一次的重量)
#[derive(Debug, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: i32,
    pub exercise_id: i32,
    /// 记录时间
    #[serde(with = "chrono::serde::ts_milliseconds")]
    #[specta(type = f64)]
    pub created_at: DateTime<Utc>,
    /// 重量 (数值，配合Exercise里的unit使用)
    pub weight: f64,
    /// 实际做了几个
    pub reps: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseStats {
    pub exercise_id: i32,
    pub total_records: i32,
    pub max_weight: Option<f64>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    #[specta(type = f64)]
    pub last_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportData {
    pub routines: Vec<Routine>,
    pub exercises: Vec<Exercise>,
    pub records: Vec<Record>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, specta::Type, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DailyRecordCount {
    /// 格式为 "YYYY-MM-DD"
    pub day: i32,
    /// 当天的记录总数
    pub count: i32,
}

pub struct Database {
    pool: Pool<Sqlite>,
    path: String,
}

impl Database {
    // 初始化数据库
    pub async fn new(app_dir: &str) -> Result<Self, sqlx::Error> {
        // 确保目录存在
        if !Path::new(app_dir).exists() {
            fs::create_dir_all(app_dir).expect("未能创建应用目录");
        }

        let db_path = format!("{}/fitness_lite.db", app_dir);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}?mode=rwc", db_path))
            .await?;

        let db = Database {
            pool,
            path: db_path,
        };
        db.init_tables().await?;
        Ok(db)
    }

    pub fn get_db_path(&self) -> &str {
        &self.path
    }

    pub async fn close(&self) -> Result<(), sqlx::Error> {
        self.pool.close().await;
        Ok(())
    }

    /// 建表 SQL
    async fn init_tables(&self) -> Result<(), sqlx::Error> {
        // 轮次表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS routines (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    `name` TEXT NOT NULL,
                    description TEXT
                );",
        )
        .execute(&self.pool)
        .await?;

        // 动作表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS exercises (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    routine_id INTEGER NOT NULL,
                    `name` TEXT NOT NULL,
                    target_sets INTEGER DEFAULT 4,
                    target_reps TEXT DEFAULT '10-12',
                    note TEXT DEFAULT '',
                    unit TEXT DEFAULT 'kg',
                    FOREIGN KEY(routine_id) REFERENCES routines(id) ON DELETE CASCADE
                );",
        )
        .execute(&self.pool)
        .await?;

        // 记录表
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS records (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    exercise_id INTEGER NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    weight REAL NOT NULL,
                    reps INTEGER DEFAULT NULL,
                    FOREIGN KEY(exercise_id) REFERENCES exercises(id) ON DELETE CASCADE
                );",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取所有轮次
    pub async fn get_routines(&self) -> Result<Vec<Routine>, sqlx::Error> {
        sqlx::query_as::<_, Routine>("SELECT * FROM routines ORDER BY id")
            .fetch_all(&self.pool)
            .await
    }

    /// 创建轮次
    pub async fn create_routine(&self, name: &str, desc: &str) -> Result<i32, sqlx::Error> {
        let id = sqlx::query("INSERT INTO routines (name, description) VALUES (?, ?)")
            .bind(name)
            .bind(desc)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id as i32)
    }

    /// 删除轮次
    pub async fn delete_routine(&self, routine_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM routines WHERE id = ?")
            .bind(routine_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 更新轮次
    pub async fn update_routine(
        &self,
        routine_id: i32,
        name: &str,
        desc: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE routines SET name = ?, description = ? WHERE id = ?")
            .bind(name)
            .bind(desc)
            .bind(routine_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取某个轮次下的所有动作
    pub async fn get_exercises(&self, routine_id: i32) -> Result<Vec<Exercise>, sqlx::Error> {
        sqlx::query_as::<_, Exercise>("SELECT * FROM exercises WHERE routine_id = ? ORDER BY id")
            .bind(routine_id)
            .fetch_all(&self.pool)
            .await
    }

    /// 添加动作
    pub async fn add_exercise(
        &self,
        routine_id: i32,
        name: &str,
        sets: i32,
        reps: &str,
        note: &str,
        unit: &str,
    ) -> Result<i32, sqlx::Error> {
        let id = sqlx::query(
            "INSERT INTO exercises (routine_id, name, target_sets, target_reps, note, unit)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(routine_id)
        .bind(name)
        .bind(sets)
        .bind(reps)
        .bind(note)
        .bind(unit)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();
        Ok(id as i32)
    }

    /// 删除动作
    pub async fn delete_exercise(&self, exercise_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM exercises WHERE id = ?")
            .bind(exercise_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 更新动作
    pub async fn update_exercise(
        &self,
        exercise_id: i32,
        name: &str,
        sets: i32,
        reps: &str,
        note: &str,
        unit: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE exercises SET name = ?, target_sets = ?, target_reps = ?, note = ?, unit = ? WHERE id = ?",
        )
        .bind(name)
        .bind(sets)
        .bind(reps)
        .bind(note)
        .bind(unit)
        .bind(exercise_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 记录一次数据
    pub async fn add_record(
        &self,
        exercise_id: i32,
        weight: f64,
        reps: Option<i32>,
    ) -> Result<i32, sqlx::Error> {
        let id = sqlx::query("INSERT INTO records (exercise_id, weight, reps) VALUES (?, ?, ?)")
            .bind(exercise_id)
            .bind(weight)
            .bind(reps)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id as i32)
    }

    /// 删除一条记录
    pub async fn delete_record(&self, record_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM records WHERE id = ?")
            .bind(record_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 分页获取某个动作的记录
    pub async fn get_all_records(&self, exercise_id: i32) -> Result<Vec<Record>, sqlx::Error> {
        sqlx::query_as::<_, Record>(
            "SELECT * FROM records WHERE exercise_id = ? ORDER BY created_at DESC, id DESC",
        )
        .bind(exercise_id)
        .fetch_all(&self.pool)
        .await
    }

    /// 更新记录
    pub async fn update_record(
        &self,
        record_id: i32,
        weight: f64,
        reps: Option<i32>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE records SET weight = ?, reps = ? WHERE id = ?")
            .bind(weight)
            .bind(reps)
            .bind(record_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取单个动作的统计信息
    pub async fn get_exercise_stats(&self, exercise_id: i32) -> Result<ExerciseStats, sqlx::Error> {
        // 总记录数
        let (total_records,): (i32,) =
            sqlx::query_as("SELECT COUNT(*) FROM records WHERE exercise_id = ?")
                .bind(exercise_id)
                .fetch_one(&self.pool)
                .await?;

        // 最大重量
        let max_weight: Option<f64> =
            sqlx::query_scalar("SELECT MAX(weight) FROM records WHERE exercise_id = ?")
                .bind(exercise_id)
                .fetch_one(&self.pool)
                .await
                .unwrap_or(None);

        // 最后训练日期
        let last_date: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT created_at FROM records WHERE exercise_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(exercise_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(None);

        Ok(ExerciseStats {
            exercise_id,
            total_records,
            max_weight,
            last_date,
        })
    }

    /// 获取某个动作的常用 reps 值（最近 N 条记录中非空的、去重后的 reps）
    pub async fn get_common_reps(&self, exercise_id: i32) -> Result<Vec<i32>, sqlx::Error> {
        // 查询最近 50 条记录中非空的 reps，按出现频率降序，取前 5 个不同的值
        // 也可以简单地取所有不重复的 reps（限制数量）
        let reps = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT reps FROM records
             WHERE exercise_id = ? AND reps IS NOT NULL
             ORDER BY created_at DESC LIMIT 50",
        )
        .bind(exercise_id)
        .fetch_all(&self.pool)
        .await?;

        // 提取非空值，去重，限制最多 6 个，并按数值排序（可选）
        let mut unique: Vec<i32> = reps
            .into_iter()
            .flatten() // 过滤掉 None
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        unique.sort(); // 按数值升序，更整齐
        unique.truncate(6); // 最多显示 6 个候选
        Ok(unique)
    }

    /// 给一个动作的记录数据倍增和添加常量，用于变更记录方式（记录片数=>记录kg等等）
    pub async fn transform_records(
        &self,
        exercise_id: i32,
        is_add: bool,
        a: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(if is_add {
            "UPDATE records SET weight = weight + ? WHERE exercise_id = ?"
        } else {
            "UPDATE records SET weight = weight * ? WHERE exercise_id = ?"
        })
        .bind(a)
        .bind(exercise_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 导出指定时间段内的所有数据
    pub async fn export_some_data(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<ExportData, sqlx::Error> {
        // 确定指定时间段内有哪些记录
        let records = sqlx::query_as::<_, Record>(
            "SELECT * FROM records
                    WHERE strftime('%s', created_at) >= strftime('%s', ?)
                    AND strftime('%s', created_at) <= strftime('%s', ?)",
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        // 从记录中提取动作ID并去重
        let exercise_ids: Vec<i32> = records
            .iter()
            .map(|r| r.exercise_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        // 获取动作信息
        let mut exercises: Vec<Exercise> = Vec::new();
        for exercise_id in exercise_ids {
            let exercise = sqlx::query_as::<_, Exercise>("SELECT * FROM exercises WHERE id = ?")
                .bind(exercise_id)
                .fetch_one(&self.pool)
                .await?;
            exercises.push(exercise);
        }

        // 从动作信息获取轮次ID并去重
        let routine_ids: Vec<i32> = exercises
            .iter()
            .map(|e| e.routine_id)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        // 获取轮次信息
        let mut routines: Vec<Routine> = Vec::new();
        for routine_id in routine_ids {
            let routine = sqlx::query_as::<_, Routine>("SELECT * FROM routines WHERE id = ?")
                .bind(routine_id)
                .fetch_one(&self.pool)
                .await?;
            routines.push(routine);
        }
        // 组装导出数据
        Ok(ExportData {
            records,
            exercises,
            routines,
        })
    }

    /// 获取某年某月下每天有几次记录
    pub async fn get_daily_record_count(
        &self,
        year: i32,
        month: u32,
    ) -> Result<Vec<DailyRecordCount>, sqlx::Error> {
        sqlx::query_as::<_, DailyRecordCount>(
            "SELECT
            CAST(strftime('%d', created_at) AS INTEGER) as day,
            CAST(COUNT(*) AS INTEGER) as count
         FROM records
         WHERE strftime('%Y-%m', created_at) = ?
         GROUP BY day
         ORDER BY day",
        )
        .bind(format!("{:04}-{:02}", year, month))
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, Timelike};
    use std::time::Duration;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_basic_flow() {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        let db = Database::new(path).await.expect("数据库初始化失败");

        // 1. 创建轮次
        let r_id = db.create_routine("推胸日", "周一练").await.unwrap();
        assert!(r_id > 0);

        // 2. 添加动作
        let e_id = db
            .add_exercise(r_id, "卧推", 5, "5x5", "重", "kg")
            .await
            .unwrap();
        assert!(e_id > 0);

        // 3. 添加记录
        let rec_id = db.add_record(e_id, 100.0, Some(5)).await.unwrap();
        assert!(rec_id > 0);

        // 4. 验证数据
        let records = db.get_all_records(e_id).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].weight, 100.0);
        assert_eq!(records[0].reps, Some(5));
    }

    #[tokio::test]
    async fn test_cascade_delete() {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        let db = Database::new(path).await.expect("数据库初始化失败");

        // 1. 建立层级数据：轮次 -> 动作 -> 记录
        let r_id = db.create_routine("背部", "").await.unwrap();
        let e_id = db
            .add_exercise(r_id, "引体向上", 4, "力竭", "", "个")
            .await
            .unwrap();
        db.add_record(e_id, 0.0, Some(10)).await.unwrap();
        db.add_record(e_id, 0.0, Some(12)).await.unwrap();

        // 确认记录存在
        let recs_before = db.get_all_records(e_id).await.unwrap();
        assert_eq!(recs_before.len(), 2);

        // 2. 删除最顶层的【轮次】
        db.delete_routine(r_id).await.unwrap();

        // 3. 验证级联删除是否生效
        // 查动作：应该为空
        let exercises = db.get_exercises(r_id).await.unwrap();
        assert!(exercises.is_empty(), "动作应该被级联删除");

        // 查记录：应该为空
        let recs_after = db.get_all_records(e_id).await.unwrap();
        assert!(recs_after.is_empty(), "记录应该被级联删除");
    }

    #[tokio::test]
    async fn test_foreign_key_constraint() {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        let db = Database::new(path).await.expect("数据库初始化失败");

        // 创建轮次和动作
        let r_id = db.create_routine("肩部", "").await.unwrap();
        let e_id = db
            .add_exercise(r_id, "推举", 4, "8-12", "", "kg")
            .await
            .unwrap();

        // 删除轮次，应该级联删除动作
        db.delete_routine(r_id).await.unwrap();

        // 尝试添加记录到已删除的动作，应该失败
        let result = db.add_record(e_id, 50.0, Some(10)).await;
        assert!(result.is_err(), "应该无法为已删除的动作添加记录");
        if let Err(e) = result {
            println!("{}", e);
        }
    }

    #[tokio::test]
    async fn test_export_data() {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        let db = Database::new(path).await.expect("数据库初始化失败");

        // init data
        let r1_id = db.create_routine("背部", "").await.unwrap();
        let e1_id = db
            .add_exercise(r1_id, "引体向上", 4, "力竭", "", "个")
            .await
            .unwrap();
        db.add_record(e1_id, 0.0, Some(10)).await.unwrap();
        db.add_record(e1_id, 0.0, Some(12)).await.unwrap();
        // 手动延迟1秒
        tokio::time::sleep(Duration::from_secs(2)).await;
        let start_time = Utc::now();
        tokio::time::sleep(Duration::from_secs(1)).await;
        let r2_id = db.create_routine("胸部", "").await.unwrap();
        let e2_id = db
            .add_exercise(r2_id, "卧推", 5, "5x5", "重", "kg")
            .await
            .unwrap();
        db.add_record(e2_id, 50.0, Some(10)).await.unwrap();
        db.add_record(e2_id, 50.0, Some(12)).await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
        let end_time = Utc::now();
        println!("start_time: {:#?}", start_time);
        println!("end_time: {:#?}", end_time);
        println!("r1_id: {:#?}, r2_id: {:#?}", r1_id, r2_id);
        println!("e1_id: {:#?}, e2_id: {:#?}", e1_id, e2_id);
        let query_routines = db.get_routines().await.unwrap();
        println!("query_routines: {:#?}", query_routines);
        let query_exercises = db.get_exercises(r2_id).await.unwrap();
        println!("query_exercises: {:#?}", query_exercises);
        let query_records = db.get_all_records(e2_id).await.unwrap();
        println!("query_records: {:#?}", query_records);
        let export_data = db.export_some_data(start_time, end_time).await.unwrap();

        // verify export data
        assert_eq!(export_data.records.len(), 2);
        assert_eq!(export_data.exercises.len(), 1);
        assert_eq!(export_data.routines.len(), 1);
    }

    #[tokio::test]
    async fn test_record_update_date() {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        let db = Database::new(path).await.expect("数据库初始化失败");

        // init data
        let r_id = db.create_routine("肩部", "").await.unwrap();
        let routines = db.get_routines().await.unwrap();
        assert_eq!(routines.len(), 1);
        assert_eq!(routines[0].id, r_id);
        assert_eq!(routines[0].name, "肩部");
        let e_id = db
            .add_exercise(r_id, "推举", 4, "8-12", "", "kg")
            .await
            .unwrap();
        let exercises = db.get_exercises(r_id).await.unwrap();
        assert_eq!(exercises.len(), 1);
        assert_eq!(exercises[0].id, e_id);
        assert_eq!(exercises[0].name, "推举");
        let add_time = Utc::now();
        let rec_id = db.add_record(e_id, 50.0, Some(10)).await.unwrap();
        let records = db.get_all_records(e_id).await.unwrap();
        println!("records: {:#?}", records);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, rec_id);
        assert_eq!(records[0].weight, 50.0);
        assert_eq!(records[0].reps, Some(10));
        let cur_time = Utc::now();
        println!("add_time: {:#?}, cur_time: {:#?}", add_time, cur_time);
        assert!(add_time.second().abs_diff(cur_time.second()) < 1);

        // update time
        let target_time =
            NaiveDateTime::parse_from_str("2023-11-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        sqlx::query("UPDATE records SET created_at = ? WHERE id = ?")
            .bind(target_time.and_utc())
            .bind(rec_id)
            .execute(&db.pool)
            .await
            .unwrap();

        let records = db.get_all_records(e_id).await.unwrap();
        println!("records: {:#?}", records);
        assert_eq!(records[0].created_at, target_time.and_utc());
    }

    #[tokio::test]
    async fn test_get_daily_record_count() {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        let db = Database::new(path).await.expect("数据库初始化失败");

        // 1. 准备基础数据
        let r_id = db.create_routine("胸部", "基础计划").await.unwrap();
        let e_id = db
            .add_exercise(r_id, "卧推", 4, "10", "", "kg")
            .await
            .unwrap();

        // 2. 添加记录（初始创建时间都会是 CURRENT_TIMESTAMP 即当天）
        let rec1 = db.add_record(e_id, 40.0, Some(10)).await.unwrap();
        let rec2 = db.add_record(e_id, 40.0, Some(10)).await.unwrap();
        let rec3 = db.add_record(e_id, 45.0, Some(8)).await.unwrap();
        let rec4 = db.add_record(e_id, 50.0, Some(5)).await.unwrap();
        let rec5_out_of_bounds = db.add_record(e_id, 50.0, Some(5)).await.unwrap();

        // 3. 修改记录时间：通过直接操作底层的 db.pool 来篡改 created_at
        // 因为测试模块使用了 `use super::*`，所以可以合法访问父模块的私有字段 db.pool
        let update_time_query = "UPDATE records SET created_at = ? WHERE id = ?";

        // -> 归属到 2026年4月5日（2条记录）
        sqlx::query(update_time_query)
            .bind("2026-04-05 09:00:00")
            .bind(rec1)
            .execute(&db.pool)
            .await
            .unwrap();
        sqlx::query(update_time_query)
            .bind("2026-04-05 18:30:00")
            .bind(rec2)
            .execute(&db.pool)
            .await
            .unwrap();

        // -> 归属到 2026年4月12日（1条记录）
        sqlx::query(update_time_query)
            .bind("2026-04-12 14:00:00")
            .bind(rec3)
            .execute(&db.pool)
            .await
            .unwrap();

        // -> 归属到 2026年4月28日（1条记录）
        sqlx::query(update_time_query)
            .bind("2026-04-28 20:00:00")
            .bind(rec4)
            .execute(&db.pool)
            .await
            .unwrap();

        // -> 归属到 2026年5月1日（1条记录，用于验证它不会被 4 月的查询统计进去）
        sqlx::query(update_time_query)
            .bind("2026-05-01 10:00:00")
            .bind(rec5_out_of_bounds)
            .execute(&db.pool)
            .await
            .unwrap();

        // 4. 执行测试：查询 2026 年 4 月的数据
        let daily_counts = db.get_daily_record_count(2026, 4).await.unwrap();

        // 5. 验证结果
        assert_eq!(daily_counts.len(), 3, "2026年4月应该只有3天包含训练记录");

        assert_eq!(daily_counts[0].day, 5);
        assert_eq!(daily_counts[0].count, 2);

        assert_eq!(daily_counts[1].day, 12);
        assert_eq!(daily_counts[1].count, 1);

        assert_eq!(daily_counts[2].day, 28);
        assert_eq!(daily_counts[2].count, 1);
    }
}
