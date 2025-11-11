use super::{constants::{self, Value as ConstantValue}, utils::{gen_lcu_auth, get_lol_client_connect_info, get_now_str}};
use futures::SinkExt;
use futures_util::StreamExt;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Notify;
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

pub struct LcuWebsocket {
    pub data: Arc<RwLock<broadcast::Sender<LcuData>>>,
    pub socket: Arc<RwLock<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pub stop_notify: Arc<Notify>,
}
impl LcuWebsocket {
    pub async fn new(stop_notify: Arc<Notify>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let connect_info = match get_lol_client_connect_info() {
            Ok(info) => info,
            Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
        };
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
        let lcu_listener = LcuWebsocket {
            data: Arc::new(RwLock::new(tx)),
            socket: Arc::new(RwLock::new(socket)),
            stop_notify: stop_notify.clone(),
        };

        let c_socket = lcu_listener.socket.clone();
        let c_data = lcu_listener.data.clone();
        let c_notify = lcu_listener.stop_notify.clone();

        tokio::spawn(async move {
            let mut socket = c_socket.write().await;
            let broadcast = c_data.read().await;
            while let Some(msg_result) = socket.next().await {
                match msg_result {
                    Ok(msg) => {
                        if msg.is_empty() { continue; }
                        let data: Result<(i32, String, Option<LcuData>), _> =
                            serde_json::from_str(&msg.to_string());
                        
                        match data {
                            Ok(parsed_data) => {
                                if parsed_data.2.is_some() {
                                    let lcu_data = parsed_data.2.unwrap().clone();
                                    let _ = broadcast.send(lcu_data);
                                }
                            }
                            Err(e) => {
                                println!("{} 解析消息失败: {}", get_now_str(), e);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} WebSocket连接错误: {}，连接将关闭", get_now_str(), e);
                        c_notify.notify_one();
                        break;
                    }
                }
            }
            // 连接已关闭，通知监听器
            println!("{} WebSocket连接已关闭", get_now_str());
            c_notify.notify_one();
        });
        Ok(lcu_listener)
    }
}

