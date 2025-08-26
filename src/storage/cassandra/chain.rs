use crate::core::Block;
use crate::CassandraDBError;
use cassandra_cpp::{
    BindRustType, CassCollection, Consistency, PreparedStatement, RetryPolicy, Session,
};

pub async fn save_block_chain(
    block: &Block,
    session: &Session,
    prepared_insert_stmt: &PreparedStatement,
) -> Result<bool, CassandraDBError> {
    let mut statement = prepared_insert_stmt.bind(); // Create a bound statement
    statement
        .set_consistency(Consistency::EACH_QUORUM)
        .map_err(|err| CassandraDBError::ServerError(err.to_string()))?;
    statement
        .set_retry_policy(RetryPolicy::downgrading_consistency_new())
        .map_err(|err| CassandraDBError::ServerError(err.to_string()))?;

    ////// set values
    statement
        .bind(0, block.app_id.as_str())
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    statement
        .bind(1, block.sequence_num as i64)
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    statement
        .bind(2, block.id.as_str())
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    statement
        .bind(3, block.chain_id.as_str())
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    statement
        .bind(4, block.region.to_string().as_str())
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    statement
        .bind(5, block.version.to_string().as_str())
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;

    let mut entry_ids_col = cassandra_cpp::List::new();

    for entry_id in block.entry_ids.clone() {
        entry_ids_col
            .append_string(&entry_id)
            .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    }
    statement
        .bind(6, entry_ids_col)
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;
    statement
        .bind(7, block.creation_date.timestamp_millis())
        .map_err(|err| CassandraDBError::SetValueError(err.to_string()))?;

    ////// execute the prepared statement and save the data to DB
    session
        .execute_with_payloads(&statement)
        .await
        .map_err(|err| CassandraDBError::ExecutionError(err.to_string()))?;

    Ok(true)
}

pub async fn prepare_insert_block_statement(
    session: &Session,
) -> Result<PreparedStatement, CassandraDBError> {
    // The order of columns in the INSERT statement (app_id, sequence_num, id, ...)
    // must match the order of the placeholders (?, ?, ?, ...)
    let insert_cql = "INSERT INTO xrf_q3_block.block_chain \
    (app_id, sequence_num, id, chain_id, region, version, entry_ids, creation_date) \
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)";

    let prepared_stmt = session
        .prepare(insert_cql)
        .await
        .map_err(|err| CassandraDBError::ServerError(err.to_string()))?;
    Ok(prepared_stmt)
}
