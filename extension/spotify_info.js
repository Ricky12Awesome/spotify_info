// NAME: Spotify Info
// AUTHOR: Ricky12Awesome
// DESCRIPTION: Get song information for other apps to use

/// <reference path="globals.d.ts" />

(function SpotifyInfo() {
  if (!Spicetify.CosmosAsync || !Spicetify.Platform) {
    setTimeout(SpotifyInfo, 500);
    return;
  }

  const storage = {
    title: undefined,
    album: undefined,
    artist: undefined,
    cover: undefined,
    background: undefined
  };

  let ws;

  function updateStorage(data) {
    if (!data?.track?.metadata) {
      return;
    }

    const meta = data.track.metadata;

    storage.title = meta.title;
    storage.album = meta.album_title;
    storage.artist = meta.artist_name;

    const cover = meta.image_xlarge_url;

    if (cover?.indexOf("localfile") === -1) {
      storage.cover = "https://i.scdn.co/image/" + cover.substring(cover.lastIndexOf(":") + 1);
    } else {
      storage.cover = undefined;
    }

    storage.background = document.getElementsByClassName("npv-background-image")[0]
      ?.firstElementChild
      ?.firstElementChild
      ?.src ?? undefined;

    console.log(storage);
    ws.send([
        storage.title,
        storage.album,
        storage.artist,
        storage.cover ?? "NONE",
        storage.background ?? "NONE"
      ].join(";")
    )
  }

  Spicetify.CosmosAsync.sub("sp://player/v2/main", updateStorage);

  (function init() {
    console.log("Creating websocket");
    ws = new WebSocket("ws://127.0.0.1:19532");

    ws.onopen = () => {

    };

    ws.onclose = () => {
      setTimeout(init, 2000);
    };
  })();

  window.onbeforeunload = () => {
    ws.onclose = null;
    ws.close();
  }
})();