use spotify_info::{SpotifyEvent, TrackListener};

#[tokio::main]
async fn main() {
  // Create listener
  let listener = TrackListener::bind_default().await.unwrap();

  // Listen for incoming connections, if spotify closes, the loop keeps listening
  while let Ok(mut connection) = listener.get_connection().await {
    while let Some(Ok(event)) = connection.next().await {
      match event {
        // Gets called when user changed track
        SpotifyEvent::TrackChanged(info) => println!("Changed track to {}", info.title),
        // Gets called when user changes state (if song is playing, paused or stopped)
        SpotifyEvent::StateChanged(state) => println!("Changed state to {}", state),
        // Gets called on a set interval, wont get called if player is paused or stopped,
        // Value is a percentage of the position between 0 and 1
        SpotifyEvent::ProgressChanged(time) => println!("Changed progress to {}", time)
      }
    }
  }
}