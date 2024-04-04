pub struct Database {
    db_pool: sqlx::mysql::MySqlPool,
}

impl Database {
    pub fn new(database_url: &str) -> Self {
        let db_pool = sqlx::mysql::MySqlPool::connect_lazy(database_url).unwrap();
        Database { db_pool }
    }

    #[allow(dead_code)]
    pub async fn insert_stage_task(
        &self,
        proof_id: &str,
        status: i32,
        context: &str,
    ) -> anyhow::Result<bool> {
        sqlx::query("INSERT INTO stage_task (id, status, context) values (?,?,?)")
            .bind(proof_id)
            .bind(status)
            .bind(context)
            .execute(&self.db_pool)
            .await?;
        Ok(true)
    }

    #[allow(dead_code)]
    pub async fn update_stage_task(
        &mut self,
        proof_id: &str,
        status: i32,
        result: &str,
    ) -> anyhow::Result<bool> {
        sqlx::query("UPDATE stage_task set status = ?, result = ? where id = ?)")
            .bind(status)
            .bind(result)
            .bind(proof_id)
            .execute(&self.db_pool)
            .await?;
        Ok(true)
    }
}
