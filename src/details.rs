use actix_web::{web, HttpResponse, middleware::from_fn};
use askama_actix::{Template, TemplateToResponse};

use super::{Error, PageComponents, repo::{Repo, RevocationCandidate, RevocationRequest}, rate_limiter};

pub(super) fn routes(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/revokinator/details").wrap(from_fn(rate_limiter::check)).get(index).name("revokinator-details"));
	cfg.service(web::resource("/revokinator/details/candidates").wrap(from_fn(rate_limiter::check)).get(candidates).name("revokinator-candidates"));
	cfg.service(web::resource("/revokinator/details/requests").wrap(from_fn(rate_limiter::check)).get(requests).name("revokinator-requests"));
}

#[derive(Clone, Debug, Template)]
#[template(path = "details/index.html")]
struct Index {
	components: PageComponents,
	request_count: u64,
	candidate_count: u64,
}

#[tracing::instrument]
async fn index(components: PageComponents, repo: web::Data<Repo>) -> Result<HttpResponse, Error> {
	Ok(Index {
		components,
		request_count: repo.revocation_request_count().await?,
		candidate_count: repo.revocation_candidate_count().await?,
	}
	.to_response())
}

#[derive(Clone, Debug, Template)]
#[template(path = "details/candidates.html")]
struct Candidates {
	components: PageComponents,
	candidates: Vec<RevocationCandidate>,
}

#[tracing::instrument]
async fn candidates(components: PageComponents, repo: web::Data<Repo>) -> Result<HttpResponse, Error> {
	Ok(Candidates {
		components,
		candidates: repo.revocation_candidates().await?,
	}
	.to_response())
}

#[derive(Clone, Debug, Template)]
#[template(path = "details/requests.html")]
struct Requests {
	components: PageComponents,
	requests: Vec<RevocationRequest>,
}

#[tracing::instrument]
async fn requests(components: PageComponents, repo: web::Data<Repo>) -> Result<HttpResponse, Error> {
	Ok(Requests {
		components,
		requests: repo.revocation_requests().await?,
	}
	.to_response())
}
