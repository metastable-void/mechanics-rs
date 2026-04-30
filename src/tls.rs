use std::io;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use tokio_rustls::server::TlsStream;

/// TLS configuration for [`super::MechanicsServer::run_tls`].
///
/// Accepts PEM-encoded certificate chain and private key bytes.
/// The crypto backend is vendored (aws-lc-rs / ring via rustls) —
/// no system OpenSSL headers are required.
pub struct TlsConfig {
    cert_chain: Vec<CertificateDer<'static>>,
    private_key: PrivateKeyDer<'static>,
}

impl TlsConfig {
    /// Creates a TLS configuration from PEM-encoded bytes.
    ///
    /// `cert_pem` should contain one or more PEM-encoded certificates
    /// (leaf first, then intermediates). `key_pem` should contain a
    /// single PEM-encoded private key (PKCS#8 or SEC1/EC or RSA).
    pub fn from_pem(cert_pem: &[u8], key_pem: &[u8]) -> io::Result<Self> {
        let cert_chain: Vec<CertificateDer<'static>> = CertificateDer::pem_slice_iter(cert_pem)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        if cert_chain.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "no certificates found in PEM data",
            ));
        }

        let private_key = PrivateKeyDer::from_pem_slice(key_pem)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        Ok(Self {
            cert_chain,
            private_key,
        })
    }

    pub(crate) fn into_acceptor(self) -> io::Result<Acceptor> {
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(self.cert_chain, self.private_key)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(Acceptor {
            inner: TlsAcceptor::from(Arc::new(config)),
        })
    }
}

pub(crate) struct Acceptor {
    inner: TlsAcceptor,
}

impl Acceptor {
    pub(crate) async fn accept(&self, stream: TcpStream) -> io::Result<TlsStream<TcpStream>> {
        self.inner.accept(stream).await
    }
}
