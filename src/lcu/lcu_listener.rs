use super::{constants::{self, Value as ConstantValue}, utils::{gen_lcu_auth, get_lol_client_connect_info}};
use futures::SinkExt;
use futures_util::StreamExt;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{broadcast, RwLock},
};
use tokio_tungstenite::Connector::NativeTls;
use tokio_tungstenite::{
    tungstenite::{protocol::WebSocketConfig, ClientRequestBuilder, Message},
    MaybeTlsStream, WebSocketStream,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcuData {
    pub data: serde_json::Value,
    pub event_type: String,
    pub uri: String,
}

pub struct LcuListener {
    pub data: Arc<RwLock<broadcast::Sender<LcuData>>>,
    pub socket: Arc<RwLock<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
}
impl LcuListener {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let connect_info_res = get_lol_client_connect_info();
        let connect_info = connect_info_res?;
        // allow invalid ssl certificates
        let connector = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?;
        // connect to the local websocket
        let url = format!("wss://127.0.0.1:{}/", connect_info.port.clone()).parse()?;
        let auth = gen_lcu_auth("riot", &connect_info.token.clone());
        let request =
            ClientRequestBuilder::new(url).with_header("authorization", auth);
        let (mut socket, _) = tokio_tungstenite::connect_async_tls_with_config(
            request,
            Some(WebSocketConfig::default()),
            false,
            Some(NativeTls(connector)),
        )
            .await?;
        // subscribe to all events
        let message = format!(
            r#"[{}, "{}"]"#,
            constants::Operator::Sub.value(),
            constants::Event::OnJsonApiEvent.value()
        );

        socket
            .send(Message::Text(message.to_owned()))
            .await?;
        let (tx, _) = broadcast::channel(100);
        let lcu_listener = LcuListener {
            data: Arc::new(RwLock::new(tx)),
            socket: Arc::new(RwLock::new(socket)),
        };

        let c_socket = lcu_listener.socket.clone();
        let c_data = lcu_listener.data.clone();

        tokio::spawn(async move {
            let mut socket = c_socket.write().await;
            let broadcast = c_data.read().await;
            loop {
                while let Some(msg) = socket.next().await {
                    if let Ok(msg) = msg {
                        if msg.is_empty() { continue; }
                        let data: (i32, String, Option<LcuData>) =
                            serde_json::from_str(&msg.to_string()).unwrap();
                        if data.2.is_some() {
                            let lcu_data = data.2.unwrap().clone();
                            let _ = broadcast.send(lcu_data);
                        }
                    } else {
                    }
                }
            }
        });
        Ok(lcu_listener)
    }
}

