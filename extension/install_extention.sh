if [ -d ~/.config/spicetify/Extensions/ ]; then
  if [ -f ~/.config/spicetify/Extensions/spotify_info.js ]; then
    wget https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/spotify_info.js -P ~/.config/spicetify/Extensions/
    spicetify config extensions spotify_info.js && spicetify apply
    echo "Extension has been successfully been installed."
  else
    rm ~/.config/spicetify/Extensions/spotify_info.js
    spicetify config extensions -spotify_info.js && spicetify apply
    echo "Extension has been successfully been uninstalled."
  fi
else
  echo "Extensions directory was not found, do you have Spicetify installed? install it here https://spicetify.app/docs/getting-started/installation"
fi