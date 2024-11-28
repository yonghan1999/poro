pub trait Value<T> {
    fn value(&self) -> T;

    fn from_value(val: T) -> Self;
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

    fn from_value(val: i32) -> Self {
        match val {
            5 => Operator::Sub,
            6 => Operator::DisSub,
            8 => Operator::Event,
            _ => unreachable!(),
        }
    }
}

pub enum Event {
    OnJsonApiEvent,
}

impl Value<&str> for Event {
    fn value(&self) -> &'static str {
        match self {
            Event::OnJsonApiEvent => "OnJsonApiEvent",
        }
    }

    fn from_value(val: &str) -> Self {
        match val {
            "OnJsonApiEvent" => Event::OnJsonApiEvent,
            _ => unreachable!(),
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum GameState {
    None,
    Lobby,
    MatchMaking,
    ReadyCheck,
    ChampSelect,
    GameStart,
    InProgress,
    PreEndOfGame,
    WaitingForStats,
    EndOfGame,
    Reconnect,
    WatchInProgress,
}
impl Value<&str> for GameState {
    fn value(&self) -> &'static str {
        match self {
            GameState::None => "None",
            GameState::Lobby => "Lobby",
            GameState::MatchMaking => "Matchmaking",
            GameState::ReadyCheck => "ReadyCheck",
            GameState::ChampSelect => "ChampSelect",
            GameState::GameStart => "GameStart",
            GameState::InProgress => "InProgress",
            GameState::PreEndOfGame => "PreEndOfGame",
            GameState::WaitingForStats => "WaitingForStats",
            GameState::EndOfGame => "EndOfGame",
            GameState::Reconnect => "Reconnect",
            GameState::WatchInProgress => "WatchInProgress",
        }
    }

    fn from_value(val: &str) -> Self {
        match val {
            "None" => GameState::None,
            "Lobby" => GameState::Lobby,
            "MatchMaking" => GameState::MatchMaking,
            "ReadyCheck" => GameState::ReadyCheck,
            "ChampSelect" => GameState::ChampSelect,
            "GameStart" => GameState::GameStart,
            "InProgress" => GameState::InProgress,
            "PreEndOfGame" => GameState::PreEndOfGame,
            "WaitingForStats" => GameState::WaitingForStats,
            "EndOfGame" => GameState::EndOfGame,
            "Reconnect" => GameState::Reconnect,
            "WatchInProgress" => GameState::WatchInProgress,
            _ => unreachable!(),
        }
    }
}

pub mod lcu_api {
    // 游戏状态
    pub const GAMEFLOW_PHASE: &str = "/lol-gameflow/v1/gameflow-phase";
    // 接受对局
    pub const GAME_ACCEPT: &str = "/lol-matchmaking/v1/ready-check/accept";
    // 再来一局
    pub const PLAY_AGAIN: &str = "/lol-lobby/v2/play-again";
    // 寻找对局
    pub const GAME_SEARCH: &str = "/lol-lobby/v2/lobby/matchmaking/search";
    // 给队友点赞
    pub const HONOR_PLAYER: &str = "/lol-honor-v2/v1/honor-player";
}
