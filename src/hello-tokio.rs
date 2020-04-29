//! An HTTP server based on `hyper`.
//!
//! Run with:
//!
//! ```
//! cargo run --bin hello-tolio --release
//! ```
//!
//! Open in the browser any of these addresses:
//!
//! - http://localhost:8001/

#![deny(warnings)]

use anyhow::{Error, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

async fn serve(_req: Request<Body>) -> Result<Response<Body>> {
    Ok(Response::new(Body::from("Hello world!")))
}

//#[tokio::main(threaded_scheduler)]
#[tokio::main(core_threads = 8)]
pub async fn main() -> Result<()> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Error>(service_fn(serve)) }
    });

    let addr = ([127, 0, 0, 1], 8001).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
