use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::fs;

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let body = fs::read_to_string("server/index.html").await.unwrap_or_else(|_| "Missing index.html".into());
    Ok(Response::new(Body::from(body)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(serve_req)) });
    let server = Server::bind(&addr).serve(make_svc);
    let ip = local_ip_address::local_ip()?;
    println!("Server running at http://{}:8080 (map to {}.ony)", ip, ip);
    server.await?;
    Ok(())
}
