use std::time::Duration;

use tokio::time;

use crate::{
    lyrics::fetch_lyrics,
    player::watch_playerctl,
    state::{AppState, LoadingStatus},
};

pub mod lyrics;
pub mod player;
pub mod state;

#[tokio::main]
async fn main() {
    let mut app_state = AppState {
        song_info: None,
        lyrics: None,
        loading_status: LoadingStatus::Idle,
        quit: false,
    };

    let mut last_printed = None;

    while !app_state.quit {
        time::sleep(Duration::from_millis(200)).await;

        if let Some((new_song_info, position)) = watch_playerctl() {
            if app_state.song_info != Some(new_song_info.clone()) {
                app_state.song_info = Some(new_song_info.clone());
                app_state.loading_status = LoadingStatus::Loading;
                println!(
                    "  Loading new Song {} - {}",
                    new_song_info.title, new_song_info.artist
                );

                let song_info = new_song_info.clone();
                let lyrics = fetch_lyrics(&song_info.title, &song_info.artist).await;
                match lyrics {
                    Ok(lyrics) => {
                        app_state.lyrics = Some(lyrics);
                        app_state.loading_status = LoadingStatus::Loaded;
                        println!(
                            "   Loaded Lyrics for {} - {}",
                            new_song_info.title, new_song_info.artist
                        );
                    }
                    Err(err) => {
                        app_state.loading_status = LoadingStatus::Error(err.to_string());
                        println!("  Error fetching lyrics: {}", err);
                        app_state.lyrics = None;
                    }
                }

                last_printed = None;
            }

            if let Some(lyrics) = &app_state.lyrics {
                if let Some(synced) = &lyrics.synced {
                    if last_printed.is_none() {
                        last_printed = Some(0);
                    }

                    if let Some((ts, line)) = synced.range(..=position).last() {
                        if Some(*ts) != last_printed {
                            println!("{line}");
                            last_printed = Some(*ts);
                        }
                    }
                } else {
                    println!("No Synced Lyrics");
                }
            }
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
    }
}
