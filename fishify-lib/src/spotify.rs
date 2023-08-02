use crate::model::{ ContentInfo, ContentType, ContentId, FromSearch, };

use time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rspotify::{
    AuthCodeSpotify,
    clients::{ OAuthClient, BaseClient, },
    model::{
        device::Device,
        enums::{
            types::{ AdditionalType, SearchType, },
            misc::{ RepeatState, },
        },
    },
    prelude::{ PlayContextId, PlayableId, },
};

fn url_to_uri(url: &str) -> Option<String> {
    let base_url: &str = url.split('?').next()?;
    let mut split_url = base_url.rsplit('/');
    let id = split_url.next()?;
    let _type = split_url.next()?;

    if split_url.next()? == "open.spotify.com" {
        Some(format!("spotify:{}:{}", _type, id))
    } else {
        None
    }
}

fn duration_clock_format(duration: Duration) -> String {
    let total_sec = duration.num_seconds();
    let h = total_sec / 60 / 60;
    let m = total_sec / 60 % 60;
    let s = total_sec % 60;
    
    if h > 0 {
        format!("{h}:{m:0>2}:{s:0>2}")
    } else {
        format!("{m}:{s:0>2}")
    }
}

impl FishifyClient for AuthCodeSpotify {}

#[derive(Clone, Debug)]
pub struct Fishify<'a> {
    spotify: &'a AuthCodeSpotify,
    pub response: Vec<String>,
    pub show: bool,
}

impl<'a> From<&'a AuthCodeSpotify> for Fishify<'a> {
    fn from(spotify: &'a AuthCodeSpotify) -> Self {
        return Self {
            spotify: spotify,
            response: vec![],
            show: true,
        };
    }
}

impl<'a> Fishify<'a> {
    pub async fn play(&mut self, q: Option<String>, _type: Option<SearchType>, is_url: bool, queue: bool) -> Result<()> {
        if q.is_none() {
            self.spotify.resume_playback(None, None).await?;
            self.response.push("Resumed playback".to_string());
            self.show = false;
            return Ok(());
        } 
        let query = q.unwrap();

        let search_type = _type.unwrap_or(SearchType::Track);

        let uri: String;
        let id = if is_url {
            uri = url_to_uri(&query).ok_or(anyhow!("Invalid url"))?;
            self.spotify.play_uri(&uri, queue).await?
        } else {
            self.spotify.play_query(&query, search_type, queue).await?
        };
        let playing = self.spotify.get_content(id).await?;

        let name = playing.name();
        let prefix: String = if queue {
            "Queued".to_string()
        } else {
            "Now playing".to_string()
        };
        
        if let Some(artist) = playing.artist() {
            self.response.push(format!("{prefix} {name} by {0}", artist.name));
        } else {
            self.response.push(format!("{prefix} {name}"));
        };

        Ok(())
    }

