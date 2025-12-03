use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    status_version: u32,
    fw_version: String,
    product_type: String,
    battery_level: u32,
    als: u32,
    free_disk: u32,
    shutdownt_imeout: u32,
    dbat_timeout: u32, 
    charging: bool,
    active_card: String,
    card_inserted: bool,
    playing_status: u32,
    headphones: bool,
    dnow_brightness: u32,
    day_bright: u32,
    night_bright: u32,
    bluetooth_hp: bool
    volume: u32,
    user_volume: u32,
    time_format: String,
    nightlight_mode: String,
    temp: String,
    day: u32,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    repeat_all: bool,
    streaming: bool,
    volume: u32,
    volume_max: u32,
    playback_wait: bool,
    sleep_timer_active: bool,
    event_utc: i64, // UNIX Timestamp
    track_length: u32, // seconds
    position: u32, // seconds
    card_id: String,
    source: String, // e.g. "card", "remote", "MQTT"
    card_updated_at: DateTime<Utc>,
    chapter_title: String,
    chapter_key: String,
    track_title: String,
    track_key: String,
    playback_status: String, // e.g. "playing", "paused", "stopped"
    sleep_timer_seconds: u32, // seconds
}

pub struct CardTarget {
    uri: String,
    chapterKey: Option<String>,
    trackKey: Option<String>,
    secondsIn: Option<u32>,
    cutOff: Option<u32>,
    anyButtonStop: Option<bool>,
}

pub enum Command {
    Reboot,
    GetStatus,
    GetEvents,
    SetVolume(u32),
    SetAmbient(u8, u8, u8),
    SetSleepTimer(u32),
    ShowIcon(uri: String, timeout: u32, animated: bool),
    Start(CardTarget),
    Stop,
    Pause,
    Resume,
    BluetoothOn,
    BluetoothOff,
    BluetoothConnect,
    BluetoothDisconnect,
    BluetoothState,
}
