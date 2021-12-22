use std::ops::DerefMut;
use std::thread::{sleep, spawn};
use std::time::Duration;
use spotify_info::Handle;

fn main() {
  // Create source handle
  let handle_src = Handle::new();
  // Clone it to be used in another thread
  let handle = handle_src.clone();

  // Create thread that will constantly listen for incoming calls
  let thread = spawn(move || {
    let server = spotify_info::Listener::new_with_handle(handle).unwrap();

    // This will not close instantly,
    for message in server.incoming().unwrap() {
      match message {
        Ok(info) => println!("{:?}", info),
        Err(err) => eprintln!("{:?}", err),
      }
    }
  });

  // This will close the listener after 3 seconds
  spawn(move || {
    sleep(Duration::from_secs(3));

    println!("Closed!");
    handle_src.lock().unwrap().deref_mut().close();
    println!("Closed!");
  }).join().unwrap();

  thread.join().unwrap();
}