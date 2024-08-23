#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub use poem;
#[cfg(feature = "web-server-grpc")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server-grpc")))]
pub use poem_grpc;
#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub use poem_openapi;
#[cfg(feature = "web-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws-client")))]
pub use reqwest;
#[cfg(feature = "ws-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws-client")))]
pub use tokio_tungstenite;

#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub mod context_extractor;
#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub mod uniform_error_mw;
#[cfg(feature = "web-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-client")))]
pub mod web_client;

// #[cfg(feature = "web-client")]
// #[cfg_attr(docsrs, doc(cfg(feature = "web-client")))]
// pub mod web_client_v2;
#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub mod web_resp;
#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub mod web_server;
#[cfg(feature = "web-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "web-server")))]
pub mod web_validation;
#[cfg(feature = "ws-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "ws-client")))]
pub mod ws_client;
