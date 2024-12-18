mod lcu;

use crate::lcu::constants::GameState;
use crate::lcu::lcu_client::LcuClient;
use crate::lcu::lcu_client_util::{accept_game, play_again};
use crate::lcu::utils::get_now_str;


#[tokio::main]
async fn main() {
    println!("{} 启动中...", get_now_str());
    loop {
        println!("{} 正在连接游戏...", get_now_str());
        let client = LcuClient::new();
        println!("{} 启动完成", get_now_str());
        client.add_game_flow_action(GameState::ReadyCheck, accept_game).await;
        println!("{} 自动接受对局功能准备完成...", get_now_str());
        // client.add_game_flow_action(GameState::EndOfGame, play_again).await;
        println!("{} 自动再来一局功能准备完成...", get_now_str());
        client.exec().await;
        client.get_stop_notify().notified().await;
        println!("{} 游戏已经退出", get_now_str());
    }
}



