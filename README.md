[![crates.io](https://img.shields.io/crates/v/spotify_info.svg)](https://crates.io/crates/spotify_info)
[![docs.rs](https://docs.rs/spotify_info/badge.svg)](https://crates.io/crates/spotify_info)
[![license](https://img.shields.io/github/license/Ricky12Awesome/spotify_info)](https://github.com/Ricky12Awesome/spotify_info/blob/main/LICENSE)

# Spotify Info
Gets metadata from spotify using a 
[spicetify](https://github.com/khanhas/spicetify-cli) 
extension using websockets

I made this mainly for my audio visualizer to have song info display along with it
since Windows doesn't offer anyway to do this, on Linux I could use MPRIS, for macOS no idea I don't use it

This will work for all desktop platforms (Windows, Linux, macOS)

## API Usage

Examples can be found in the [examples](https://github.com/Ricky12Awesome/spotify_info/tree/main/examples) directory

```rust
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
```

## Use API
#### Add this to your Cargo.toml dependencies
```toml
spotify_info = "0.5"
```

## Plans
- [ ] Improve Documentation
- [ ] Make instructions easy to understand for regular users
- [ ] When the track was created
- [ ] What playlist the track is in

## Install/Uninstall Spicetify Extension

### Auto Install
Run script to install, uninstall by running it again
#### Windows
Open `PowerShell` (Win + S) type powershell and press enter and run this command
```sh
Invoke-WebRequest -UseBasicParsing "https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/install_extension.ps1" | Invoke-Expression
```
#### Linux / macOS
Open `Terminal` (usually Ctrl + Alt + T on linux) and run this command
```sh
curl https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/install_extension.sh | sh
```

### Manual
You can get the extension from 
[here](https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/spotify_info.js)
(right-click -> save as)

Place that file 
to `%userprofile%\.spicetify\Extensions\` on Windows 
or `~/.config/spicetify/Extensions` on Linux / macOS 

##### Install
Run command
`spicetify config extensions spotify_info.js && spicetify apply` 
from the terminal to install the plugin

##### Uninstall
Run command
`spicetify config extensions spotify_info.js- && spicetify apply`
from the terminal to uninstall the plugin

More details about install extensions https://spicetify.app/docs/getting-started/extensions

