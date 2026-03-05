# Research: Kodi JSON-RPC Formats

This document provides the exact JSON formats required to mock the Kodi RPC API for `tv_mode_web` testing.

## Common Wrapper
All Kodi RPC responses follow this structure:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": { ... }
}
```

## 1. VideoLibrary.GetTVShows
**Kodi Response**:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "tvshows": [
      { "tvshowid": 1, "title": "The Office" },
      { "tvshowid": 2, "title": "Breaking Bad" }
    ],
    "limits": { "end": 2, "start": 0, "total": 2 }
  }
}
```

## 2. VideoLibrary.GetEpisodes
**Kodi Response**:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "episodes": [
      { "episodeid": 101, "title": "Pilot", "season": 1, "episode": 1 },
      { "episodeid": 102, "title": "Diversity Day", "season": 1, "episode": 2 }
    ],
    "limits": { "end": 2, "start": 0, "total": 12 }
  }
}
```

## 3. VideoLibrary.GetEpisodeDetails
**Kodi Response**:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": {
    "episodedetails": {
      "file": "/media/tv/The Office/S01E01.mkv",
      "label": "Pilot"
    }
  }
}
```

## 4. Player.GetActivePlayers
**Kodi Response (Active)**:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": [
    { "playerid": 1, "type": "video" }
  ]
}
```
**Kodi Response (Inactive)**:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": []
}
```

## 5. Player.Open / Player.Stop
**Kodi Response**:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "result": "OK"
}
```
