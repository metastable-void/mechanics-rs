use std::convert::Infallible;
use std::{net::SocketAddr, str::FromStr};

use mechanics::MechanicsPoolConfig;
use mechanics::MechanicsServer;

fn main() -> std::io::Result<Infallible> {
    let bind_addr: SocketAddr =
        SocketAddr::from_str(&std::env::var("LISTEN_ADDR").unwrap_or("".to_string()))
            .unwrap_or(SocketAddr::from(([127, 0, 0, 1], 3001)));
    let config = MechanicsPoolConfig::default();
    let server = MechanicsServer::new(config)?;
    if let Ok(tokens) = std::env::var("MECHANICS_ALLOWED_TOKENS") {
        for token in tokens.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            server.add_token(token.to_string());
        }
    }
    server.run(bind_addr)?;
    println!("Running mechanics server on {}", bind_addr);
    
    loop {
        std::thread::park();
    }
}
