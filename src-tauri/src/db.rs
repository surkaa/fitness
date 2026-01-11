use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};
use std::fs;
use std::path::Path;

// 轮次 (比如: "一轮次: 胸肩")
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Routine {
    pub id: i64,
    /// 轮次名称
    pub name: String,
    /// 可选描述
    pub description: Option<String>,
}

// 动作 (比如: "杠铃卧推", 包含单位配置)
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub id: i64,
    pub routine_id: i64,
    /// 动作名称
    pub name: String,
    /// 计划做几组
    pub target_sets: i64,
    /// 计划做几个 (用String是为了支持 "8-12" 这种写法)
    pub target_reps: String,
    /// 详细描述 (离心要慢...)
    pub note: Option<String>,
    /// 记录时的单位 'kg', 'lb', 'plate' (多少片)
    pub unit: String,
}

// 记录 (具体每一次的重量)
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub id: i64,
    pub exercise_id: i64,
    /// 记录时间
    pub created_at: chrono::NaiveDateTime,
    /// 重量 (数值，配合Exercise里的unit使用)
    pub weight: f64,
    /// 实际做了几个
    pub reps: Option<i64>,
}

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    // 初始化数据库
    pub async fn new(app_dir: &str) -> Result<Self, sqlx::Error> {
        // 确保目录存在
        if !Path::new(app_dir).exists() {
            fs::create_dir_all(app_dir).expect("未能创建应用目录");
        }

        let db_path = format!(
            "sqlite:{}/fitness_lite.db?mode=rwc",
            app_dir
        );
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_path)
            .await?;

        let db = Database { pool };
        db.init_tables().await?;
        Ok(db)
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
    pub async fn create_routine(&self, name: &str, desc: &str) -> Result<i64, sqlx::Error> {
        let id = sqlx::query("INSERT INTO routines (name, description) VALUES (?, ?)")
            .bind(name)
            .bind(desc)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    /// 删除轮次
    pub async fn delete_routine(&self, routine_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM routines WHERE id = ?")
            .bind(routine_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取某个轮次下的所有动作
    pub async fn get_exercises(&self, routine_id: i64) -> Result<Vec<Exercise>, sqlx::Error> {
        sqlx::query_as::<_, Exercise>("SELECT * FROM exercises WHERE routine_id = ? ORDER BY id")
            .bind(routine_id)
            .fetch_all(&self.pool)
            .await
    }

    /// 添加动作
    pub async fn add_exercise(
        &self,
        routine_id: i64,
        name: &str,
        sets: i64,
        reps: &str,
        note: &str,
        unit: &str,
    ) -> Result<i64, sqlx::Error> {
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
        Ok(id)
    }

    /// 删除动作
    pub async fn delete_exercise(&self, exercise_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM exercises WHERE id = ?")
            .bind(exercise_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 记录一次数据
    pub async fn add_record(
        &self,
        exercise_id: i64,
        weight: f64,
        reps: Option<i64>,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query("INSERT INTO records (exercise_id, weight, reps) VALUES (?, ?, ?)")
            .bind(exercise_id)
            .bind(weight)
            .bind(reps)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
        Ok(id)
    }

    /// 删除一条记录
    pub async fn delete_record(&self, record_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM records WHERE id = ?")
            .bind(record_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 分页获取某个动作的记录
    pub async fn page_records(
        &self,
        exercise_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<Record>, sqlx::Error> {
        let offset = (page - 1) * page_size;
        sqlx::query_as::<_, Record>(
            "SELECT * FROM records WHERE exercise_id = ? ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?",
        )
        .bind(exercise_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
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
        let records = db.page_records(e_id, 1, 10).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].weight, 100.0);
        assert_eq!(records[0].reps, Some(5));
    }

    #[tokio::test]
    async fn test_cascade_delete() {
        let db = setup_test_db().await;

        // 1. 建立层级数据：轮次 -> 动作 -> 记录
        let r_id = db.create_routine("背部", "").await.unwrap();
        let e_id = db.add_exercise(r_id, "引体向上", 4, "力竭", "", "个").await.unwrap();
        db.add_record(e_id, 0.0, Some(10)).await.unwrap();
        db.add_record(e_id, 0.0, Some(12)).await.unwrap();

        // 确认记录存在
        let recs_before = db.page_records(e_id, 1, 10).await.unwrap();
        assert_eq!(recs_before.len(), 2);

        // 2. 删除最顶层的【轮次】
        db.delete_routine(r_id).await.unwrap();

        // 3. 验证级联删除是否生效
        // 查动作：应该为空
        let exercises = db.get_exercises(r_id).await.unwrap();
        assert!(exercises.is_empty(), "动作应该被级联删除");

        // 查记录：应该为空
        let recs_after = db.page_records(e_id, 1, 10).await.unwrap();
        assert!(recs_after.is_empty(), "记录应该被级联删除");
    }

    #[tokio::test]
    async fn test_pagination() {
        let db = setup_test_db().await;
        let r_id = db.create_routine("腿", "").await.unwrap();
        let e_id = db.add_exercise(r_id, "深蹲", 5, "5", "", "kg").await.unwrap();

        // 插入 15 条记录
        for i in 0..15 {
            db.add_record(e_id, 100.0 + i as f64, Some(5)).await.unwrap();
        }

        // 第 1 页，每页 10 条 -> 应该有 10 条
        let page1 = db.page_records(e_id, 1, 10).await.unwrap();
        assert_eq!(page1.len(), 10);

        // 第 2 页，每页 10 条 -> 应该剩余 5 条
        let page2 = db.page_records(e_id, 2, 10).await.unwrap();
        assert_eq!(page2.len(), 5);

        // 第 3 页 -> 0 条
        let page3 = db.page_records(e_id, 3, 10).await.unwrap();
        assert!(page3.is_empty());
    }

    #[tokio::test]
    async fn test_foreign_key_constraint() {
        let db = setup_test_db().await;

        // 创建轮次和动作
        let r_id = db.create_routine("肩部", "").await.unwrap();
        let e_id = db.add_exercise(r_id, "推举", 4, "8-12", "", "kg").await.unwrap();

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
