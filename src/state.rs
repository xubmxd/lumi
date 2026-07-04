use std::collections::{BTreeMap};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub song_info: Option<SongInfo>,
    pub lyrics: Option<Lyrics>,
    pub loading_status: LoadingStatus,
    pub quit: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Lyrics {
    pub plain: Option<String>,
    pub synced: Option<BTreeMap<u32, String>>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadingStatus {
    Idle,
    Loading,
    Loaded,
    Error(String),
}