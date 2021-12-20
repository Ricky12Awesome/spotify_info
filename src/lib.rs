use std::net::TcpListener;
use std::thread::spawn;

use tungstenite::{accept, Message};

#[repr(u32)]
#[derive(Debug, Clone)]
pub enum State {
  Playing = 2,
  Paused = 1,
  Stopped = 0,
}

impl State {
  pub fn from_u32(n: u32) -> State {
    match n {
      2 => State::Playing,
      1 => State::Paused,
      _ => State::Stopped
    }
  }
}

impl Default for State {
  fn default() -> Self {
    Self::Stopped
  }
}

/// Stores information about the track
#[derive(Debug, Clone, Default)]
pub struct Info {
  /// State of the track
  pub state: State,
  /// Title of the track
  pub title: String,
  /// Album of the track
  pub album: String,
  /// Vec since there can be multiple artists
  pub artist: Vec<String>,
  /// Cover art of the track, option because it may not exist
  pub cover_url: Option<String>,
  /// Background art of the track, option because it may nto exist
  /// (when you hit the "full screen" thing in the bottom-right corner of spotify)
  pub background_url: Option<String>,
}

pub fn websocket() {
  let server = TcpListener::bind("127.0.0.1:19532").unwrap();

  for stream in server.incoming() {
    spawn(move || {
      let mut websocket = accept(stream.unwrap()).unwrap();

      loop {
        let msg = websocket.read_message().unwrap();

        if let Message::Text(msg) = msg {
          let data = msg.split(';').collect::<Vec<_>>();

          let info = Info {
            state: State::from_u32(data[0].parse().unwrap()),
            title: data[1].to_string(),
            album: data[2].to_string(),
            artist: vec![data[3].to_string()],
            cover_url: Some(data[4].to_string()),
            background_url: Some(data[5].to_string()),
          };

          println!("{info:?}");
        }
      }
    });
  }
}