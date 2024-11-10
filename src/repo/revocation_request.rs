use time::{PrimitiveDateTime, format_description::well_known::Rfc3339};

use super::{Repo, Error, QuickError};

#[derive(Clone, Debug)]
pub(crate) struct RevocationRequest {
	pub(crate) sent_at: String,
	pub(crate) sent_to: String,
	pub(crate) accepted: String,
}

impl Repo {
    pub(crate) async fn revocation_requests(&self) -> Result<Vec<RevocationRequest>, Error> {
   		let db = self.db().await?;
		let stmt = db.prepare_cached(r#"
			SELECT
				sent_at,
				email_address AS sent_to,
				successful
			FROM revocation_request_emails AS rre
			JOIN issuer_contact_emails AS ice
			  ON rre.issuer_contact_email_id=ice.id

			UNION ALL

			SELECT
				sent_at,
				url AS sent_to,
				successful
			FROM revocation_request_pwnedkey_p10s AS rrpp
			JOIN issuer_contact_pwnedkey_p10s AS icpp
			  ON rrpp.issuer_contact_pwnedkey_p10_id=icpp.id

			ORDER BY sent_at
		"#).await.server_error("Failed to prepare statement")?;
		let rows = db.query(&stmt, &[]).await.server_error("Failed to execute query")?;
		rows.into_iter().map(|row| {
			Ok::<RevocationRequest, Error>(RevocationRequest{
				sent_at: row.get::<_, PrimitiveDateTime>("sent_at").assume_utc().format(&Rfc3339).server_error("Failed to format sent_at")?,
				sent_to: row.get("sent_to"),
				accepted: if row.get("successful") {
					"Yes"
				} else {
					"No"
				}.to_string(),
			})
		}).collect()
    }
}
