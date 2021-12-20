if [ -d ~/.config/spicetify/Extensions/ ]; then
  wget https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/spotify_info.js -P ~/.config/spicetify/Extensions/
  spicetify config extensions spotify_info.js && spicetify apply
else
  echo "Extensions directory was not found, do you have Spicetify installed? install it here https://spicetify.app/docs/getting-started/installation"
fi