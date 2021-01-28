use async_std::io::prelude::*;
use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use regex::Regex;


lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<size>[0-9]+)M\s+(?P<avail>[0-9]+)M").unwrap();
}

#[derive(Deserialize, Serialize)]
pub struct DfOutput {
    pub size: u32,
    pub avail: u32,
}

pub async fn main(path: &str) -> DfOutput {
    let mut child = Command::new("df")
        .arg("-BM")
        .arg("--output=size,avail")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.as_mut().unwrap()).lines();
    lines.next().await; // skip header
    let line = lines.next().await.unwrap();
    let caps = RE.captures(line.as_ref().unwrap()).unwrap();
    let size = caps.name("size").unwrap().as_str();
    let avail = caps.name("avail").unwrap().as_str();
    DfOutput {
        size: u32::from_str_radix(size, 10).unwrap(),
        avail: u32::from_str_radix(avail, 10).unwrap(),
    }
}
