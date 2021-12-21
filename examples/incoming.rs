fn main() {
  // Make a WebSocket server for spicetify to talk to
  let server = spotify_info::Listener::new().unwrap();

  for message in server.incoming().unwrap() {
    match message {
      Ok(info) => println!("{:?}", info),
      Err(err) => eprintln!("{:?}", err),
    }
  }
}