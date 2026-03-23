use std::convert::Infallible;
use std::time::Duration;
use std::{net::SocketAddr, str::FromStr};

use mechanics::MechanicsPoolConfig;
use mechanics::MechanicsServer;
use mechanics_core::job::MechanicsExecutionLimits;

fn main() -> std::io::Result<Infallible> {
    let bind_addr: SocketAddr =
        SocketAddr::from_str(&std::env::var("LISTEN_ADDR").unwrap_or("".to_string()))
            .unwrap_or(SocketAddr::from(([127, 0, 0, 1], 3001)));
    let mut config = MechanicsPoolConfig::default();
    config = config.with_execution_limits(MechanicsExecutionLimits::new(Duration::from_secs(3600), 65536, 65536, 131072).map_err(std::io::Error::other)?);
    config = config.with_run_timeout(Duration::from_secs(3600));
    config = config.with_default_http_timeout_ms(Some(300_000));
    let server = MechanicsServer::new(config)?;
    let mut token_count = 0usize;
    if let Ok(tokens) = std::env::var("MECHANICS_ALLOWED_TOKENS") {
        for token in tokens.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            server.add_token(token.to_string());
            token_count += 1;
        }
    }
    server.run(bind_addr)?;
    println!("Running mechanics server on {}", bind_addr);
    if token_count == 0 {
        println!(
            "No tokens configured via MECHANICS_ALLOWED_TOKENS. Requests will be denied until tokens are added."
        );
    } else {
        println!("Loaded {} bearer token(s).", token_count);
    }

    loop {
        std::thread::park();
    }
}
