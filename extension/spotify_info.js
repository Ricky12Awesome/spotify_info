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
  let ws_connected;
  let ws_data;
  let storage = {
    uid: undefined,
    uri: undefined,
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

    console.log(data);

    const meta = data.track.metadata;
    const local = {
      uid: undefined,
      uri: undefined,
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

    local.uid = data.track.uid;
    local.uri = data.track.uri;
    local.state = data.is_paused ? 1 : 2;
    local.title = meta.title;
    local.album = meta.album_title;
    local.artist = meta.artist_name;

    const cover = meta.image_xlarge_url;

    local.cover = cover?.indexOf("localfile") === -1 ? "https://i.scdn.co/image/" + cover.substring(cover.lastIndexOf(":") + 1) : undefined;

    try {
      const res = await Spicetify.CosmosAsync.get(
        `https://api-partner.spotify.com/pathfinder/v1/query?operationName=queryArtistOverview&variables=%7B%22uri%22%3A%22${meta.artist_uri}%22%7D&extensions=%7B%22persistedQuery%22%3A%7B%22version%22%3A1%2C%22sha256Hash%22%3A%22433e28d1e949372d3ca3aa6c47975cff428b5dc37b12f5325d9213accadf770a%22%7D%7D`
      )

      local.background = res.data.artist.visuals.headerImage.sources[0].url
    } catch (e) {
      local.background = undefined;
    }

    // so it doesn't spam multiple messages
    if (!storage_eq()) {
      storage = local;
      ws_data = [
        local.uid ?? "NONE",
        local.uri ?? "NONE",
        local.state ?? 0,
        local.title ?? "NONE",
        local.album ?? "NONE",
        local.artist ?? "NONE",
        local.cover ?? "NONE",
        local.background ?? "NONE"
      ].join(";");

      if (ws_connected) {
        ws.send(ws_data);
      }
    }
  }

  Spicetify.CosmosAsync.sub("sp://player/v2/main", updateStorage);

  function init() {
    ws_connected = false;
    ws = new WebSocket("ws://127.0.0.1:19532");

    ws.onopen = () => {
      ws_connected = true;
      if (ws_data) ws.send(ws_data);
    };

    ws.onclose = () => setTimeout(init, 1000);
  }

  init()

  window.onbeforeunload = () => {
    ws_connected = false;
    ws.onclose = null;
    ws.close();
  }
}

SpotifyInfo()