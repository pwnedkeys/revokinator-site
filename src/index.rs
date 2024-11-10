use actix_web::{web, HttpResponse, middleware::from_fn};
use askama_actix::{Template, TemplateToResponse};

use super::{Error, PageComponents, Repo, rate_limiter};

pub(super) fn routes(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/revokinator").wrap(from_fn(rate_limiter::check)).get(index).name("revokinator-index"));
	cfg.service(web::resource("/revokinator/faq").wrap(from_fn(rate_limiter::check)).get(faq).name("revokinator-faq"));
}

#[derive(Clone, Debug, Template)]
#[template(path = "index.html")]
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
#[template(path = "faq.html")]
struct Faq {
	components: PageComponents,
}

#[tracing::instrument]
async fn faq(components: PageComponents) -> Result<HttpResponse, Error> {
	Ok(Faq {
		components,
	}
	.to_response())
}
