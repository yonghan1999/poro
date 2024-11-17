use super::lcu_listener::{LcuData, LcuListener};
use crate::lcu::constants::{game_state, lcu_api};
use crate::lcu::utils::{gen_lcu_auth, get_lol_client_connect_info, get_now_str};
use reqwest::{header, Client};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use chrono::Local;
use tokio::sync::{Notify, RwLock};
use tokio::time::sleep;

struct LcuClientConfig {
    auto_accept: bool,
}

pub struct LcuClient {
    listener: Arc<RwLock<Option<LcuListener>>>,
    stop_notify: Arc<Notify>,
    config: Arc<RwLock<LcuClientConfig>>,
}

impl LcuClient {
    pub fn new() -> Self {
        let listener = Arc::new(RwLock::new(None));
        let c_listener = listener.clone();

        tokio::spawn(async move {
            // 获取websocket
            let mut my_listener = c_listener.write().await;
            loop {
                if my_listener.is_none() {
                    if let Ok(listener) = LcuListener::new().await {
                        *my_listener = Some(listener);
                        break;
                    }
                    sleep(Duration::from_millis(500)).await;
                } else {
                    break;
                }
            }
        });

        let config = Arc::new(RwLock::new(
            LcuClientConfig {
                auto_accept: false
            }
        ));
        let stop_notify = Arc::new(Notify::new());
        LcuClient {
            listener,
            stop_notify,
            config,
        }
    }

    pub fn get_stop_notify(&self) -> Arc<Notify> {
        self.stop_notify.clone()
    }

    pub fn get_event_listener(&self) -> Arc<RwLock<Option<LcuListener>>> {
        self.listener.clone()
    }

    pub async fn exec(&self) {
        let c_listener = self.listener.clone();
        let notify = self.stop_notify.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            let listener = c_listener;
            let mut rx;
            loop {
                let listener = listener.read().await;
                if let Some(a) = listener.as_ref() {
                    rx = a.data.read().await.subscribe();
                    break;
                }
                sleep(Duration::from_millis(500)).await;
            }
            println!("{} 已连接游戏", get_now_str());
            while let Ok(lcu_data) = rx.recv().await {
                Self::match_data(config.clone(), lcu_data).await;
            }
            notify.notify_one();
        });
    }

    async fn match_data(config: Arc<RwLock<LcuClientConfig>>, lcu_data: LcuData) {
        match lcu_data.uri.as_str() {
            // 游戏状态
            lcu_api::GAMEFLOW_PHASE => {
                let game_state = lcu_data.data.as_str().unwrap();
                if game_state == game_state::READY_CHECK {
                    println!("{} 找到对局！", get_now_str());
                    Self::game_ready(config.clone(), lcu_data).await;
                } else if game_state == game_state::MATCH_MAKING {
                    println!("{} 正在队列中...", get_now_str());
                } else if game_state == game_state::CHAMP_SELECT {
                    println!("{} 选择英雄中...", get_now_str());
                } else if game_state == game_state::GAME_START {
                    println!("{} 游戏开始", get_now_str());
                } else if game_state == game_state::IN_PROGRESS {
                    println!("{} 游戏中...", get_now_str());
                } else if game_state == game_state::PRE_END_OF_GAME {
                    println!("{} 游戏即将结束...", get_now_str());
                } else if game_state == game_state::WAITING_FOR_STATS {
                    println!("{} 等待结算界面", get_now_str());
                } else if game_state == game_state::END_OF_GAME {
                    println!("{} 游戏结束", get_now_str());
                } else if game_state == game_state::NONE {
                    println!("{} 在游戏主界面等待中...", get_now_str());
                } else if game_state == game_state::LOBBY {
                    println!("{} 在房间中，等待开始游戏...", get_now_str());
                } else {
                    // todo!()
                }
            }
            _ => {}
        }
    }

    async fn game_ready(config: Arc<RwLock<LcuClientConfig>>, lcu_data: LcuData) {
        // 自动接受对局
        let auto_accept = config.read().await.auto_accept;
        if auto_accept {
            let instance = get_lcu_http_client();
            let lcu_client = instance.read().await;
            lcu_client.client.post(format!("{}{}", lcu_client.url.clone(), lcu_api::GAME_ACCEPT)).body("").send().await.unwrap();
            println!("{} 已自动接受对局。", get_now_str())
        }
    }

    pub async fn auto_accept(&self, enable: bool) {
        self.config.write().await.auto_accept = enable;
    }
}

static mut LCU_HTTP_CLIENT: Option<Arc<RwLock<LcuHttpClient>>> = None;
static mut ONCE: std::sync::Once = std::sync::Once::new();

struct LcuHttpClient {
    client: Client,
    url: String,
}

fn get_lcu_http_client() -> Arc<RwLock<LcuHttpClient>> {
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
            ONCE.call_once(|| {
                let instance = LcuHttpClient { client, url };
                let _ = LCU_HTTP_CLIENT.insert(Arc::new(RwLock::new(instance)));
            });
            return LCU_HTTP_CLIENT.as_ref().unwrap().clone();
        }
    }
}
