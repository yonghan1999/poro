use super::lcu_listener::{LcuData, LcuWebsocket};
use crate::lcu::constants::{lcu_api, GameState, Value};
use crate::lcu::utils::{gen_lcu_auth, get_lol_client_connect_info, get_now_str};
use reqwest::{header, Client};
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Notify, RwLock};
use tokio::time::sleep;

pub type Callback = fn() -> Pin<Box<dyn Future<Output=()> + Send>>;

struct CallbackRes {}


pub struct LcuClient {
    websocket: Arc<RwLock<Option<LcuWebsocket>>>,
    game_flow_actions: Arc<RwLock<HashMap<GameState, Vec<Callback>>>>,
    stop_notify: Arc<Notify>,
}

impl LcuClient {
    pub fn new() -> Self {
        let listener = Arc::new(RwLock::new(None));
        let c_listener = listener.clone();
        let stop_notify = Arc::new(Notify::new());
        let c_stop_notify = stop_notify.clone();

        tokio::spawn(async move {
            // 获取websocket连接，带重试逻辑
            let mut my_listener = c_listener.write().await;
            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 10;
            
            loop {
                let l_stop_notify = c_stop_notify.clone();
                if my_listener.is_none() {
                    match LcuWebsocket::new(l_stop_notify).await {
                        Ok(ws) => {
                            *my_listener = Some(ws);
                            println!("{} WebSocket连接建立成功", get_now_str());
                            break;
                        }
                        Err(_) => {
                            retry_count += 1;
                            if retry_count >= MAX_RETRIES {
                                println!("{} WebSocket连接失败{}次，停止尝试", get_now_str(), retry_count);
                                break;
                            }
                            println!("{} WebSocket连接失败{}次，等待1秒后重试...", get_now_str(), retry_count);
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        let actions = Arc::new(RwLock::new(HashMap::new()));
        LcuClient {
            websocket: listener,
            game_flow_actions: actions,
            stop_notify,
        }
    }

    pub async fn add_game_flow_action(&self, game_state: GameState, callback: Callback) {
        let mut game_flow_actions = self.game_flow_actions.write().await;
        let res = game_flow_actions.get_mut(&game_state);
        if let Some(callback_list) = res {
            callback_list.push(callback);
        } else {
            let mut callback_list = Vec::new();
            callback_list.push(callback);
            game_flow_actions.insert(game_state, callback_list);
        }
    }

    pub async fn remove_game_flow_action(&self, game_state: GameState, index: usize) {
        let mut game_flow_actions = self.game_flow_actions.write().await;
        if let Some(callback_list) = game_flow_actions.get_mut(&game_state) {
            callback_list.remove(index);
        }
    }

    pub fn get_stop_notify(&self) -> Arc<Notify> {
        self.stop_notify.clone()
    }

    pub fn get_event_listener(&self) -> Arc<RwLock<Option<LcuWebsocket>>> {
        self.websocket.clone()
    }

    pub async fn reconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // 重置HTTP客户端
        reset_lcu_http_client()?;
        
        // 重新创建WebSocket连接
        let mut websocket = self.websocket.write().await;
        *websocket = None;
        
        // 尝试重新连接
        let stop_notify = self.stop_notify.clone();
        loop {
            match LcuWebsocket::new(stop_notify.clone()).await {
                Ok(new_websocket) => {
                    *websocket = Some(new_websocket);
                    println!("{} 重新连接成功！", get_now_str());
                    return Ok(());
                }
                Err(e) => {
                    println!("{} 重新连接失败: {}，等待5秒后重试...", get_now_str(), e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    pub async fn exec(&self) {
        let c_listener = self.websocket.clone();
        let notify = self.get_stop_notify();
        let actions = self.game_flow_actions.clone();
        tokio::spawn(async move {
            let listener = c_listener;
            let mut rx;
            let mut connection_attempts = 0;
            const MAX_CONNECTION_ATTEMPTS: u32 = 30; // 最多尝试30秒
            
            // 尝试连接本地英雄联盟客户端
            loop {
                let listener = listener.read().await;
                if let Some(a) = listener.as_ref() {
                    rx = a.data.read().await.subscribe();
                    println!("{} 成功订阅游戏事件", get_now_str());
                    break;
                }
                
                connection_attempts += 1;
                if connection_attempts >= MAX_CONNECTION_ATTEMPTS {
                    println!("{} 连接游戏失败，超过最大重试次数", get_now_str());
                    notify.notify_one();
                    return;
                }
                
                // 如果连接失败，则等待1s后重新连接
                sleep(Duration::from_secs(1)).await;
            }
            
            // 监听游戏事件
            loop {
                match rx.recv().await {
                    Ok(lcu_data) => {
                        let a = actions.clone();
                        Self::match_data(a, lcu_data).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // 如果落后了，继续接收新消息
                        println!("{} 消息处理落后，跳过一些消息", get_now_str());
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // 连接已关闭，可能是游戏重启了
                        println!("{} WebSocket连接已关闭，可能是游戏重启了", get_now_str());
                        break;
                    }
                }
            }
            notify.notify_one();
        });
    }

    async fn match_data(actions: Arc<RwLock<HashMap<GameState, Vec<Callback>>>>, lcu_data: LcuData) {
        match lcu_data.uri.as_str() {
            // 游戏状态
            lcu_api::GAMEFLOW_PHASE => {
                let state = lcu_data.data.as_str().unwrap();
                let game_state = GameState::from_value(state);
                if game_state == GameState::EndOfGame {
                    println!("{} {:?}\n\n\n",get_now_str(), &lcu_data);
                }
                let actions = actions.read().await;
                let res = actions.get(&game_state);
                if let Some(callbacks) = res {
                    for callback in callbacks {
                        callback().await;
                    }
                }
            }
            _ => {
                // TODO：暂未实现其他状态的功能
            }
        }
    }
}

static mut LCU_HTTP_CLIENT: Option<Arc<RwLock<LcuHttpClient>>> = None;
static mut ONCE: std::sync::Once = std::sync::Once::new();

pub(in crate::lcu) struct LcuHttpClient {
    pub(in crate::lcu) client: Client,
    pub(in crate::lcu) url: String,
}

pub(in crate::lcu) fn get_lcu_http_client() -> Arc<RwLock<LcuHttpClient>> {
    unsafe {
        if LCU_HTTP_CLIENT.is_some() {
            return LCU_HTTP_CLIENT.as_ref().unwrap().clone();
        } else {
            let mut headers = header::HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json; charset=utf-8"));
            headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json; charset=utf-8"));
            let connect_info = get_lol_client_connect_info().unwrap();
            let auth = gen_lcu_auth("riot", &connect_info.token.clone());
            headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(auth.as_str()).unwrap());
            let client = Client::builder()
                .default_headers(headers)
                .danger_accept_invalid_certs(true)
                .build().unwrap();
            let url = format!("https://{}:{}", "127.0.0.1", connect_info.port);
            let instance = LcuHttpClient { client, url };
            let _ = LCU_HTTP_CLIENT.insert(Arc::new(RwLock::new(instance)));
            LCU_HTTP_CLIENT.as_ref().unwrap().clone()
        }
    }
}

pub(in crate::lcu) fn reset_lcu_http_client() -> Result<(), Box<dyn Error + Send + Sync>> {
    unsafe {
        if LCU_HTTP_CLIENT.is_some() {
            // 获取新的连接信息
            let connect_info = match get_lol_client_connect_info() {
                Ok(info) => info,
                Err(e) => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
            };
            let mut headers = header::HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json; charset=utf-8"));
            headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json; charset=utf-8"));
            let auth = gen_lcu_auth("riot", &connect_info.token);
            headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(auth.as_str()).unwrap());
            let client = Client::builder()
                .default_headers(headers)
                .danger_accept_invalid_certs(true)
                .build()?;
            let url = format!("https://{}:{}", "127.0.0.1", connect_info.port);
            let instance = LcuHttpClient { client, url };
            LCU_HTTP_CLIENT = Some(Arc::new(RwLock::new(instance)));
            Ok(())
        } else {
            // 如果还没有初始化，直接创建新的
            let _ = get_lcu_http_client();
            Ok(())
        }
    }
}
