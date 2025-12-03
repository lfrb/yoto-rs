#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Device {
    #[serde(rename = "deviceId")]
    pub id: String,
    pub name: String,
    pub description: String,
    online: bool,
    release_channel: Option<String>,
    device_type: Option<String>,
    family: Option<String>,
    group: Option<String>,
}

/*#[derive(Deserialize, Serialize)]
pub enum CardType {
    None = 0,
    Physical = 1,
    Remote = 2,
}
pub enum DayMode {
    Unknown = -1,
    Night = 0,
    Day = 1,
}
pub enum NightlightMode {
    Off,
    On(String),
}
pub enum PowerSource {
    Battery = 0,
    V2Dock = 1,
    USB-C = 2,
    QiDock = 3,
}
*/

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeviceStatus {
    #[serde(rename = "deviceId")]
    id: String,
    uptime: u64,
    utc_offset_seconds: u32,
    utc_time: String,
    updated_at: DateTime<Utc>,

    /* Mode */
    active_card: String,
    card_insertion_state: u32,
    day_mode: u32,
    nightlight_mode: String,

    /* Network */
    #[serde(rename = "isBackgroundDownloadActive")]
    active_download: bool,
    #[serde(rename = "averageDownloadSpeedBytesSecond")]
    download_speed: u64,
    #[serde(rename = "isOnline")]
    online: bool,
    network_ssid: String,
    wifi_strength: u32,

    /* Power */
    #[serde(rename = "isCharging")]
    charging: bool,
    #[serde(rename = "batteryLevelPercentage")]
    battery_level: u32,
    power_source: u32,

    /* Audio */
    #[serde(rename = "userVolumePercentage")]
    user_volume: u32,
    #[serde(rename = "systemVolumePercentage")]
    system_volume: u32,
    is_audio_device_connected: bool,
    is_bluetooth_audio_connected: bool,

    /* Storage */
    #[serde(rename = "freeDiskSpaceBytes")]
    free_disk_space: u64,
    #[serde(rename = "totalDiskSpaceBytes")]
    total_disk_space: u64,

    /* Sensors */
    #[serde(rename = "ambientLightSensorReading")]
    ambient_light: Option<String>,
    #[serde(rename = "temperatureCelsius")]
    temperature: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Image {
    #[serde(rename = "imageId")]
    id: String,
    #[serde(rename = "eTag")]
    etag: String,
    last_modified: DateTime<Utc>,
    size: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DisplayIcon {
    #[serde(rename = "displayIconId")]
    id: String,
    media_id: String,
    public: bool,
    url: String,
    created_at: DateTime<Utc>,
    user_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Audio,
    Stream,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFormat {
    Mp3,
    Aac,
    Opus,
    Ogg,
    #[serde(untagged)]
    Unknown(String),
}

impl MediaFormat {
    pub fn from_ext(ext: &str) -> Result<MediaFormat, String> {
        match ext {
            "mp3" => Ok(MediaFormat::Mp3),
            "aac" => Ok(MediaFormat::Aac),
            "ogg" => Ok(MediaFormat::Ogg),
            "opus" => Ok(MediaFormat::Opus),
            _ => Err("Unsupported file extension".to_string()),
        }
    }

    pub fn content_type(&self) -> String {
        match self {
            MediaFormat::Mp3 => String::from("audio/mpeg"),
            MediaFormat::Aac => String::from("audio/aac"),
            MediaFormat::Ogg => String::from("audio/ogg"),
            MediaFormat::Opus => String::from("audio/opus"),
            MediaFormat::Unknown(f) => format!("audio/{}", f),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Mono,
    Stereo,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackType {
    #[default]
    Linear,
    Interactive,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub title: String,
    slug: Option<String>,
    sort_key: Option<String>,
    availability: String,
    pub card_id: String,
    content: CardContent,
    created_at: String,
    deleted: bool,
    metadata: CardMetadata,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct CardContent {
    version: String,
    chapters: Vec<Chapter>,
    config: ContentConfig,
    playback_type: PlaybackType,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentConfig {
    autoadvance: Option<bool>,
    resume_timeout: Option<u64>,
    system_activity: Option<bool>,
    track_number_overlay_timeout: Option<u64>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct CardMetadata {
    author: String,
    category: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    key: String,
    title: String,
    overlay_label: Option<String>,
    overlay_label_override: Option<String>,
    tracks: Vec<Track>,
    default_track_display: Option<String>,
    default_track_ambient: Option<String>,
    duration: Option<u64>,
    file_size: Option<u64>,
    display: Option<Icon>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    title: String,
    track_url: String,
    key: String,
    uid: Option<String>,
    #[serde(rename = "type")]
    media: MediaType,
    format: MediaFormat,
    #[serde(rename = "display")]
    icon: Option<Icon>,
    overlay_label_override: Option<String>,
    overlay_label: String,
    duration: u64,
    file_size: u64,
    channels: Option<ChannelType>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Icon {
    #[serde(rename = "icon16x16")]
    small: Option<String>,
}
