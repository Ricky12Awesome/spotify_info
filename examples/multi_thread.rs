use std::time::Duration;

use spotify_info::{TrackHandle, TrackListener};

#[tokio::main]
async fn main() {
  // Create handle to be used for the main thread
  // that will constantly listen for incoming calls
  let main_handle = TrackHandle::default();
  // clones that handle to be used on another thread
  let handle = main_handle.clone();

  // Create thread that will constantly listen for incoming calls
  let main = tokio::spawn(async {
    let listener = TrackListener::bind_default().await.unwrap();
    listener.listen(main_handle).await;
  });

  // Creates a reading thread, since reading currently
  // just gets the latest TrackInfo and wll not wait until
  // the next message is sent
  let reading = tokio::spawn(async move {
    loop {
      let read = handle.read();
      println!("{:?}", read);
      tokio::time::sleep(Duration::from_millis(100)).await;
    }
  });

  // Waits 5 seconds to stop the threads
  tokio::time::sleep(Duration::from_secs(5)).await;
  // You would need to abort main thread to stop it
  main.abort();
  // Since reading is an infinite loop, stop that as well
  reading.abort();
}