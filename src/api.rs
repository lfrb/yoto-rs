use chrono::{DateTime, TimeDelta, Utc};
use reqwest::{header, header::HeaderMap, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crate::model::*;

#[derive(Default)]
pub struct Client {
    pub id: String,
    pub token: Option<Token>,
    client: reqwest::blocking::Client,
}

pub enum ClientError {
    NotFound,
    Failed,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Token {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: String,
    scope: String,
    id_token: String,
    valid_until: DateTime<Utc>,
}

pub enum RefreshStatus {
    AlreadyValid,
    Refreshed,
    Failed,
}

enum GrantType<'a> {
    DeviceCode(&'a str),
    RefreshToken(&'a str),
}

#[derive(Deserialize)]
struct AuthResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    expires_in: i64,
    interval: u64,
}

#[derive(Deserialize)]
struct AuthErrorMessage {
    error: String,
    #[serde(default)]
    error_description: String,
}

enum AuthError {
    Pending,
    SlowDown,
    Failed,
}

#[derive(Deserialize, Serialize)]
struct DeviceList {
    devices: Vec<Device>,
}

#[derive(Deserialize, Serialize)]
struct CardList {
    cards: Vec<Card>,
}

#[derive(Deserialize, Serialize)]
struct ImageList {
    images: Vec<Image>,
}

#[derive(Deserialize, Serialize)]
struct IconList {
    #[serde(rename = "displayIcons")]
    icons: Vec<DisplayIcon>,
}

#[derive(Deserialize, Serialize)]
struct ContentResponse {
    card: Card,
}

#[derive(Deserialize)]
pub struct Upload {
    #[serde(rename = "uploadId")]
    pub id: String,
    #[serde(rename = "uploadUrl")]
    pub url: String,
}

#[derive(Deserialize)]
struct UploadResponse {
    upload: Upload,
}

#[derive(Deserialize)]
struct TranscodedAudio {
    #[serde(rename = "transcodedSha256")]
    uri: Option<String>,
}

#[derive(Deserialize)]
struct TranscodeResponse {
    transcode: TranscodedAudio,
}

static TOKEN_URL: &str = "https://login.yotoplay.com/oauth/token";
static AUTH_URL: &str = "https://login.yotoplay.com/oauth/device/code";
static BASE_URL: &str = "https://api.yotoplay.com";

impl Token {
    pub fn is_expired(&self) -> bool {
        let expiration = Utc::now() + TimeDelta::seconds(30);
        return self.valid_until < expiration;
    }
}

impl Client {
    pub fn new(client_id: &str, token: Option<Token>) -> Client {
        Client {
            id: client_id.to_string(),
            token,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn auth(&mut self) -> Result<(), String> {
        let mut data = HashMap::new();
        data.insert("client_id", self.id.as_ref());
        data.insert("scope", "profile");
        data.insert("audience", BASE_URL);

        let response = self
            .client
            .post(AUTH_URL)
            .form(&data)
            .send()
            .unwrap()
            .json::<AuthResponse>()
            .unwrap();
        println!("User Code: {}", response.user_code);
        println!("Verification: {}", response.verification_uri_complete);

        let mut interval = response.interval;
        loop {
            let result = self.request_token(GrantType::DeviceCode(&response.device_code));
            match result {
                Ok(token) => {
                    self.token = Some(token);
                    return Ok(());
                }
                Err(AuthError::Pending) => {
                    sleep(Duration::from_secs(interval));
                }
                Err(AuthError::SlowDown) => {
                    interval += 5;
                    sleep(Duration::from_secs(interval));
                }
                Err(_) => return Err("Failed to authenticate".to_string()),
            }
        }
    }

    pub fn refresh_token(&mut self) -> RefreshStatus {
        match &self.token {
            Some(token) => {
                if !token.is_expired() {
                    return RefreshStatus::AlreadyValid;
                }

                if let Ok(token) = self.request_token(GrantType::RefreshToken(&token.refresh_token))
                {
                    self.token = Some(token);
                    RefreshStatus::Refreshed
                } else {
                    RefreshStatus::Failed
                }
            }
            None => RefreshStatus::Failed,
        }
    }

    fn request_token(&self, grant_type: GrantType) -> Result<Token, AuthError> {
        let mut data = HashMap::new();
        data.insert("client_id", self.id.as_ref());

        match grant_type {
            GrantType::DeviceCode(code) => {
                println!("Polling for new token");
                data.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
                data.insert("device_code", code);
                data.insert("audience", BASE_URL);
            }
            GrantType::RefreshToken(token) => {
                println!("Requesting refresh token");
                data.insert("grant_type", "refresh_token");
                data.insert("refresh_token", token);
            }
        };

        let response = self.client.post(TOKEN_URL).form(&data).send().unwrap();

        match response.status() {
            StatusCode::OK => {
                let mut token = response.json::<Token>().unwrap();
                token.valid_until = Utc::now() + TimeDelta::seconds(token.expires_in);
                println!("New token valid for {} seconds", token.expires_in);
                Ok(token)
            }
            StatusCode::FORBIDDEN => match response.json::<AuthErrorMessage>() {
                Ok(error) => match error.error.as_ref() {
                    "authorization_pending" => Err(AuthError::Pending),
                    "slow_down" => Err(AuthError::SlowDown),
                    err => {
                        println!("Err: {}", err);
                        Err(AuthError::Failed)
                    }
                },
                _ => Err(AuthError::Failed),
            },
            err => {
                println!("Err {}", err);
                Err(AuthError::Failed)
            }
        }
    }

    fn ensure_token(&self) -> Option<&Token> {
        let token = self.token.as_ref().expect("Not authenticated");
        if token.is_expired() {
            None
        } else {
            Some(token)
        }
    }

    pub fn get_objects<T: DeserializeOwned>(&self, endpoint: &str) -> T {
        let token = self.ensure_token().unwrap();
        let url = format!("{}{}", BASE_URL, endpoint);
        self.client
            .get(url)
            .bearer_auth(&token.access_token)
            .send()
            .unwrap()
            .json::<T>()
            .unwrap()
    }

    pub fn get_object<T: DeserializeOwned>(
        &self,
        endpoint: impl AsRef<str>,
        params: Option<&HashMap<&str, &str>>,
    ) -> T {
        let token = self.ensure_token().unwrap();
        let url = format!("{}{}", BASE_URL, endpoint.as_ref());
        let mut builder = self.client.get(url).bearer_auth(&token.access_token);
        if let Some(p) = params {
            builder = builder.query(p);
        }
        builder.send().unwrap().json::<T>().unwrap()
    }

    pub fn delete_object(&self, endpoint: impl AsRef<str>) {
        let token = self.ensure_token().unwrap();
        let url = format!("{}{}", BASE_URL, endpoint.as_ref());
        self.client
            .delete(url)
            .bearer_auth(&token.access_token)
            .send()
            .unwrap();
    }

    pub fn get_devices(&self) -> Vec<Device> {
        self.get_objects::<DeviceList>("/device-v2/devices/mine")
            .devices
    }

    pub fn get_device_status(&self, id: &str) -> DeviceStatus {
        self.get_object::<DeviceStatus>(format!("/device-v2/{}/status", id), None)
    }

    pub fn get_cards(&self) -> Vec<Card> {
        self.get_objects::<CardList>("/content/mine").cards
    }

    pub fn get_card(&self, id: &str, playable: bool) -> Result<Card, ClientError> {
        let endpoint = format!("/content/{}", id);
        let mut params = HashMap::new();
        if playable {
            params.insert("playable", "true");
            params.insert("signingType", "s3");
        }
        Ok(self
            .get_object::<ContentResponse>(endpoint, Some(&params))
            .card)
    }

    pub fn delete_card(&self, id: &str) {
        let endpoint = format!("/content/{}", id);
        self.delete_object(endpoint);
    }

    pub fn get_family_images(&self) -> Vec<Image> {
        self.get_objects::<ImageList>("/media/family/images").images
    }

    fn request_audio_upload_url(&self) -> Upload {
        let url = format!("{}/media/transcode/audio/uploadUrl", BASE_URL);
        let token = &self.token.as_ref().unwrap().access_token;
        let response = self.client.get(url).bearer_auth(token).send().unwrap();
        response.json::<UploadResponse>().unwrap().upload
    }

    fn send_audio_file(&self, path: &Path, upload: &Upload) -> Result<(), String> {
        let ext = path
            .extension()
            .ok_or("File without extesnion")?
            .to_str()
            .unwrap();
        let format = MediaFormat::from_ext(ext)?;
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, format.content_type().parse().unwrap());

        let data: Vec<u8> = fs::read(path).unwrap();
        let response = self
            .client
            .put(&upload.url)
            .headers(headers)
            .body(data)
            .send()
            .unwrap();

        match response.status() {
            StatusCode::OK => Ok(()),
            _ => Err("Failed to upload file".to_string()),
        }
    }

    fn wait_audio_transcode(&self, upload: &Upload) -> Result<String, String> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());

        let mut attempts = 0;
        let url = format!(
            "{}/media/upload/{}/transcoded?loudnorm=false",
            BASE_URL, &upload.id
        );
        let token = &self.token.as_ref().unwrap().access_token;
        let request = self
            .client
            .get(&url)
            .bearer_auth(token)
            .headers(headers)
            .build()
            .unwrap();

        loop {
            let response = self.client.execute(request.try_clone().unwrap()).unwrap();
            let audio = response.json::<TranscodeResponse>().unwrap().transcode;
            if audio.uri.is_some() {
                return Ok(audio.uri.unwrap());
            }
            attempts += 1;
            if attempts >= 30 {
                return Err("Error transcoding".to_string());
            }
            sleep(Duration::from_millis(500));
        }
    }

    pub fn upload_audio_file(&self, path: &Path) -> Result<String, String> {
        let upload = self.request_audio_upload_url();
        self.send_audio_file(path, &upload)?;
        self.wait_audio_transcode(&upload)
    }
}
