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

## Install/Uninstall Spicetify Extension

### Auto Install
Run script to install, uninstall by running it again
#### Windows (Not made yet)
```sh
not made yet, coming soon once I get on windows
```
#### Linux / macOS
```sh
curl https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/install_extention.sh | sh
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

## Use API
### Add this to your Cargo.toml dependencies
```toml
spotify_info = "0.1.0"
```

Examples can be found in the [examples](https://github.com/Ricky12Awesome/spotify_info/tree/main/examples) directory 