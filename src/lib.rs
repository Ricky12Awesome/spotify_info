//! Gets metadata from spotify using a
//! [spicetify](https://github.com/khanhas/spicetify-cli)
//! extension using websockets
//!
//! More information can be found on https://github.com/Ricky12Awesome/spotify_info

use std::fmt::{Display, Formatter};
use std::iter::FilterMap;
use std::net::{Incoming, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use tungstenite::{accept, HandshakeError, Message, ServerHandshake, WebSocket};
use tungstenite::handshake::server::NoCallback;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;

//region errors
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Error {
  Handshake(HandshakeError<ServerHandshake<TcpStream, NoCallback>>),
  Tcp(std::io::Error),
  Message(MessageError),
}

#[derive(Debug)]
pub enum MessageError {
  Invalid,
  Other(tungstenite::Error),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Handshake(err) => std::fmt::Display::fmt(err, f),
      _ => std::fmt::Debug::fmt(self, f)
    }
  }
}

impl Display for MessageError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MessageError::Other(err) => std::fmt::Display::fmt(err, f),
      _ => std::fmt::Debug::fmt(self, f)
    }
  }
}

impl std::error::Error for Error {}

impl std::error::Error for MessageError {}

impl From<HandshakeError<ServerHandshake<TcpStream, NoCallback>>> for Error {
  fn from(err: HandshakeError<ServerHandshake<TcpStream, NoCallback>>) -> Self {
    Self::Handshake(err)
  }
}

impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Self::Tcp(err)
  }
}

impl From<MessageError> for Error {
  fn from(err: MessageError) -> Self {
    Self::Message(err)
  }
}

impl From<tungstenite::Error> for Error {
  fn from(err: tungstenite::Error) -> Self {
    Self::Message(MessageError::Other(err))
  }
}

impl From<tungstenite::Error> for MessageError {
  fn from(err: tungstenite::Error) -> Self {
    Self::Other(err)
  }
}
//endregion errors

/// The state of the track weather it's **Playing**, **Paused** or **Stopped**
///
/// Default: Stopped
#[repr(u32)]
#[derive(Debug, Clone)]
pub enum State {
  Playing = 2,
  Paused = 1,
  Stopped = 0,
}

impl State {
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

type FilterMapFn = fn(std::io::Result<TcpStream>) -> Option<WebSocket<TcpStream>>;

/// Listens to incoming messages from spotify to
/// get information about the currently playing track
///
/// **Requires having the spicetify extension installed
/// so it can send messages to this listener
/// which can be found here**
///
/// https://github.com/Ricky12Awesome/spotify_info#installuninstall-spicetify-extension
pub struct Listener {
  listener: TcpListener,
  handle: Arc<Mutex<Handle>>,
}

/// Handles the current connection
#[derive(Default)]
pub struct Handle {
  is_closed: bool,
  current_connection: Option<WebsocketConnection>,
}

impl Handle {
  pub fn new() -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::default()))
  }

  pub fn close(&mut self) {
    self.is_closed = true;

    if let Some(ws) = &mut self.current_connection {
      ws.close().unwrap();
    }
  }
}

impl Listener {
  /// Creates a [TcpListener] bound on `127.0.0.1:19532`
  pub fn new() -> Result<Self, std::io::Error> {
    Self::bind(19532)
  }

  /// Creates a [TcpListener] bound on `127.0.0.1:19532`
  pub fn new_with_handle(handle: Arc<Mutex<Handle>>) -> Result<Self, std::io::Error> {
    Self::bind_with_handle(19532, handle)
  }

  /// Creates a [TcpListener] bound on `127.0.0.1` with the given port
  pub fn bind(port: u16) -> Result<Self, std::io::Error> {
    Self::bind_with_handle(port, Handle::new())
  }

  /// Creates a [TcpListener] bound on `127.0.0.1` with the given port
  pub fn bind_with_handle(port: u16, handle: Arc<Mutex<Handle>>) -> Result<Self, std::io::Error> {
    TcpListener::bind(("127.0.0.1", port)).map(|listener| Self {
      listener,
      handle,
    })
  }

  /// Iterates for every message, waits until it finds a connection, if that connection closes
  /// it will wait for another connection and so on.
  /// to stop this from iterating use [Self::close]
  /// or set the close handle to true
  pub fn incoming(&self) -> Result<ListenerIter, std::io::Error> {
    ListenerIter::from(&self.listener, self.handle.clone())
  }

  /// Closes the listener and will make [Self::incoming] stop iterating
  pub fn close(&self) {}
}

/// Handles the tcp listener and any incoming messages
pub struct ListenerIter<'a> {
  incoming: FilterMap<Incoming<'a>, FilterMapFn>,
  handle: Arc<Mutex<Handle>>,
}

impl<'a> ListenerIter<'a> {
  pub fn from(server: &'a TcpListener, handle: Arc<Mutex<Handle>>) -> Result<Self, std::io::Error> {
    let filter: FilterMapFn = |it| accept(it.ok()?).ok();
    let incoming = server.incoming().filter_map(filter);

    Ok(
      Self {
        incoming,
        handle,
      }
    )
  }
}

impl<'a> Iterator for ListenerIter<'a> {
  type Item = Result<Info, MessageError>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut result = None;

    loop {
      let mut handle = self.handle.lock().unwrap();

      if handle.is_closed {
        return None;
      }

      if handle.current_connection.is_none() {
        let ws = self.incoming.next()?;
        handle.current_connection = Some(WebsocketConnection::new(ws));
        continue;
      }

      if let Some(ws) = &mut handle.current_connection {
        result = ws.next();
      }

      if result.is_none() && !handle.is_closed {
        handle.current_connection = None;
        continue;
      }

      break;
    }

    result
  }
}

/// Handles incoming messages from a websocket
pub struct WebsocketConnection {
  socket: WebSocket<TcpStream>,
}

impl WebsocketConnection {
  pub fn new(socket: WebSocket<TcpStream>) -> Self {
    Self {
      socket,
    }
  }

  pub fn close(&mut self) -> Result<()> {
    self.socket.write_message(Message::Text("Closing".to_string()))?;

    Ok(self.socket.close(Some(CloseFrame {
      code: CloseCode::Away,
      reason: "Server is closing".into(),
    }))?)
  }
}

impl Iterator for WebsocketConnection {
  type Item = Result<Info, MessageError>;

  fn next(&mut self) -> Option<Self::Item> {
    let message = self.socket.read_message().ok()?;

    match message {
      Message::Close(_) => None,
      Message::Text(msg) => {
        let data = msg.split(';').collect::<Vec<_>>();

        if data.len() < 6 {
          return Some(Err(MessageError::Invalid));
        }

        let info = Info {
          state: State::from_u32(data[0].parse().unwrap_or(0)),
          title: data[1].to_string(),
          album: data[2].to_string(),
          artist: vec![data[3].to_string()],
          cover_url: Some(data[4].to_string()),
          background_url: Some(data[5].to_string()),
        };

        Some(Ok(info))
      }
      _ => Some(Err(MessageError::Invalid))
    }
  }
}