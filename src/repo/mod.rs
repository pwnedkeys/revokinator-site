use deadpool_postgres::{
	        Config as DbConfig, Object as PoolObject, Pool as DeadPool,
};
use tokio_postgres::NoTls;

use crate::{error::QuickError, Error};

mod revocation_candidate;
pub(crate) use revocation_candidate::RevocationCandidate;
mod revocation_request;
pub(crate) use revocation_request::RevocationRequest;

#[derive(Debug, Clone)]
pub(crate) struct Repo {
	pool: DeadPool,
}

pub(crate) fn new(url: &url::Url) -> Result<Repo, Error> {
	Ok(Repo { pool: DbConfig { url: Some(url.to_string()), ..DbConfig::default() }.create_pool(None, NoTls).server_error("Failed to create database pool")? })
}

impl Repo {
	async fn db(&self) -> Result<PoolObject, Error> {
		Ok(self.pool.get().await.server_error("Failed to get DB connection")?)
	}

	pub(crate) async fn revocation_request_count(&self) -> Result<u64, Error> {
		let db = self.db().await?;
		let stmt = db.prepare_cached("SELECT COUNT(DISTINCT rr.id) FROM revocation_requests AS rr JOIN revocation_candidates AS rc ON rr.id=rc.revocation_request_id WHERE rc.not_after > NOW()").await.server_error("Failed to prepare statement")?;
		let rows = db.query(&stmt, &[]).await.server_error("Failed to execute query")?;
		Ok(rows[0].get::<_, i64>(0).try_into().server_error("Failed to convert count to u64")?)
	}

	pub(crate) async fn revocation_candidate_count(&self) -> Result<u64, Error> {
		let db = self.db().await?;
		let stmt = db.prepare_cached("SELECT COUNT(id) FROM revocation_candidates WHERE not_after > NOW()").await.server_error("Failed to prepare statement")?;
		let rows = db.query(&stmt, &[]).await.server_error("Failed to execute query")?;
		Ok(rows[0].get::<_, i64>(0).try_into().server_error("Failed to convert count to u64")?)
	}
}
