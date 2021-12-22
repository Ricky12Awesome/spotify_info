// NAME: Spotify Info
// AUTHOR: Ricky12Awesome
// DESCRIPTION: Get song information for other apps to use

/// <reference path="globals.d.ts" />

function SpotifyInfo() {
  if (!Spicetify.CosmosAsync || !Spicetify.Platform) {
    setTimeout(SpotifyInfo, 500);
    return;
  }

  let ws;
  let storage = {
    state: undefined,
    title: undefined,
    album: undefined,
    artist: undefined,
    cover: undefined,
    background: undefined
  };

  async function updateStorage(data) {
    if (!data?.track?.metadata) {
      return;
    }

    const meta = data.track.metadata;
    const local = {
      state: undefined,
      title: undefined,
      album: undefined,
      artist: undefined,
      cover: undefined,
      background: undefined
    };

    // doing local === storage or local == storage doesn't work,
    // so I needed to do this
    function storage_eq() {
      return local.state === storage.state &&
        local.title === storage.title &&
        local.album === storage.album &&
        local.artist === storage.artist &&
        local.cover === storage.cover &&
        local.background === storage.background;
    }

    local.state = data.is_paused ? 1 : 2;
    local.title = meta.title;
    local.album = meta.album_title;
    local.artist = meta.artist_name;

    const cover = meta.image_xlarge_url;

    local.cover = cover?.indexOf("localfile") === -1 ? "https://i.scdn.co/image/" + cover.substring(cover.lastIndexOf(":") + 1) : undefined;

    try {
      const uriBase62 = meta.artist_uri.substring(meta.artist_uri.lastIndexOf(":") + 1);
      const artistInfo = await Spicetify.CosmosAsync.get(`hm://artist/v1/${uriBase62}/desktop?format=json`);
      local.background = artistInfo.header_image.image;
    } catch (e) {
      local.background = undefined;
    }

    // so it doesn't spam multiple messages
    if (!storage_eq()) {
      storage = local;

      ws.send([
          local.state ?? 0,
          local.title ?? "NONE",
          local.album ?? "NONE",
          local.artist ?? "NONE",
          local.cover ?? "NONE",
          local.background ?? "NONE"
        ].join(";")
      );
    }
  }

  Spicetify.CosmosAsync.sub("sp://player/v2/main", updateStorage);

  function init() {
    ws = new WebSocket("ws://127.0.0.1:19532");

    ws.onclose = () => {
      setTimeout(init, 1000);
    };
  }

  init()

  window.onbeforeunload = () => {
    ws.onclose = null;
    ws.close();
  }
}

SpotifyInfo()