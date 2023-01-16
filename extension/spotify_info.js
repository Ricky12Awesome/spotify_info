// NAME: Spotify Info
// AUTHOR: Ricky12Awesome
// DESCRIPTION: Get song information for other apps to use

/// <reference path="globals.d.ts" />

// ----- SETTINGS -----

// Change this if you want to use a custom port
// make sure to also change it on the other end
//
// default: 19532
const port = 19532;

// How often should this check for connections?
//
// default: 1000
const checkConnectionInterval = 1000;

// How often should this send position updates in milliseconds
//
// can be changed by the other end
//
// default: 1000
let progressUpdateInterval = 1000;

// --------------------

function SpotifyInfo() {
  if (!Spicetify.CosmosAsync || !Spicetify.Platform) {
    setTimeout(SpotifyInfo, 500);
    return;
  }

  let ws;
  let ws_connected;
  let storage = {
    uid: undefined,
    uri: undefined,
    state: undefined,
    duration: undefined,
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
    const local = {};

    local.uid = data.track.uid;
    local.uri = data.track.uri;
    local.state = data.is_paused ? 1 : 2;
    local.duration = Number.parseInt(meta.duration);
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
    if (local.uid !== storage.uid) {
      storage = local;
      
      if (ws_connected) {
        ws.send(JSON.stringify({
          "TrackChanged": storage
        }));
      }
    } else if (local.state !== storage.state) {
      storage.state = local.state;

      if (ws_connected) {
        ws.send(JSON.stringify({
          "StateChanged": local.state ?? 0
        }));

        if (storage.state !== 2) {
          ws.send(JSON.stringify({
            "ProgressChanged": Spicetify.Player.getProgressPercent()
          }));
        }
      }
    }
  }

  Spicetify.CosmosAsync.sub("sp://player/v2/main", updateStorage);

  function init() {
    ws_connected = false;
    ws = new WebSocket(`ws://127.0.0.1:${port}`);


    ws.onopen = () => {
      ws_connected = true;

      if (storage) {
        ws.send(JSON.stringify({
          "TrackChanged": storage
        }));
      }
    };

    ws.onclose = () => {
      ws_connected = false;
      setTimeout(init, checkConnectionInterval);
    };


    ws.onmessage = (message) => {
      let data = JSON.parse(message.data);
      let interval = data["ProgressInterval"];

      if (interval) {
        let n = Number.parseInt(interval);
        
        if (!isNaN(n)) {
          progressUpdateInterval = n;
        }
      }
    };
  }

  init();

  const progressInterval = () => {
    if (ws_connected && storage.state === 2) {
      ws.send(JSON.stringify({
        "ProgressChanged": Spicetify.Player.getProgressPercent()
      }));
    }

    setTimeout(progressInterval, progressUpdateInterval)
  };

  setTimeout(progressInterval, progressUpdateInterval);

  window.onbeforeunload = () => {
    ws_connected = false;
    ws.onclose = null;
    ws.close();
  }
}

SpotifyInfo()