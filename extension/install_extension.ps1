$URL="https://raw.githubusercontent.com/Ricky12Awesome/spotify_info/main/extension/spotify_info.js"
$FOLDER="$env:userprofile\.spicetify\Extensions\"

if (Test-Path -Path $FOLDER) {
    if (Test-Path -Path "$FOLDER/spotify_info.js") {
        spicetify config extensions spotify_info.js-
        spicetify apply
        Remove-Item -Path "$FOLDER/spotify_info.js"
        Write-Output "Extension has been successfully been uninstalled."
    } else {
        Invoke-WebRequest $URL -OutFile "$FOLDER/spotify_info.js"
        spicetify config extensions spotify_info.js
        spicetify apply
        Write-Output "Extension has been successfully been installed."
    }
} else {
    Write-Output "Extensions directory was not found, do you have Spicetify installed? install it here https://spicetify.app/docs/getting-started/installation"
}