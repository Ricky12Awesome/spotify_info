//! Gets metadata from spotify using a
//! [spicetify](https://github.com/khanhas/spicetify-cli)
//! extension using websockets
//!
//! More information can be found on https://github.com/Ricky12Awesome/spotify_info

use std::fmt::{Display, Formatter};
use std::io::Read;
use std::iter::FilterMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use tungstenite::{accept, HandshakeError, Message, ServerHandshake, WebSocket};
use tungstenite::handshake::server::NoCallback;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;

//region errors
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type HandshakeError2 = HandshakeError<ServerHandshake<TcpStream, NoCallback>>;

#[derive(Debug)]
pub enum Error {
  Handshake(HandshakeError2),
  Tcp(std::io::Error),
  Message(MessageError),
  ClosingError,
}

#[derive(Debug)]
pub enum MessageError {
  Invalid,
  Other(tungstenite::Error),
}

impl<'a> Display for Error {
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

impl From<HandshakeError2> for Error {
  fn from(err: HandshakeError2) -> Self {
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

/// Server token for mio events
const SERVER: Token = Token(0);
/// Client token for mio events
const CLIENT: Token = Token(1);

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
  poll: Poll,
  events: Events,
  handle: Handle,
}

/// Handles the current connection
#[derive(Default, Clone)]
pub struct Handle {
  is_closed: Arc<AtomicBool>,
}

impl Handle {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn close(&self) {
    self.is_closed.store(true, Ordering::SeqCst);
  }
}

impl Listener {
  /// Creates a [TcpListener] bound on `127.0.0.1:19532`
  pub fn new() -> Result<Self, std::io::Error> {
    Self::bind(19532)
  }

  /// Creates a [TcpListener] bound on `127.0.0.1:19532`
  pub fn new_with_handle(handle: Handle) -> Result<Self, std::io::Error> {
    Self::bind_with_handle(19532, handle)
  }

  /// Creates a [TcpListener] bound on `127.0.0.1` with the given port
  pub fn bind(port: u16) -> Result<Self, std::io::Error> {
    Self::bind_with_handle(port, Handle::new())
  }

  /// Creates a [TcpListener] bound on `127.0.0.1` with the given port
  pub fn bind_with_handle(port: u16, handle: Handle) -> Result<Self, std::io::Error> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port).parse().unwrap())?;
    let poll = Poll::new()?;
    let events = Events::with_capacity(128);

    let mut slf = Self { poll, events, listener, handle };

    slf.setup()?;

    Ok(slf)
  }

  /// Set up polling events for server
  pub fn setup(&mut self) -> Result<(), std::io::Error> {
    self.poll.registry().register(&mut self.listener, SERVER, Interest::READABLE)
  }

  /// Iterates for every message, waits until it finds a connection, if that connection closes
  /// it will wait for another connection and so on.
  /// to stop this from iterating use [Self::close]
  /// or set the close handle to true
  pub fn incoming(&mut self) -> Result<ListenerIter, std::io::Error> {
    ListenerIter::from(self)
  }

  /// Closes the listener and will make [Self::incoming] stop iterating
  pub fn close(&self) {}
}

/// Handles the tcp listener and any incoming messages
pub struct ListenerIter<'a> {
  listener: &'a mut Listener,
  stream: Option<TcpStream>,
  connection: Option<WebsocketConnection<'a>>,
}

impl<'a> ListenerIter<'a> {
  pub fn from(listener: &'a mut Listener) -> Result<Self, std::io::Error> {
    Ok(Self { listener, stream: None, connection: None })
  }
}

impl<'a> Iterator for ListenerIter<'a> {
  type Item = Result<Info, MessageError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.listener.poll.poll(&mut self.listener.events, None).ok()?;

    let should_close = || self.listener.handle.is_closed.load(Ordering::SeqCst);

    for event in &self.listener.events {
      match event.token() {
        SERVER => {
          loop {
            if should_close() {
              break;
            }

            let stream = self.listener.listener.accept().ok();

            if stream.is_none() {
              continue;
            }

            let stream = stream.unwrap().0;

            self.stream = Some(stream);

            break;
          }


          if let Some(stream) = &mut self.stream {
            self.listener.poll.registry()
              .register(stream, CLIENT, Interest::READABLE | Interest::WRITABLE).ok()?;

            let connection = WebsocketConnection::<'a>::new(stream as &_);
            self.connection = Some(connection);
          }
        }

        CLIENT => {
          let mut result = None;

          loop {
            if !event.is_readable() || event.is_read_closed() || should_close() {
              break;
            }

            if let Some(connection) = &mut self.connection {
              result = connection.next();
            }

            if result.is_none() {
              continue;
            }

            return result;
          }
        }
        _ => unreachable!("Other tokens was somehow registered")
      }
    }

    None
  }
}

/// Handles incoming messages from a websocket
pub struct WebsocketConnection<'a> {
  stream: &'a TcpStream,
  socket: Option<WebSocket<&'a TcpStream>>,
}

impl<'a> WebsocketConnection<'a> {
  pub fn new(stream: &'a TcpStream) -> Self {
    Self {
      stream,
      socket: None,
    }
  }

  fn init(&mut self) {
    loop {
      if self.socket.is_none() {
        self.socket = accept(self.stream).ok();
        continue;
      }

      break;
    }
  }

  pub fn close(&mut self) -> Result<()> {
    if self.socket.is_none() {
      return Ok(());
    }

    Ok(self.socket.as_mut().unwrap().close(Some(CloseFrame {
      code: CloseCode::Away,
      reason: "Server is closing".into(),
    }))?)
  }
}

impl<'a> Iterator for WebsocketConnection<'a> {
  type Item = Result<Info, MessageError>;

  fn next(&mut self) -> Option<Self::Item> {
    self.init();
    let message = self.socket.as_mut().unwrap().read_message().ok()?;

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