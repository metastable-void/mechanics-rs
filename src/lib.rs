use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response};
use hyper_util::rt::TokioIo;
use mechanics_core::job::MechanicsJob;
use tokio::net::TcpListener;

use mechanics_core::{MechanicsPool, MechanicsPoolConfig};

fn json_response(v: &serde_json::Value, code: Option<u16>) -> Response<Full<Bytes>> {
    Response::builder()
        .header("content-type", "application/json")
        .status(code.unwrap_or(200))
        .body(Full::new(
            Bytes::from(
                serde_json::to_string(v)
                    .unwrap_or("{}".to_string())))).unwrap()
}

#[derive(Clone)]
pub struct MechanicsServer {
    pool: Arc<MechanicsPool>,
}

impl MechanicsServer {
    pub fn new(config: MechanicsPoolConfig) -> std::io::Result<Self> {
        let pool = Arc::new(MechanicsPool::new(config).map_err(std::io::Error::other)?);
        let this = Self {
            pool,
        };
        Ok(this)
    }

    pub fn pool(&self) -> Arc<MechanicsPool> {
        self.pool.clone()
    }

    pub fn run(&self, bind_addr: SocketAddr) -> std::io::Result<()> {
        let std_listener = std::net::TcpListener::bind(bind_addr)?;
        std_listener.set_nonblocking(true)?;
        let this = self.clone();
        let _ = std::thread::Builder::new().name("MechanicsServer".to_string()).spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all().build()?;

            rt.block_on(async move {
                let listener = TcpListener::from_std(std_listener)?;
                loop {
                    let (stream, _) = listener.accept().await?;

                    let io = TokioIo::new(stream);

                    let srv = this.clone();
                    tokio::task::spawn(async move {
                        let srv = srv.clone();
                        if let Err(err) = http1::Builder::new()
                            .serve_connection(io, service_fn(|req: Request<Incoming>| {
                                let srv = srv.clone();
                                async move {
                                    let this = srv.clone();
                                    {
                                        let method = req.method();
                                        let path = req.uri().path();
                                        match (method, path) {
                                            (&Method::POST, "/api/v1/mechanics") => {},
                                            _ => {
                                                return Ok(json_response(&serde_json::json!({
                                                    "error": "Not found",
                                                }), Some(404)));
                                            },
                                        }
                                    }
                                    {
                                        let ct = req.headers().get("content-type")
                                            .map(|v| v.to_str().ok()).flatten();

                                        if !matches!(ct, Some("application/json")) {
                                            return Ok(json_response(&serde_json::json!({
                                                "error": "Invalid type",
                                            }), Some(400)));
                                        }
                                    }

                                    let body = req.into_body().collect().await.map(|b| b.to_bytes());
                                    
                                    if let Err(_) = body {
                                        return Ok(json_response(&serde_json::json!({
                                            "error": "Invalid request",
                                        }), Some(400)));
                                    }

                                    let body = body.unwrap();
                                    let job: Result<MechanicsJob, _> = serde_json::from_slice(&body);
                                        
                                    if let Err(_) = job {
                                        return Ok(json_response(&serde_json::json!({
                                            "error": "Invalid request",
                                        }), Some(400)));
                                    }

                                    let job = job.unwrap();

                                    let res = tokio::task::spawn_blocking(move || {
                                        this.clone().pool().run(job)
                                    }).await.unwrap();    
                                    if let Err(e) = res {
                                        return Ok(json_response(&serde_json::json!({
                                            "error": format!("{}", e),
                                        }), Some(400)));
                                    }

                                    let res = res.unwrap();
                                    
                                    Ok::<_, Infallible>(json_response(&res, None))
                                }
                            }))
                            .await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
                    });
                }

                #[allow(unused)]
                Ok::<_, std::io::Error>(())
            })
        })?;
        Ok(())
    }
}
