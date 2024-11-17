use std::error::Error;
use std::f32::consts::E;
use std::io::{BufReader, Read};
use std::process::{Command, Stdio};
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{DateTime, Local};
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use serde::de::Unexpected::Str;

#[derive(Debug)]
pub(super) struct LolClientConnectInfo {
    pub port: i32,
    pub token: String,
}

pub(super) fn get_lol_client_connect_info() -> Result<LolClientConnectInfo, Box<dyn Error>> {
    // 查询lol的启动进程
    let output = Command::new("wmic")
        .args(&[
            "PROCESS",
            "WHERE",
            "name='LeagueClientUx.exe'",
            "GET",
            "commandline",
        ])
        .stdout(Stdio::piped())
        .spawn()?;
    if let Some(stdout) = output.stdout {
        let mut reader = BufReader::new(stdout);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        // windows 命令需要GBK编码
        if let Ok(line) = GBK.decode(&buffer, DecoderTrap::Strict) {

            //通过进程名查询出进程的启动命令,解析出需要的客户端token和端口
            let token_pattern = regex::Regex::new(r"--remoting-auth-token=([\w-]+)")?;
            let mut token = String::from("");
            if let Some(captures) = token_pattern.captures(&line) {
                if let Some(t) = captures.get(1) {
                    token = t.as_str().to_string();
                }
            }
            let port_pattern = regex::Regex::new(r"--app-port=(\d+)")?;
            let mut port = 0;
            if let Some(captures) = port_pattern.captures(&line) {
                if let Some(t) = captures.get(1) {
                    port = t.as_str().parse::<i32>()?;
                }
            }

            if port == 0 || token.is_empty() {
                return Err(From::from("Couldn't get lol client connect"));
            }

            return Ok(LolClientConnectInfo { port, token });
        }
    }
    Err(From::from("Couldn't get lol client connect"))
}

pub(super) fn gen_lcu_auth(username: &str, password: &str) -> String {
    let credentials = format!("{}:{}", username, password);
    let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
    format!("Basic {}", encoded)
}

pub fn get_now_str() -> String {
    let now: DateTime<Local> = Local::now();
    let time_str = now.time().format("%H:%M:%S").to_string();
    return time_str;
}