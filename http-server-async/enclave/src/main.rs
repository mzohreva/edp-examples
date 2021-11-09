use anyhow::Result;
use bytes::Buf;
use hyper::{rt, server::conn::Http, service::service_fn, Body, StatusCode};
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use std::future::Future;
use std::result::Result as StdResult;
use tokio::net::TcpListener;
use tokio_compat_02::IoCompat;

type Request = hyper::Request<Body>;
type Response = hyper::Response<Body>;

async fn digest_handler(req: Request) -> Result<Response> {
    let mut body = hyper::body::aggregate(req.into_body()).await?;
    let mut hasher = Sha256::new();
    while body.has_remaining() {
        let bytes = body.bytes();
        hasher.update(bytes);
        let n = bytes.len();
        body.advance(n);
    }
    let hash = hex::encode(hasher.finalize());
    Ok(Response::new(Body::from(hash)))
}

async fn handle_request(req: Request) -> StdResult<Response, Infallible> {
    match digest_handler(req).await {
        Ok(response) => Ok(response),
        Err(e) => {
            let mut response = Response::new(Body::from(e.to_string()));
            *response.status_mut() = StatusCode::BAD_REQUEST;
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Started listening on: {}", listener.local_addr()?);

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            let http = Http::new().with_executor(Executor);
            let res = http
                .serve_connection(IoCompat::new(socket), service_fn(handle_request))
                .await;

            match res {
                Err(e) => println!("Error handling request from client {}: {}", addr, e),
                Ok(()) => {}
            }
        });
    }
}

#[derive(Copy, Clone)]
pub struct Executor;

impl<F> rt::Executor<F> for Executor
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::spawn(fut);
    }
}
