FROM rust:alpine as rust-build

RUN --mount=type=cache,target=/var/cache/apt,id=revokinator-site-rust-build-apt-cache \
	--mount=type=cache,target=/var/lib/apt,id=revokinator-site-rust-build-apt-lib \
	rustup target add x86_64-unknown-linux-musl \
	&& apk add build-base \
	&& mkdir /build

COPY askama.toml Cargo.* /build
COPY layouts /build/layouts
COPY templates /build/templates
COPY src /build/src

WORKDIR /build

RUN --mount=type=cache,target=/build/target,id=revokinator-site-rust-build-target \
	--mount=type=cache,target=/usr/local/cargo/registry,id=revokinator-site-rust-build-registry \
	cargo build --target x86_64-unknown-linux-musl --release \
	&& cp target/x86_64-unknown-linux-musl/release/revokinator-site /build/revokinator-site

FROM scratch

COPY --from=rust-build /build/revokinator-site /revokinator-site

ENTRYPOINT ["/revokinator-site"]
