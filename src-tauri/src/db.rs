use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn setup_test_db() -> Database {
        let dir = tempdir().expect("创建临时目录失败");
        let path = dir.path().to_str().expect("路径转换失败");
        Database::new(path).await.expect("数据库初始化失败")
    }

    #[tokio::test]
    async fn test_basic_flow() {
        let db = setup_test_db().await;

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
        let db = setup_test_db().await;

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
        let db = setup_test_db().await;

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
}
