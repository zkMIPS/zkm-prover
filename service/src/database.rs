// use mysql_async::prelude::*;

pub struct Database {
    // db_pool: mysql_async::Pool,
}

impl Database {
    pub fn new(_database_url: &str) -> Self {
        // let db_pool = mysql_async::Pool::new(database_url);
        Database {}
    }

    // #[allow(dead_code)]
    // pub async fn insert_stage_task(
    //     &self,
    //     proof_id: &str,
    //     status: i32,
    //     context: &str,
    // ) -> std::result::Result<bool, String> {
    //     let mut conn = self.db_pool.get_conn().await.map_err(|e| (e.to_string()))?;
    //     let stmt = conn
    //         .prep("INSERT INTO stage_task (id, status, context) values (:id, :status, :context)")
    //         .await
    //         .map_err(|e| (e.to_string()))?;
    //     let params = mysql_async::Params::from(vec![
    //         (
    //             String::from("id"),
    //             mysql_async::Value::Bytes(proof_id.as_bytes().to_vec()),
    //         ),
    //         (
    //             String::from("status"),
    //             mysql_async::Value::Int(status as i64),
    //         ),
    //         (
    //             String::from("context"),
    //             mysql_async::Value::Bytes(context.as_bytes().to_vec()),
    //         ),
    //     ]);
    //     let _: std::result::Result<Vec<String>, String> =
    //         conn.exec(stmt, params).await.map_err(|e| (e.to_string()));
    //     Ok(true)
    // }

    // #[allow(dead_code)]
    // pub async fn update_stage_task(
    //     &mut self,
    //     proof_id: &str,
    //     status: i32,
    //     result: &str,
    // ) -> std::result::Result<bool, String> {
    //     let mut conn = self.db_pool.get_conn().await.map_err(|e| (e.to_string()))?;
    //     let stmt = conn
    //         .prep("UPDATE stage_task set status = :status, result = :result where id = :id)")
    //         .await
    //         .map_err(|e| (e.to_string()))?;
    //     let params = mysql_async::Params::from(vec![
    //         (
    //             String::from("id"),
    //             mysql_async::Value::Bytes(proof_id.as_bytes().to_vec()),
    //         ),
    //         (
    //             String::from("status"),
    //             mysql_async::Value::Int(status as i64),
    //         ),
    //         (
    //             String::from("result"),
    //             mysql_async::Value::Bytes(result.as_bytes().to_vec()),
    //         ),
    //     ]);
    //     let _: std::result::Result<Vec<String>, String> =
    //         conn.exec(stmt, params).await.map_err(|e| (e.to_string()));
    //     Ok(true)
    // }
}
