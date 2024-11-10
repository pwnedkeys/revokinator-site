use actix_web::{HttpResponse, ResponseError};
use std::{convert::AsRef, error::Error as StdError};

#[derive(Debug)]
pub(crate) enum Error {
	Server {
		msg: String,
		cause: Option<Box<dyn StdError>>,
	},
	User {
		msg: String,
		cause: Option<Box<dyn StdError>>,
	},
	RateLimited,
}

impl Error {
	pub(crate) fn user(msg: impl Into<String>) -> Self {
		Self::User {
			msg: msg.into(),
			cause: None,
		}
	}

	pub(crate) fn user_error(msg: impl Into<String>, error: impl StdError + 'static) -> Self {
		Self::User {
			msg: msg.into(),
			cause: Some(Box::new(error)),
		}
	}

	pub(crate) fn server(msg: impl Into<String>) -> Self {
		Self::Server {
			msg: msg.into(),
			cause: None,
		}
	}

	pub(crate) fn server_error(msg: impl Into<String>, error: impl StdError + 'static) -> Self {
		Self::Server {
			msg: msg.into(),
			cause: Some(Box::new(error)),
		}
	}

	pub(crate) fn rate_limited() -> Self {
		Self::RateLimited
	}
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Server { cause, .. } | Self::User { cause, .. } => {
				cause.as_ref().map(AsRef::as_ref)
			},
			Self::RateLimited => None,
		}
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.write_str(match self {
			Self::Server { .. } => "server-side error occurred",
			Self::User { msg, .. } => msg,
			Self::RateLimited => "We're a bit overloaded at the moment.  Try again later.",
		})
	}
}

pub(crate) trait QuickError<T, E> {
	fn server_error(self, s: impl Into<String>) -> Result<T, Error>;
	fn user_error(self, s: impl Into<String>) -> Result<T, Error>;
}

impl<T, E: StdError + 'static> QuickError<T, E> for Result<T, E> {
	fn server_error(self, s: impl Into<String>) -> Result<T, Error> {
		self.map_err(|e| Error::server_error(s.into(), e))
	}

	fn user_error(self, s: impl Into<String>) -> Result<T, Error> {
		self.map_err(|e| Error::user_error(s.into(), e))
	}
}

impl<T> QuickError<T, ()> for Option<T> {
	fn server_error(self, s: impl Into<String>) -> Result<T, Error> {
		self.ok_or_else(|| Error::server(s.into()))
	}

	fn user_error(self, s: impl Into<String>) -> Result<T, Error> {
		self.ok_or_else(|| Error::user(s.into()))
	}
}

impl Drop for Error {
	fn drop(&mut self) {
		match self {
			Self::Server { msg, cause } => tracing::error!("Server error: {msg} ({cause:?})"),
			Self::User { msg, cause } => tracing::debug!("User error: {msg} ({cause:?})"),
			Self::RateLimited => (),
		};
	}
}

impl ResponseError for Error {
	fn error_response(&self) -> HttpResponse {
		match self {
			Self::Server { .. } => HttpResponse::InternalServerError().finish(),
			Self::User { msg, .. } => HttpResponse::BadRequest()
				.content_type("text/plain")
				.body(msg.clone()),
			Self::RateLimited => HttpResponse::TooManyRequests().content_type("text/plain").body("We're a bit overloaded at the moment.  Try again later."),

		}
	}
}
