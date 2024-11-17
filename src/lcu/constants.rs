pub trait Value<T> {
    fn value(&self) -> T;
}

pub enum Operator {
    Sub,
    DisSub,
    Event,
}

impl Value<i32> for Operator {
    fn value(&self) -> i32 {
        match self {
            Operator::Sub => 5,
            Operator::DisSub => 6,
            Operator::Event => 8,
        }
    }
}

pub enum Event {
    OnJsonApiEvent,
}

impl Value<&'static str> for Event {
    fn value(&self) -> &'static str {
        match self {
            Event::OnJsonApiEvent => "OnJsonApiEvent",
        }
    }
}

pub mod game_state {
    // 在大厅
    pub const NONE: &str = "None";
    // 在房间中
    pub const LOBBY: &str = "Lobby";
    // 队列中
    pub const MATCH_MAKING: &str = "Matchmaking";
    // 找到对局等待接受
    pub const READY_CHECK: &str = "ReadyCheck";
    // 选择英雄中
    pub const CHAMP_SELECT: &str = "ChampSelect";
    // 游戏开始
    pub const GAME_START: &str = "GameStart";
    // 游戏中
    pub const IN_PROGRESS: &str = "InProgress";
    // 游戏即将结束
    pub const PRE_END_OF_GAME: &str = "PreEndOfGame";
    // 等待结算界面
    pub const WAITING_FOR_STATS: &str = "WaitingForStats";
    // 游戏结束
    pub const END_OF_GAME: &str = "EndOfGame";
    // 重新连接
    pub const RECONNECT: &str = "Reconnect";
    // 观战中
    pub const WATCH_IN_PROGRESS: &str = "WatchInProgress";
    
}

pub mod lcu_api {
    // 游戏状态
    pub const GAMEFLOW_PHASE: &str = "/lol-gameflow/v1/gameflow-phase";
    // 接受对局
    pub const GAME_ACCEPT: &str = "/lol-matchmaking/v1/ready-check/accept";
}
