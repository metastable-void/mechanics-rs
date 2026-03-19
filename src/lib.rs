use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::header::CONTENT_TYPE;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use mechanics_core::job::MechanicsJob;
use tokio::net::TcpListener;

use mechanics_core::{MechanicsPool, MechanicsPoolConfig};

type HttpResponse = Response<Full<Bytes>>;

enum ApiError {
    NotFound,
    InvalidType,
    InvalidRequest,
    Pool(String),
    Internal,
}

impl ApiError {
    fn to_response(&self) -> HttpResponse {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Self::InvalidType => (StatusCode::BAD_REQUEST, "Invalid type".to_string()),
            Self::InvalidRequest => (StatusCode::BAD_REQUEST, "Invalid request".to_string()),
            Self::Pool(err) => (StatusCode::BAD_REQUEST, err.clone()),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        json_response(status, &serde_json::json!({ "error": message }))
    }
}

fn json_response(status: StatusCode, value: &serde_json::Value) -> HttpResponse {
    let body = serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec());

    let mut response = Response::new(Full::new(Bytes::from(body)));
    *response.status_mut() = status;
    response.headers_mut().insert(
        CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );

    response
}

fn has_json_content_type(req: &Request<Incoming>) -> bool {
    req.headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
        })
        .unwrap_or(false)
}

async fn parse_json_job(req: Request<Incoming>) -> Result<MechanicsJob, ApiError> {
    if !has_json_content_type(&req) {
        return Err(ApiError::InvalidType);
    }

    let body = req
        .into_body()
        .collect()
        .await
        .map_err(|_| ApiError::InvalidRequest)?
        .to_bytes();

    serde_json::from_slice(&body).map_err(|_| ApiError::InvalidRequest)
}

async fn execute_job(
    pool: Arc<MechanicsPool>,
    job: MechanicsJob,
) -> Result<serde_json::Value, ApiError> {
    let task = tokio::task::spawn_blocking(move || pool.run(job));
    let run_result = task.await.map_err(|_| ApiError::Internal)?;
    run_result.map_err(|error| ApiError::Pool(error.to_string()))
}

async fn handle_request(
    pool: Arc<MechanicsPool>,
    req: Request<Incoming>,
) -> Result<HttpResponse, Infallible> {
    if req.method() != Method::POST || req.uri().path() != "/api/v1/mechanics" {
        return Ok(ApiError::NotFound.to_response());
    }

    let job = match parse_json_job(req).await {
        Ok(job) => job,
        Err(error) => return Ok(error.to_response()),
    };

    match execute_job(pool, job).await {
        Ok(result) => Ok(json_response(StatusCode::OK, &result)),
        Err(error) => Ok(error.to_response()),
    }
}

#[derive(Clone)]
/// HTTP server wrapper around a shared [`MechanicsPool`].
///
/// The server exposes a single endpoint:
/// `POST /api/v1/mechanics` with a JSON [`MechanicsJob`] payload.
pub struct MechanicsServer {
    pool: Arc<MechanicsPool>,
}

impl MechanicsServer {
    /// Creates a new server with an initialized worker pool.
    ///
    /// Returns an I/O error when the underlying pool creation fails.
    pub fn new(config: MechanicsPoolConfig) -> std::io::Result<Self> {
        let pool = Arc::new(MechanicsPool::new(config).map_err(std::io::Error::other)?);
        Ok(Self { pool })
    }

    /// Returns a clone of the internal shared pool handle.
    pub(crate) fn pool(&self) -> Arc<MechanicsPool> {
        Arc::clone(&self.pool)
    }

    /// Starts the HTTP server on `bind_addr` in a dedicated thread.
    ///
    /// This method is non-blocking from the caller perspective: it spawns the
    /// runtime thread and returns once the listener setup succeeds.
    ///
    /// Returns an I/O error if binding the socket, configuring non-blocking
    /// mode, or spawning the runtime thread fails.
    pub fn run(&self, bind_addr: SocketAddr) -> std::io::Result<()> {
        let std_listener = std::net::TcpListener::bind(bind_addr)?;
        std_listener.set_nonblocking(true)?;

        let server = self.clone();
        std::thread::Builder::new()
            .name("MechanicsServer".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;

                rt.block_on(async move {
                    let listener = TcpListener::from_std(std_listener)?;

                    loop {
                        let (stream, _) = listener.accept().await?;
                        let io = TokioIo::new(stream);
                        let pool = server.pool();

                        tokio::task::spawn(async move {
                            let service = service_fn(move |req| handle_request(pool.clone(), req));
                            if let Err(err) =
                                http1::Builder::new().serve_connection(io, service).await
                            {
                                eprintln!("Error serving connection: {err:?}");
                            }
                        });
                    }

                    #[allow(unreachable_code)]
                    Ok::<_, std::io::Error>(())
                })
            })?;

        Ok(())
    }
}
