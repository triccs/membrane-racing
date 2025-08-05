// Track Manager Contract Architecture

## Purpose

The TrackManager stores and serves race tracks. It defines the spatial layout and terrain properties of race environments.

## Responsibilities

* Store 2D grid-based tracks with per-tile metadata
* Support creation and preview of tracks
* Provide sensor and distance data for simulation

## Track Layout Format

```rust
Track {
  id: u64,
  name: String,
  width: u8,
  height: u8,
  layout: Vec<Vec<TrackTile>> // 2D (y,x)
}

TrackTile {
  tile_type: TileType,
  progress_towards_finish: u16,
}

TileType = Normal | Slow | Boost | Wall | Stick | Finish
```

## Core Messages

### Execute

* `AddTrack { track }`

### Query

* `GetTrack { track_id }` → full track layout
* `ListTracks {}` → available track IDs

## Notes

* The contract enforces distance-from-finish preprocessing on upload.
* Each tile has `(x,y)` coordinates implicitly defined by layout index.
* Used only for read-only simulation and training context.
