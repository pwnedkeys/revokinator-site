use time::{PrimitiveDateTime, format_description::well_known::{Rfc2822, Rfc3339}};
//use uuid::Uuid;

use super::{Repo, Error, QuickError};

#[derive(Clone, Debug)]
pub(crate) struct RevocationCandidate {
	pub(crate) subject: String,
	pub(crate) issuer: String,
	pub(crate) expiry: String,
	pub(crate) notified_at: String,
//	pub(crate) ocsp_checks_link: String,
	pub(crate) cert_display_link: String,
}

impl Repo {
	pub(crate) async fn revocation_candidates(&self) -> Result<Vec<RevocationCandidate>, Error> {
		let db = self.db().await?;
		let stmt = db.prepare_cached(r#"
			SELECT id,
			       COALESCE(NULLIF(x509_commonName(certificate_der), ''), (SELECT x509_altNames(certificate_der) LIMIT 1), '???') AS subject,
			       x509_issuerName(certificate_der) AS issuer,
			       x509_notAfter(certificate_der) AS expiry,
			       rs.notified_at,
			       encode(digest(certificate_der, 'sha256'), 'hex') AS hash
			  FROM revocation_candidates AS rc
			  JOIN revocation_status AS rs
			    ON rc.id=rs.candidate_id
			 WHERE rc.not_after > NOW()
		"#).await.server_error("Failed to prepare statement")?;
		let rows = db.query(&stmt, &[]).await.server_error("Failed to execute query")?;
		rows.into_iter().map(|row| {
			let maybe_notified_at: Option<PrimitiveDateTime> = row.get("notified_at");
//			let id = row.get::<_, Uuid>("id").to_string();
			let hash: String = row.get("hash");

			let expiry = row.get::<_, PrimitiveDateTime>("expiry").assume_utc().format(&Rfc3339).server_error("Failed to format expiry")?;

			let notified_at = maybe_notified_at.map_or(
				Ok("Pending".to_string()),
				|t| t.assume_utc().format(&Rfc2822).server_error("Failed to format notified_at")
			)?;
//			let ocsp_checks_link = maybe_notified_at.map_or(
//				"&nbsp;".to_string(),
//				|_| format!(r#"<a href="ocsp_history/{id}">OCSP Checks</a>"#)
//			);
			let cert_display_link = maybe_notified_at.map_or(
				"&nbsp;".to_string(),
				|_| format!(r#"<a target="_blank" href="https://crt.sh/?sha256={hash}">View full certificate</a>"#)
			);

			Ok::<RevocationCandidate, Error>(RevocationCandidate{
				subject: row.get("subject"),
				issuer: row.get("issuer"),
				expiry,
				notified_at,
//				ocsp_checks_link,
				cert_display_link,
			})
		}).collect()
	}
}
