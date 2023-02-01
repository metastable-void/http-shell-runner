
use std::process::Command;
use std::env;
use dotenvy::dotenv;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use hyper::server::conn::AddrStream;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::StatusCode;

async fn handle_request(req: Request<Body>, _addr: SocketAddr) -> StatusCode {
    let path = req.uri().path().trim_start_matches('/');

    let secret_path;
    if let Ok(path) = env::var("SECRET_PATH") {
        secret_path = path;
    } else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if secret_path == "" {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let command;
    if let Ok(cmd) = env::var("COMMAND") {
        command = cmd;
    } else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if command == "" {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if path != &secret_path {
        return StatusCode::NOT_FOUND;
    }

    let result = Command::new(command).status();
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn get(req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    let status_code = handle_request(req, addr).await;
    Ok(
        Response::builder()
            .status(status_code)
            .body(Body::empty())
            .unwrap()
    )
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // ignore if .env file is not present

    let addr_string = env::var("LISTEN_ADDR").unwrap_or("".to_string());
    let addr = SocketAddr::from_str(&addr_string).unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080)));

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let addr = conn.remote_addr();
        async move {
            let addr = addr.clone();
            Ok::<_, Infallible>(service_fn(move |req : Request<Body>| {
                get(req, addr)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
