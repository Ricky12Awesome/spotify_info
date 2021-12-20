// NAME: Spotify Info
// AUTHOR: Ricky12Awesome
// DESCRIPTION: Get song information for other apps to use

/// <reference path="globals.d.ts" />

(function SpotifyInfo() {
  if (!Spicetify.CosmosAsync || !Spicetify.Platform) {
    setTimeout(SpotifyInfo, 500)
    return
  }

  const storage = {}

  function updateStorage(data) {
    if (!data?.track?.metadata) {
      return
    }

    const meta = data.track.metadata

    storage.title = meta.title
    storage.album = meta.album_title
    storage.artist = meta.artist_name

    const cover = meta.image_xlarge_url

    if (cover?.indexOf("localfile") === -1) {
      storage.cover = "https://i.scdn.co/image/" + cover.substring(cover.lastIndexOf(":") + 1)
    } else {
      storage.cover = undefined
    }

    storage.background = document.getElementsByClassName("npv-background-image")[0]
      ?.firstElementChild
      ?.firstElementChild
      ?.src ?? undefined


    console.log(storage)
    console.log(data)
  }

  Spicetify.CosmosAsync.sub("sp://player/v2/main", updateStorage)


})()