//! An HTTP server based on `hyper`.
//!
//! Run with:
//!
//! ```
//! cargo run --bin hello-smol --release
//! ```
//!
//! Open in the browser any of these addresses:
//!
//! - http://localhost:8000/

use std::io;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::{Error, Result};
use futures::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use smol::{Async, Task};

/// Serves a request and returns a response.
async fn serve(_req: Request<Body>) -> Result<Response<Body>> {
    Ok(Response::new(Body::from("Hello world!")))
}

/// Listens for incoming connections and serves them.
async fn listen(listener: Async<TcpListener>) -> Result<()> {
    // Start a hyper server.
    Server::builder(SmolListener::new(listener))
        .executor(SmolExecutor)
        .serve(make_service_fn(move |_| async {
            Ok::<_, Error>(service_fn(serve))
        }))
        .await?;

    Ok(())
}

fn main() -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

    println!("Listening on http://{}", addr);

    // Start HTTP server.
    smol::run(listen(Async::<TcpListener>::bind(&addr)?))
}

/// Spawns futures.
#[derive(Clone)]
struct SmolExecutor;

impl<F: Future + Send + 'static> hyper::rt::Executor<F> for SmolExecutor {
    fn execute(&self, fut: F) {
        Task::spawn(async { drop(fut.await) }).detach();
    }
}

/// Listens for incoming connections.
struct SmolListener {
    listener: Async<TcpListener>,
}

impl SmolListener {
    fn new(listener: Async<TcpListener>) -> Self {
        Self { listener }
    }
}

impl hyper::server::accept::Accept for SmolListener {
    type Conn = SmolStream;
    type Error = Error;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let poll = Pin::new(&mut self.listener.incoming()).poll_next(cx);
        let stream = futures::ready!(poll).unwrap()?;
        Poll::Ready(Some(Ok(SmolStream(stream))))
    }
}

/// A TCP connection.
struct SmolStream(Async<TcpStream>);

impl hyper::client::connect::Connection for SmolStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}

impl tokio::io::AsyncRead for SmolStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for SmolStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.0.get_ref().shutdown(Shutdown::Write)?;
        Poll::Ready(Ok(()))
    }
}
