use std::fmt::{Display, Formatter};
use std::iter::FilterMap;
use std::net::{Incoming, TcpListener, TcpStream};

use tungstenite::{accept, HandshakeError, Message, ServerHandshake, WebSocket};
use tungstenite::handshake::server::NoCallback;

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

type FilterMapFn = fn(std::io::Result<TcpStream>) -> Option<WebSocket<TcpStream>>;

pub struct TcpConnection {
  server: TcpListener,
  should_close: bool,
}

impl TcpConnection {
  pub fn new() -> Result<Self, std::io::Error> {
    TcpListener::bind("127.0.0.1:19532").map(|server| Self {
      server,
      should_close: false,
    })
  }

  pub fn incoming(&self) -> Result<TcpConnectionIter, std::io::Error> {
    TcpConnectionIter::from(&self.server, &self.should_close)
  }

  pub fn close(&mut self) {
    self.should_close = true;
  }
}

/// Handles the tcp listener connection and incoming connections
pub struct TcpConnectionIter<'a> {
  incoming: FilterMap<Incoming<'a>, FilterMapFn>,
  should_close: &'a bool,
  current_connection: Option<WebsocketConnection>,
}

impl<'a> TcpConnectionIter<'a> {
  pub fn from(server: &'a TcpListener, should_close: &'a bool) -> Result<Self, std::io::Error> {
    let filter: FilterMapFn = |it| accept(it.ok()?).ok();
    let incoming = server.incoming().filter_map(filter);

    Ok(
      Self {
        incoming,
        should_close,
        current_connection: None,
      }
    )
  }
}

impl<'a> Iterator for TcpConnectionIter<'a> {
  type Item = Result<Info, MessageError>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut result = None;

    if *self.should_close {
      self.current_connection = None;
      return None;
    }

    if self.current_connection.is_none() {
      let ws = self.incoming.next()?;
      self.current_connection = Some(WebsocketConnection::new(ws));
    }

    if let Some(ws) = &mut self.current_connection {
      result = ws.next();
    }

    if result.is_none() {
      self.current_connection = None;
      return self.next();
    }

    result
  }
}

/// Handles incoming messages from a websocket
pub struct WebsocketConnection {
  should_close: bool,
  socket: WebSocket<TcpStream>,
}

impl WebsocketConnection {
  pub fn new(socket: WebSocket<TcpStream>) -> Self {
    Self {
      should_close: false,
      socket,
    }
  }

  pub fn close(&mut self) {
    self.should_close = true;
  }
}

impl Iterator for WebsocketConnection {
  type Item = Result<Info, MessageError>;

  fn next(&mut self) -> Option<Self::Item> {
    let message = self.socket.read_message().ok()?;

    if self.should_close || message.is_close() {
      None
    } else {
      Some(handle_message(message))
    }
  }
}

pub fn handle_message(msg: Message) -> Result<Info, MessageError> {
  if let Message::Text(msg) = msg {
    let data = msg.split(';').collect::<Vec<_>>();

    if data.len() < 6 {
      return Err(MessageError::Invalid);
    }

    let info = Info {
      state: State::from_u32(data[0].parse().unwrap_or(0)),
      title: data[1].to_string(),
      album: data[2].to_string(),
      artist: vec![data[3].to_string()],
      cover_url: Some(data[4].to_string()),
      background_url: Some(data[5].to_string()),
    };

    Ok(info)
  } else {
    Err(MessageError::Invalid)
  }
}

pub fn websocket() -> Result<()> {
  let connection = TcpConnection::new()?;

  for message in connection.incoming()? {
    match message {
      Ok(info) => println!("{:?}", info),
      Err(err) => eprintln!("{:?}", err),
    }
  }

  Ok(())
}
