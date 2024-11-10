use service_skeleton::ServiceConfig;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Debug, ServiceConfig)]
pub(crate) struct Config {
	listen_address: ListenAddress,
	database_url: Url,
	base_url: Url,
	css_url: Url,
}

impl Config {
	pub(crate) fn listen_address(&self) -> &ListenAddress {
		&self.listen_address
	}

	pub(crate) fn database_url(&self) -> &Url {
		&self.database_url
	}

	pub(crate) fn base_url(&self) -> &Url {
		&self.base_url
	}

	pub(crate) fn css_url(&self) -> &Url {
		&self.css_url
	}
}

#[derive(Clone, Debug)]
pub(super) enum ListenAddress {
	Tcp(std::net::SocketAddr),
	Unix(PathBuf),
}

impl std::str::FromStr for ListenAddress {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s.strip_prefix("unix:") {
			Some(p) => Self::Unix(PathBuf::from(p)),
			None => Self::Tcp(
				s.parse()
					.map_err(|e| format!("invalid listen address: {e}"))?,
			),
		})
	}
}
