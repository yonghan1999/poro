mod lcu;

use crate::lcu::lcu_client::LcuClient;
use crate::lcu::utils::get_now_str;


#[tokio::main]
async fn main() {
    println!("{} LOL自动确认正在部署...", get_now_str());
    loop {
        println!("{} 正在连接游戏...", get_now_str());
        let client = LcuClient::new();
        client.auto_accept(true).await;
        client.exec().await;
        client.get_stop_notify().notified().await;
        println!("{} 游戏已经退出", get_now_str());
    }
}
