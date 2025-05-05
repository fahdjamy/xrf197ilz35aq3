use crate::CassandraDBError;
use cassandra_cpp::{BindRustType, Consistency, PreparedStatement, RetryPolicy, Session};

pub async fn save_block_chain(session: Session) -> Result<bool, CassandraDBError> {
    // The order of columns in the INSERT statement (app_id, sequence_num, id, ...)
    // must match the order of the placeholders (?, ?, ?, ...)
    let insert_cql = "INSERT INTO xrf_q3_block.block_chain \
    (app_id, sequence_num, id, chain_id, region, version, entry_ids, creation_date) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
    let prepared_insert_stmt = session
        .prepare(insert_cql)
        .await
        .map_err(|err| CassandraDBError::Unknown(err.to_string()))?;

    let key: i64 = 10;
    let mut statement = prepared_insert_stmt.bind(); // Create a bound statement
    statement
        .set_consistency(Consistency::EACH_QUORUM)
        .map_err(|err| CassandraDBError::ServerError(err.to_string()))?;
    statement
        .set_retry_policy(RetryPolicy::downgrading_consistency_new())
        .map_err(|err| CassandraDBError::ServerError(err.to_string()))?;
    statement
        .bind(0, key)
        .map_err(|err| CassandraDBError::Unknown(err.to_string()))?;

    let result = session
        .execute_with_payloads(&statement)
        .await
        .map_err(|err| CassandraDBError::ExecutionError(err.to_string()))?;

    Ok(result.0.row_count() == 1)
}

async fn prepare(session: &Session) -> cassandra_cpp::Result<PreparedStatement> {
    // The order of columns in the INSERT statement (app_id, sequence_num, id, ...)
    // must match the order of the placeholders (?, ?, ?, ...)
    let insert_cql = "INSERT INTO xrf_q3_block.block_chain \
    (app_id, sequence_num, id, chain_id, region, version, entry_ids, creation_date) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

    session.prepare(insert_cql).await
}
