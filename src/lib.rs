//! Gets metadata from spotify using a
//! [spicetify](https://github.com/khanhas/spicetify-cli)
//! extension using websockets
//!
//! More information can be found on https://github.com/Ricky12Awesome/spotify_info

use std::{
  fmt::{Display, Formatter},
  io::ErrorKind,
  net::SocketAddr,
  time::Duration,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde_json")]
use serde_json::{from_str, to_string};

#[cfg(feature = "miniserde")]
use miniserde::json::{from_str, to_string};
#[cfg(feature = "miniserde")]
use miniserde::{Deserialize, Serialize};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
  accept_async,
  tungstenite::{Error, Message},
  WebSocketStream,
};

/// The state of the track weather it's **Playing**, **Paused** or **Stopped**
///
/// Default: Stopped
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TrackState {
  Playing = 2,
  Paused = 1,
  Stopped = 0,
}

impl Display for TrackState {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      TrackState::Playing => write!(f, "Playing"),
      TrackState::Paused => write!(f, "Paused"),
      TrackState::Stopped => write!(f, "Stopped"),
    }
  }
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
      _ => Self::Stopped,
    }
  }
}

impl Default for TrackState {
  fn default() -> Self {
    Self::Stopped
  }
}

/// Stores information about the track
#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct TrackInfo {
  /// UID of track
  pub uid: String,
  // URI of track
  pub uri: String,
  /// State of the track
  pub state: TrackState,
  /// Duration of the track
  pub duration: Duration,
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

impl TrackInfo {
  pub fn eq_ignore_state(&self, other: &Self) -> bool {
    self.uid == other.uid
  }
}

#[derive(Debug)]
pub enum SpotifyEvent {
  /// Gets called when user changes track
  TrackChanged(TrackInfo),
  /// Gets called when user changes state (if song is playing, paused or stopped)
  ///
  /// **NOTE**: Doesn't get called when user changes track
  StateChanged(TrackState),
  /// Gets called on a set interval, wont get called if player is paused or stopped,
  /// Value is a percentage of the position between 0 and 1
  ///
  /// **NOTE**: Doesn't get called when user changes track
  ProgressChanged(f64),
}

pub struct SpotifyListener {
  pub listener: TcpListener,
}

#[derive(Debug)]
pub struct SpotifyConnection {
  pub ws: WebSocketStream<TcpStream>,
}

impl SpotifyConnection {
  fn parse_track_info(data: &[&str]) -> TrackInfo {
    TrackInfo {
      uid: data[0].to_string(),
      uri: data[1].to_string(),
      state: TrackState::from_u32(data[2].parse().unwrap_or(0)),
      duration: Duration::from_millis(data[3].parse().unwrap_or(0)),
      title: data[4].to_string().replace("${#{#{SEMI_COLON}#}#}$", ";"),
      album: data[5].to_string().replace("${#{#{SEMI_COLON}#}#}$", ";"),
      artist: vec![data[6].to_string().replace("${#{#{SEMI_COLON}#}#}$", ";")],
      cover_url: Some(data[7].to_string()).filter(|it| !it.contains("NONE")),
      background_url: Some(data[8].to_string()).filter(|it| !it.contains("NONE")),
    }
  }

  fn handle_message(message: String) -> Option<Result<SpotifyEvent, Error>> {
    let mut data = message.split(';').collect::<Vec<_>>();
    let invalid_data_err = Some(Err(Error::Io(std::io::Error::new(
      ErrorKind::InvalidData,
      "Invalid data",
    ))));

    if data.is_empty() {
      return invalid_data_err;
    }

    match data.remove(0) {
      "TRACK_CHANGED" if data.len() >= 9 => {
        let info = Self::parse_track_info(&data);

        Some(Ok(SpotifyEvent::TrackChanged(info)))
      }
      "STATE_CHANGED" if !data.is_empty() => {
        let state = TrackState::from_u32(data[0].parse().unwrap_or(0));

        Some(Ok(SpotifyEvent::StateChanged(state)))
      }
      "PROGRESS_CHANGED" if !data.is_empty() => {
        let progress = data[0].parse().unwrap_or(0f64);

        Some(Ok(SpotifyEvent::ProgressChanged(progress)))
      }
      _ => invalid_data_err,
    }
  }

  /// Sets how often it should update the progress,
  ///
  /// by default it's set to 1 second
  pub async fn set_progress_interval(&mut self, interval: Duration) -> Result<(), Error> {
    let ms = interval.as_millis();
    let text = format!("SET_PROGRESS_INTERVAL;{ms}");

    self.ws.send(Message::Text(text)).await
  }

  /// Waits for the next message to be received
  pub async fn next(&mut self) -> Option<Result<SpotifyEvent, Error>> {
    let message = self.ws.next().await?;

    match message {
      Ok(Message::Text(message)) => Self::handle_message(message),
      Ok(_) => Some(Err(Error::Io(std::io::Error::new(
        ErrorKind::Unsupported,
        "Unsupported message type, only supports Text",
      )))),
      Err(err) => Some(Err(err)),
    }
  }
}

impl SpotifyListener {
  /// Binds to 127.0.0.1:19532
  pub async fn bind_default() -> std::io::Result<Self> {
    Self::bind_local(19532).await
  }

  /// Binds to 127.0.0.1 with a custom port
  pub async fn bind_local(port: u16) -> std::io::Result<Self> {
    Self::bind(format!("127.0.0.1:{port}").parse().unwrap()).await
  }

  /// Binds to the given address, same as calling [TcpListener::bind(addr)]
  pub async fn bind(addr: SocketAddr) -> std::io::Result<Self> {
    let listener = TcpListener::bind(addr).await?;

    Ok(Self { listener })
  }

  /// Establishes a websocket connection to the spotify extension
  pub async fn get_connection(&self) -> Result<SpotifyConnection, Error> {
    let listener = self.listener.accept().await;
    let (stream, _) = listener.map_err(|_| Error::ConnectionClosed)?;
    let ws = accept_async(stream).await?;

    Ok(SpotifyConnection { ws })
  }
}
