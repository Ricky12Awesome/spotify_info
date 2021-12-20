use std::net::TcpListener;
use std::thread::spawn;

use tungstenite::{accept, Message};

/// Stores information about the track
#[derive(Debug, Clone, Default)]
pub struct Info {
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

        // We do not want to send back ping/pong messages.
        if let Message::Text(msg) = msg {
          let data = msg.split(';').collect::<Vec<_>>();

          let info = Info {
            title: data[0].to_string(),
            album: data[1].to_string(),
            artist: vec![data[2].to_string()],
            cover_url: Some(data[3].to_string()),
            background_url: Some(data[4].to_string()),
          };

          println!("{info:?}");
        }
      }
    });
  }
}