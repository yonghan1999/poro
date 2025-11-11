mod lcu;

use crate::lcu::constants::GameState;
use crate::lcu::lcu_client::LcuClient;
use crate::lcu::lcu_client_util::accept_game;
use crate::lcu::utils::get_now_str;


#[tokio::main]
async fn main() {
    println!("{} 启动中...", get_now_str());
    let mut reconnection_count = 0;
    
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
        
        // 检查是否是游戏重启导致的连接中断
        println!("{} 检测到游戏连接中断，正在尝试重新连接...", get_now_str());
        reconnection_count += 1;
        
        // 创建新的客户端实例来重新连接
        println!("{} 创建新的连接实例...", get_now_str());
        
        if reconnection_count >= 5 {
            println!("{} 连续重连失败{}次，等待30秒后重试...", get_now_str(), reconnection_count);
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            reconnection_count = 0;
        } else {
            // 等待一段时间后重试
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}



