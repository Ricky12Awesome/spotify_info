#!/usr/bin/env bash

if [ -d ~/.config/spicetify/Extensions/ ]; then
  if [ -f ~/.config/spicetify/Extensions/spotify_info.js ]; then
    spicetify config extensions spotify_info.js- && spicetify apply
    echo "Extension has been successfully been uninstalled."
    rm ~/.config/spicetify/Extensions/spotify_info.js
  else
    wget https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/spotify_info.js -P ~/.config/spicetify/Extensions/
    spicetify config extensions spotify_info.js && spicetify apply
    echo "Extension has been successfully been installed."
  fi
else
  echo "Extensions directory was not found, do you have Spicetify installed? install it here https://spicetify.app/docs/getting-started/installation"
fi