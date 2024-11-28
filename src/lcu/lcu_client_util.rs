use crate::lcu::constants::lcu_api;
use crate::lcu::lcu_client::get_lcu_http_client;
use crate::lcu::utils::get_now_str;
use std::future::Future;
use std::pin::Pin;

/// 接受对局
pub fn accept_game() -> Pin<Box<dyn Future<Output=()> + Send>> {
    Box::pin(async {
        let instance = get_lcu_http_client();
        let lcu_client = instance.read().await;
        lcu_client.client.post(format!("{}{}", lcu_client.url.clone(), lcu_api::GAME_ACCEPT)).body("").send().await.unwrap();
        println!("{} 已自动接受对局。", get_now_str());
    })
}

/// 再来一局
pub fn play_again() -> Pin<Box<dyn Future<Output=()> + Send>> {
    Box::pin(async {
        let instance = get_lcu_http_client();
        let lcu_client = instance.read().await;
        lcu_client.client.post(format!("{}{}", lcu_client.url.clone(), lcu_api::PLAY_AGAIN)).body("").send().await.unwrap();
        println!("{} 已自动再来一局。", get_now_str());
    })
}

/// 寻找对局
pub fn search_game() -> Pin<Box<dyn Future<Output=()> + Send>> {
    Box::pin(async {
        let instance = get_lcu_http_client();
        let lcu_client = instance.read().await;
        lcu_client.client.post(format!("{}{}", lcu_client.url.clone(), lcu_api::GAME_SEARCH)).body("").send().await.unwrap();
        println!("{} 已自动寻找对局。", get_now_str());
    })
}