    pub async fn queue_list(&mut self) -> Result<()> {
        let current_queue = self.spotify.current_user_queue().await?;
        if let Some(item) = &current_queue.currently_playing {
            let name = item.name();
            if let Some(artist) = item.artist() {
                self.response.push(format!("Currently playing {name} by {0}", artist.name));
            } else {
                self.response.push(format!("Currently playing {name}"));
            }
        }

        for (i, item) in current_queue.queue.iter().enumerate() {
            let name = item.name();
            let index = i+1;
            if let Some(artist) = item.artist() {
                self.response.push(format!("{index:>3}. {name} \u{2014} {0}", artist.name));
            } else {
                self.response.push(format!("{index:>3}. {name}"));
            }
        }
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<()> {
        self.spotify.pause_playback(None).await?;
        self.response.push("Paused playback".to_string());
        self.show = false;
        Ok(())
    }

    pub async fn skip(&mut self, count: u8) -> Result<()> {
        for _ in 0..count {
            self.spotify.next_track(None).await?;
        }
        self.response.push(format!("Skipped {count} tracks"));
        self.show = false;
        Ok(())
    }

    pub async fn status(&mut self) -> Result<()> {
        let playback = self.spotify.current_playback(None, None::<Vec<&AdditionalType>>).await?.ok_or(anyhow!("No current playback"))?;

        // This will create a message with the format:
        //   {is_playing}
        //   {_type} {type_name}
        //   {name} --- {artist}
        //   {progress} / {duration}
        //   Volume: {volume}%
        //   Shuffle: {shuffle}
        //   Repeat: {repeat}

        if playback.is_playing {
            self.response.push("Playing".to_string());
        } else {
            self.response.push("Paused".to_string());
        }

        if let Some(context) = playback.context {
            let _type = context._type;
            let name = self.spotify.get_content(ContentId::from_uri(&context.uri).unwrap()).await?.name();
            
            self.response.push(format!("{_type:?}: {name}"));
        }

        if let Some(item) = playback.item {
            let name = item.name();

            self.response.push(
                if let Some(artist) = item.artist() {
                    format!("{name} \u{2014} {0}", artist.name)
                } else {
                    format!("{name}")
                }
            );

            if let Some(progress) = playback.progress {
                let duration = item.duration().unwrap();
                self.response.push(format!("{} / {}", duration_clock_format(progress), duration_clock_format(duration)));
            }
        }

        if let Some(volume) = playback.device.volume_percent {
            self.response.push(format!("Volume: {volume}%"));
        }

        let shuffle_state = if playback.shuffle_state {
            "On"
        } else {
            "Off"
        };
        self.response.push(format!("Shuffle: {shuffle_state}"));

        self.response.push(format!("Repeat: {:?}", playback.repeat_state));

        Ok(())
    }

    pub async fn search(&mut self, q: String, _type: Option<SearchType>, limit: Option<u32>) -> Result<()> {
        let result = self.spotify.search(&q, _type.unwrap_or(SearchType::Track), None, None, Some(limit.unwrap_or(10)), None).await?;
        let results = ContentType::from_search(result);

        for item in results {
            if let Some(artist) = item.artist() {
                self.response.push(format!("{} \u{2014} {}", item.name(), artist.name))
            } else {
                self.response.push(format!("{}", item.name()))
            }
        }

        Ok(())
    }

    pub async fn device_list(&mut self) -> Result<()> {
        let devices = self.spotify.device().await?;

        for dev in devices {
            let name = dev.name;
            let _type = dev._type;
            let id = match dev.id {
                Some(id) => id,
                None => "None".to_string(),
            };
            self.response.push(format!("{_type:?} {name} \u{2014} {id}"));
        }

        Ok(())
    }

    pub async fn device_connect(&mut self, name: Option<String>) -> Result<()> {
        let device = self.spotify.device_get(name).await?;
        let device_id = device.id.as_ref().ok_or(anyhow!("Missing device id"))?;
        let device_name = device.name;

        self.spotify.transfer_playback(&device_id, None).await?;
        self.response.push(format!("Connected to {device_name}"));
        self.show = false;

        Ok(())
    }

    pub async fn device_status(&mut self) -> Result<()> {
        let device = self.spotify.active_device().await?;

        match device {
            Some(dev) => {
                let name = dev.name;
                let id = match dev.id {
                    Some(id) => id,
                    None => "None".to_string(),
                };
                let is_active = dev.is_active.to_string();
                let _type = dev._type;
                self.response.push(format!("Device: {name}"));
                self.response.push(format!("Id: {id}"));
                self.response.push(format!("Active: {is_active}"));
                self.response.push(format!("Type: {_type:?}"));
            },
            None => return Err(anyhow!("No active device")),
        }

        Ok(())
    }

    pub async fn set_volume(&mut self, level: u8) -> Result<()> {
        self.spotify.volume(level, None).await?;
        self.response.push(format!("Set volume to {level}"));
        self.show = false;
        Ok(())
    }

    pub async fn set_shuffle(&mut self, state: bool) -> Result<()> {
        self.spotify.shuffle(state, None).await?;
        self.response.push(format!("Set shuffle to {state}"));
        self.show = false;
        Ok(())
    }

    pub async fn set_repeat(&mut self, state: RepeatState) -> Result<()> {
        self.spotify.repeat(state, None).await?;
        self.response.push(format!("Set repeat to {state:?}"));
        self.show = false;
        Ok(())
    }
}

#[async_trait]
trait FishifyClient: OAuthClient + BaseClient {
    async fn device_get(&self, name: Option<String>) -> Result<Device> {
        let devices: Vec<Device> = self.device().await?;

        let device = match name {
            Some(target) => devices.into_iter().find(|device| device.name == target).ok_or(anyhow!("Device not found"))?,
            None => devices.into_iter().next().ok_or(anyhow!("No devices found"))?,
        };

        Ok(device)
    }

