use async_std::io::prelude::*;
use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::process::{Command, Stdio};
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<size>[0-9]+)M").unwrap();
}

#[derive(Deserialize, Serialize)]
pub struct DuOutput {
    pub size: u32,
}

pub async fn main(path: &str) -> DuOutput {
    let mut child = Command::new("du")
        .arg("-BM")
        .arg("-s")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.as_mut().unwrap()).lines();
    let line = lines.next().await.unwrap();
    let caps = RE.captures(line.as_ref().unwrap()).unwrap();
    let size = caps.name("size").unwrap().as_str();
    DuOutput {
        size: u32::from_str_radix(size, 10).unwrap(),
    }
}
