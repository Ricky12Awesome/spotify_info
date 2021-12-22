//! Gets metadata from spotify using a
//! [spicetify](https://github.com/khanhas/spicetify-cli)
//! extension using websockets
//!
//! More information can be found on https://github.com/Ricky12Awesome/spotify_info

use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use futures_util::TryStreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

/// The state of the track weather it's **Playing**, **Paused** or **Stopped**
///
/// Default: Stopped
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum TrackState {
  Playing = 2,
  Paused = 1,
  Stopped = 0,
}

impl TrackState {
  /// 2 will be [Self::Playing]
  ///
  /// 1 will be [Self::Paused]
  ///
  /// anything else will be [Self::Stopped]
  pub fn from_u32(n: u32) -> Self {
    match n {
      2 => Self::Playing,
      1 => Self::Paused,
      _ => Self::Stopped
    }
  }
}

impl Default for TrackState {
  fn default() -> Self {
    Self::Stopped
  }
}

/// Stores information about the track
#[derive(Debug, Clone, Default)]
pub struct TrackInfo {
  /// State of the track
  pub state: TrackState,
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

/// Stores the currently playing track
///
/// Cloning this is the same as doing [Arc::clone]
#[derive(Default, Debug, Clone)]
pub struct Handle(Arc<RwLock<Option<TrackInfo>>>);

impl Handle {
  /// Reads the track that is currently stored, this clones the value.
  pub fn read(&self) -> Option<TrackInfo> {
    self.0.try_read()
      .map(|it| it.clone())
      .ok()
      .flatten()
  }
}


pub struct Listener {
  listener: TcpListener,
}

impl Listener {
  /// Binds to 127.0.0.1:19532
  pub async fn bind_default() -> std::io::Result<Self> {
    Self::bind_local(19532).await
  }

  /// Binds to 127.0.0.1 with a custom port
  pub async fn bind_local(port: u16) -> std::io::Result<Self> {
    Self::bind(format!("127.0.0.1:{}", port).parse().unwrap()).await
  }

  /// Binds to the given address, same as calling [TcpListener::bind(addr)]
  pub async fn bind(addr: SocketAddr) -> std::io::Result<Self> {
    let listener = TcpListener::bind(addr).await?;

    Ok(Self { listener })
  }

  /// Listens for any incoming connections
  /// if it fails getting a connection it will ignore it
  /// and wait for new connections
  ///
  /// **NOTE** this is an infinite loop and will never return
  pub async fn listen(&self, handle: Handle) {
    loop {
      match self.listener.accept().await {
        Ok((stream, _)) => Self::handle_connection(stream, &handle).await,
        Err(_) => continue,
      };
    }
  }

  async fn handle_connection(stream: TcpStream, handle: &Handle) {
    if let Ok(ws) = tokio_tungstenite::accept_async(stream).await {
      let incoming = ws.try_for_each(|msg| {
        if let Message::Text(msg) = msg {
          Self::handle_message(msg, handle);
        }

        futures_util::future::ok(())
      });

      incoming.await.unwrap();
    };
  }

  fn handle_message(message: String, handle: &Handle) {
    let data = message.split(';').collect::<Vec<_>>();

    if data.len() < 6 {
      return;
    }

    let info = TrackInfo {
      state: TrackState::from_u32(data[0].parse().unwrap_or(0)),
      title: data[1].to_string(),
      album: data[2].to_string(),
      artist: vec![data[3].to_string()],
      cover_url: Some(data[4].to_string()),
      background_url: Some(data[5].to_string()),
    };

    *handle.0.write().unwrap() = Some(info);
  }
}