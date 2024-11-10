use actix_web::{dev::Payload, FromRequest, HttpRequest, web};
use std::{convert::Infallible, future::Future, pin::Pin};
use url::Url;

/// These are various bits and pieces that need to be available on all pages, typically because
/// they're variables used in the layout.
///
/// They get bundled into a single struct because passing around lots of arguments into every page
/// gets very tedious, *very* quickly.
#[derive(Clone, Debug)]
pub(crate) struct PageComponents {
	pub(crate) css_url: Url,
}

impl FromRequest for PageComponents {
	type Error = Infallible;
	type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

	fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
		let req = req.clone();

		let css_url = req.app_data::<web::Data<super::CssUrl>>().expect("Failed to obtain CssUrl from request app data").0.clone();

		Box::pin(async move {
			Ok(Self { css_url })
		})
	}
}
