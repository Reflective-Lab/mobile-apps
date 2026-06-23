//! Smoke test: fire one event to the Rust-core Sentry project and flush, so we
//! can confirm the rust-client observability path end-to-end.
//!
//!   cargo run -p quorum-ffi --example sentry_smoke
//!
//! The DSN is the same public client key the apps pass to `init_observability`
//! (Sentry project 4511614643142736). `debug = true` logs the HTTP send.
use std::time::Duration;

fn main() {
    let _guard = sentry::init((
        "https://096bf7f5a5e69d38023975659d020217@o4511614588223488.ingest.de.sentry.io/4511614643142736",
        sentry::ClientOptions {
            release: Some("quorum-smoke@0.1.0".into()),
            environment: Some("debug".into()),
            debug: true,
            ..Default::default()
        },
    ));

    sentry::capture_message(
        "rust-client Sentry smoke event (cargo example)",
        sentry::Level::Info,
    );

    // Statics aren't dropped at exit, so flush explicitly to guarantee delivery.
    if let Some(client) = sentry::Hub::current().client() {
        client.flush(Some(Duration::from_secs(10)));
    }
    println!("rust-client smoke event captured + flushed");
}
