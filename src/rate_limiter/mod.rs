use actix_web::{
    Error as ActixError,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
web,
};
use actix_web::middleware::Next;

use nonzero_ext::nonzero;
use std::net::{IpAddr, Ipv4Addr};

use crate::Error;

pub(crate) type GlobalRateLimiter = governor::DefaultDirectRateLimiter;
pub(crate) type AddressRateLimiter = governor::DefaultKeyedRateLimiter<IpAddr>;

pub(crate) fn global() -> GlobalRateLimiter {
	governor::RateLimiter::direct(governor::Quota::per_second(nonzero!(5u32)))
}

pub(crate) fn address() -> AddressRateLimiter {
	governor::RateLimiter::keyed(governor::Quota::per_minute(nonzero!(15u32)))
}

const UNKNOWN_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));

pub(crate) async fn check(
	g: web::Data<GlobalRateLimiter>,
	a: web::Data<AddressRateLimiter>,
	req: ServiceRequest,
	next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, ActixError> {
	g.check().map_err(|_| Error::rate_limited())?;
	a.check_key(&req.request().peer_addr().map_or(UNKNOWN_ADDRESS, |p| p.ip())).map_err(|_| Error::rate_limited())?;

	next.call(req).await
}
