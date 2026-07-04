use std::collections::BTreeMap;
use crate::state::Lyrics;

pub async fn fetch_lyrics(title: &String, artist: &String) -> Result<Lyrics, Box<dyn std::error::Error>> {
    let clean_title = title
        .trim()
        .trim_end_matches(".mp3")
        .trim_end_matches(".flac")
        .trim_end_matches(".wav")
        .trim_end_matches(".m4a")
        .trim_end_matches(".ogg")
        .trim()
        .to_string();

    let query = format!("{} - {}", artist, clean_title);
    let url = format!(
        "https://lrclib.net/api/search?q={}",
        urlencoding::encode(&query)
    );

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    let json: serde_json::Value = response.json().await?;

    let Some(results) = json.as_array() else {
        return Err("Unexpected API response: not an array".into());
    };

    if results.is_empty() {
        return Err("No lyrics found".into());
    }

    let lower_title = clean_title.to_lowercase();
    let lower_artist = artist.to_lowercase();

    let mut best_match = None;
    let mut best_score = 0;

    for item in results {
        let track_name = item["trackName"].as_str().unwrap_or("").to_lowercase();
        let artist_name = item["artistName"].as_str().unwrap_or("").to_lowercase();

        let mut score = 0;
        if track_name.contains(&lower_title) {
            score += 2;
        }
        if artist_name.contains(&lower_artist) {
            score += 1;
        }

        if score > best_score {
            best_score = score;
            best_match = Some(item);
        }
    }

    let Some(best) = best_match else {
        return Err("No suitable lyrics match found".into());
    };

    Ok(Lyrics {
        plain: best["plainLyrics"].as_str().map(|s| s.trim_start_matches('\u{feff}').to_string()),
        synced: Some(parse_lyrics_to_map(best["syncedLyrics"].as_str().unwrap_or(""))),
    })
}

pub fn parse_lyrics_to_map(lyrics_string: &str) -> BTreeMap<u32, String> {
    let mut lyrics_map = BTreeMap::new();

    let clean_lyrics = lyrics_string.trim_start_matches('\u{feff}');

    for line in clean_lyrics.lines() {
        if let Some(end_of_timestamp) = line.find(']') {
            let timestamp_str = &line[1..end_of_timestamp];
            let lyric_str = line[end_of_timestamp + 1..].trim();

            if let Ok(timestamp_ms) = parse_timestamp_to_ms(timestamp_str) {
                lyrics_map.insert(timestamp_ms, lyric_str.to_string());
            }
        }
    }

    lyrics_map
}

fn parse_timestamp_to_ms(timestamp_str: &str) -> Result<u32, String> {
    let parts: Vec<&str> = timestamp_str.split(&[':', '.']).collect();

    if parts.len() != 3 {
        return Err("Invalid timestamp format".to_string());
    }

    let minutes = parts[0].parse::<u32>().unwrap_or(0);
    let seconds = parts[1].parse::<u32>().unwrap_or(0);
    let centiseconds = parts[2].parse::<u32>().unwrap_or(0);

    Ok((minutes * 60_000) + (seconds * 1_000) + (centiseconds * 10))
}
