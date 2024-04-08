use serde_derive::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, Default, sqlx::FromRow)]
pub struct StageTask {
    pub id: String,
    pub status: i32,
    pub context: Option<String>,
    pub result: Option<String>,
    pub check_at: i64,
}

pub struct Database {
    db_pool: sqlx::mysql::MySqlPool,
}

impl Database {
    pub fn new(database_url: &str) -> Self {
        let db_pool = sqlx::mysql::MySqlPool::connect_lazy(database_url).unwrap();
        Database { db_pool }
    }

    #[allow(dead_code)]
    pub async fn get_stage_task(&self, proof_id: &str) -> anyhow::Result<StageTask> {
        let row = sqlx::query_as!(
            StageTask,
            "SELECT id, status, context, result, check_at from stage_task where id = ?",
            proof_id,
        )
        .fetch_one(&self.db_pool)
        .await?;
        Ok(row)
    }

    #[allow(dead_code)]
    pub async fn insert_stage_task(
        &self,
        proof_id: &str,
        status: i32,
        context: &str,
    ) -> anyhow::Result<bool> {
        sqlx::query!(
            "INSERT INTO stage_task (id, status, context) values (?,?,?)",
            proof_id,
            status,
            context
        )
        .execute(&self.db_pool)
        .await?;
        Ok(true)
    }

    #[allow(dead_code)]
    pub async fn update_stage_task(
        &self,
        proof_id: &str,
        status: i32,
        result: &str,
    ) -> anyhow::Result<bool> {
        sqlx::query!(
            "UPDATE stage_task set status = ?, result = ? where id = ?",
            status,
            result,
            proof_id
        )
        .execute(&self.db_pool)
        .await?;
        Ok(true)
    }
}