    async fn active_device(&self) -> Result<Option<Device>> {
        let playback_option = self.current_playback(None, None::<Vec<&AdditionalType>>).await?;

        match playback_option {
            Some(playback) => Ok(Some(playback.device)),
            None => Ok(None),
        }
    }

    async fn play_query(&self, query: &str, _type: SearchType, queue: bool) -> Result<ContentId> {
        let result = self.search(query, _type, None, None, Some(1), None).await?;
        let id = ContentId::from_search(result).next().ok_or(anyhow!("No search result"))?;
        self.play_id(id.clone(), queue).await?;
        Ok(id)
    }

    async fn play_uri(&'async_trait self, uri: &'async_trait str, queue: bool) -> Result<ContentId> {
        let id = ContentId::from_uri(uri)?;
        self.play_id(id.clone(), queue).await?;
        Ok(id)
    }

    async fn play_id(&self, content_id: ContentId<'async_trait>, queue: bool) -> Result<()> {
        if !queue {
            match content_id {
                ContentId::Track(id) => self.start_uris_playback([PlayableId::from(id)], None, None, None).await?,
                ContentId::Episode(id) => self.start_uris_playback([PlayableId::from(id)], None, None, None).await?,
                ContentId::Album(id) => self.start_context_playback(PlayContextId::from(id), None, None, None).await?,
                ContentId::Playlist(id) => self.start_context_playback(PlayContextId::from(id), None, None, None).await?,
                ContentId::Artist(id) => self.start_context_playback(PlayContextId::from(id), None, None, None).await?,
                ContentId::Show(id) => self.start_context_playback(PlayContextId::from(id), None, None, None).await?,
            }
        } else {
            match content_id {
                ContentId::Track(id) => self.add_item_to_queue(PlayableId::from(id), None).await?,
                ContentId::Episode(id) => self.add_item_to_queue(PlayableId::from(id), None).await?,
                ContentId::Album(id) => self.queue_context_id(PlayContextId::from(id)).await?,
                ContentId::Playlist(id) => self.queue_context_id(PlayContextId::from(id)).await?,
                ContentId::Artist(id) => self.queue_context_id(PlayContextId::from(id)).await?,
                ContentId::Show(id) => self.queue_context_id(PlayContextId::from(id)).await?,
            }
        }
        Ok(())
    }

    async fn queue_context_id(&self, context_id: PlayContextId<'async_trait>) -> Result<()> {
        let content_id = ContentId::from(context_id);
        let context = self.get_content(content_id).await?;
        let ids: Vec<PlayableId> = context.ids().ok_or(anyhow!("Failed to queue"))?;
        for id in ids {
            self.add_item_to_queue(id, None).await?;
        }
        Ok(())
    }

    async fn get_content(&self, content_id: ContentId<'async_trait>) -> Result<ContentType> {
        match content_id {
            ContentId::Track(id) => Ok(ContentType::from(self.track(id).await?)),
            ContentId::Episode(id) => Ok(ContentType::from(self.get_an_episode(id, None).await?)),
            ContentId::Album(id) => Ok(ContentType::from(self.album(id).await?)),
            ContentId::Playlist(id) => Ok(ContentType::from(self.playlist(id, None, None).await?)),
            ContentId::Artist(id) => Ok(ContentType::from(self.artist(id).await?)),
            ContentId::Show(id) => Ok(ContentType::from(self.get_a_show(id, None).await?)),
        }
    }
}

