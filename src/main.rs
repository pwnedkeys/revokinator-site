use actix_web::{dev::Service as _, middleware, web, App as ActixApp, HttpServer};
use futures_util::future::FutureExt as _;
use service_skeleton::{
	metric::{
		self,
		encode_labels::{prometheus_client, EncodeLabelSet},
	},
	service,
};
use std::time::Instant;

mod error;
use error::Error;

mod config;
use config::{Config, ListenAddress};

mod repo;
use repo::Repo;
mod rate_limiter;

mod index;
mod details;

mod page_components;
use page_components::PageComponents;

#[derive(Clone, Debug, Default, EncodeLabelSet, Eq, Hash, PartialEq)]
struct HttpRequest {
	endpoint: String,
	method: String,
}

impl HttpRequest {
	fn new(endpoint: impl Into<String>, method: impl Into<String>) -> Self {
		Self {
			endpoint: endpoint.into(),
			method: method.into(),
		}
	}
}

#[derive(Clone, Debug, Default, EncodeLabelSet, Eq, Hash, PartialEq)]
struct CompletedHttpRequest {
	endpoint: String,
	method: String,
	status: String,
}

impl CompletedHttpRequest {
	fn new(
		endpoint: impl Into<String>,
		method: impl Into<String>,
		status: impl Into<String>,
	) -> Self {
		Self {
			endpoint: endpoint.into(),
			method: method.into(),
			status: status.into(),
		}
	}
}

const HTTP_HISTOGRAM_BUCKETS: [f64; 13] = [
	0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0,
];

fn main() {
	let svc = service("RevokinatorSite")
		.counter::<HttpRequest>(
			"http_requests_received",
			"How many HTTP requests we've received",
		)
		.gauge::<HttpRequest>(
			"http_requests_in_progress",
			"How many HTTP requests we're currently processing",
		)
		.counter::<CompletedHttpRequest>(
			"http_requests_completed",
			"How many HTTP requests have been completed, one way or another",
		)
		.histogram::<CompletedHttpRequest>(
			"http_requests_duration",
			"How long each HTTP request took to complete",
			HTTP_HISTOGRAM_BUCKETS.as_ref(),
		);
	svc.run(|cfg: Config| run_site(cfg));
}

struct CssUrl(url::Url);

#[allow(clippy::unwrap_used, clippy::expect_used)] // This is allowed to shit itself
#[actix_web::main]
async fn run_site(cfg: Config) {
	let site_url = web::Data::new(cfg.base_url().clone());
	let global_rate_limiter = web::Data::new(rate_limiter::global());
	let address_rate_limiter = web::Data::new(rate_limiter::address());
	let repo = web::Data::new(repo::new(cfg.database_url()).expect("Repo creation failed"));
	let css_url = web::Data::new(CssUrl(cfg.css_url().clone()));

	let server = HttpServer::new(move || {
		ActixApp::new()
			.app_data(site_url.clone())
			.app_data(global_rate_limiter.clone())
			.app_data(address_rate_limiter.clone())
			.app_data(repo.clone())
			.app_data(css_url.clone())
			.wrap_fn(|req, srv| {
				let start_time = Instant::now();
				let (endpoint, method) = (
					req.match_name().unwrap_or("None").to_string(),
					req.method().to_string(),
				);

				srv.call(req).map(move |res| {
					let labels = HttpRequest::new(&endpoint, &method);
					metric::counter("http_requests_received", &labels, |m| {
						m.inc();
					});
					metric::gauge("http_requests_in_progress", &labels, |m| {
						m.inc();
					});
					let res_labels = CompletedHttpRequest::new(
						endpoint,
						method,
						res.as_ref()
							.map(|r| r.status().as_u16().to_string())
							.unwrap_or("???".to_string()),
					);
					metric::gauge("http_requests_in_progress", &labels, |m| {
						m.dec();
					});
					metric::histogram("http_requests_duration", &res_labels, |m| {
						m.observe(Instant::now().duration_since(start_time).as_secs_f64());
					});
					metric::counter("http_requests_completed", &res_labels, |m| {
						m.inc();
					});
					res
				})
			})
			.wrap(middleware::NormalizePath::trim())
			.wrap_fn(|req, srv| {
				tracing::debug!("{:?}", req);
				srv.call(req).map(move |res| {
					tracing::debug!("{:?}", res);
					res
				})
			})
			.wrap(tracing_actix_web::TracingLogger::default())
			.configure(index::routes)
			.configure(details::routes)
	})
	.disable_signals();

	let server = match cfg.listen_address() {
		ListenAddress::Unix(p) => {
			match std::fs::remove_file(&p) {
				Ok(()) => tracing::debug!("Removed stale socket {p}", p = p.display()),
				Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
					tracing::debug!("No stale socket found");
				}
				Err(e) => tracing::warn!(
					"Failed to remove listening socket {p}: {e}",
					p = p.display()
				),
			};
			file_mode::set_umask(0o117);
			server.bind_uds(p).unwrap()
		}
		ListenAddress::Tcp(a) => server.bind(a).unwrap(),
	};

	let _unused = server.run().await;
}
