use crate::storage::prepare_insert_block_statement;
use crate::CassandraDBError;
use cassandra_cpp::{PreparedStatement, Session};

#[derive(Debug)]
pub struct PreparedAppStatements {
    pub insert_block_stmt: PreparedStatement,
}

impl PreparedAppStatements {
    pub async fn new(session: &Session) -> Result<Self, CassandraDBError> {
        let insert_block_stmt = prepare_insert_block_statement(session).await?;

        Ok(PreparedAppStatements { insert_block_stmt })
    }
}
