mod model;

use model::{ ContentInfo, ContentType, ContentId, };

use time::Duration;

use anyhow::{anyhow, /* bail, */ Result};
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

// TODO: fix this shit it ass (all of it, like the whole file)

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

impl Fishify for AuthCodeSpotify {}
impl FishifyClient for AuthCodeSpotify {}

#[async_trait]
pub trait FishifyClient: Fishify {
    async fn play(&self, q: Option<String>, _type: Option<SearchType>, is_url: bool, queue: bool) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];
        if q.is_none() {
            self.resume_playback(None, None).await?;
            response.push("Resumed playback".to_string());
            return Ok(response)
        } 
        let query = q.unwrap();

        let search_type = _type.unwrap_or(SearchType::Track);

        let uri = url_to_uri(&query);
        let id = if is_url {
            self.play_uri(uri.as_ref().ok_or(anyhow!("Invalid url"))?, queue).await?
        } else {
            self.play_query(&query, search_type, queue).await?
        };
        let playing = self.get_content(id).await?;

        let name = playing.name();
        let artists = playing.artists();
        let artist = artists.get(0);
        let prefix: String = if queue {
            "Queued".to_string()
        } else {
            "Now playing".to_string()
        };
        
        if let Some(art) = artist {
            response.push(format!("{prefix} {name} by {0}", art.name));
        } else {
            response.push(format!("{prefix} {name}"));
        };

        Ok(response)
    }

    async fn queue_list(&self) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];

        let current_queue = self.current_user_queue().await?;
        if let Some(item) = &current_queue.currently_playing {
            let name = item.name();
            let artists = item.artists();
            let artist = artists.get(0);
            if let Some(art) = artist {
                response.push(format!("Currently playing {name} by {0}", art.name));
            } else {
                response.push(format!("Currently playing {name}"));
            }
        }

        for (i, item) in current_queue.queue.iter().enumerate() {
            let name = item.name();
            let artists = item.artists();
            let artist = artists.get(0);
            let index = i+1;
            if let Some(art) = artist {
                response.push(format!("{index:>3}. {name} \u{2014} {0}", art.name));
            } else {
                response.push(format!("{index:>3}. {name}"));
            }
        }

        Ok(response)
    }

    async fn pause(&self) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];
        self.pause_playback(None).await?;
        response.push("Paused playback".to_string());
        Ok(response)
    }

    async fn skip(&self, count: u8) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];
        for _ in 0..count {
            self.next_track(None).await?;
        }
        response.push(format!("Skipped {count} tracks"));
        Ok(response)
    }

    async fn status(&self) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];

        let playback = self.current_playback(None, None::<Vec<&AdditionalType>>).await?.ok_or(anyhow!("No current playback"))?;

        // This will create a message with the format:
        //   {is_playing}
        //   {_type} {type_name}
        //   {name} --- {artist}
        //   {progress} / {duration}
        //   Volume: {volume}%
        //   Shuffle: {shuffle}
        //   Repeat: {repeat}

        if playback.is_playing {
            response.push("Playing".to_string());
        } else {
            response.push("Paused".to_string());
        }

        if let Some(context) = playback.context {
            let _type = context._type;
            let name = self.get_content(ContentId::from_uri(&context.uri).unwrap()).await?.name();
            
            response.push(format!("{_type:?}: {name}"));
        }

        if let Some(item) = playback.item {
            let name = item.name();
            let artists = item.artists();
            let artist = artists.get(0);

            response.push(
                if let Some(art) = artist {
                    format!("{name} \u{2014} {0}", art.name)
                } else {
                    format!("{name}")
                }
            );

            if let Some(progress) = playback.progress {
                let duration = item.duration().unwrap();
                response.push(format!("{} / {}", duration_clock_format(progress), duration_clock_format(duration)));
            }
        }

        if let Some(volume) = playback.device.volume_percent {
            response.push(format!("Volume: {volume}%"));
        }

        let shuffle_state = if playback.shuffle_state {
            "On"
        } else {
            "Off"
        };
        response.push(format!("Shuffle: {shuffle_state}"));

        response.push(format!("Repeat: {:?}", playback.repeat_state));

        Ok(response)
    }

    async fn device_list(&self) -> Result<Vec<String>> {
        let devices = self.device().await?;
        Ok(devices.into_iter().map(|dev| {
            let name = dev.name;
            let _type = dev._type;
            let id = match dev.id {
                Some(id) => id,
                None => "None".to_string(),
            };

            format!("{_type:?} {name} \u{2014} {id}")
        }).collect())
    }

    async fn device_connect(&self, name: Option<String>) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];
        
        let device = self.device_get(name).await?;
        let device_id = device.id.as_ref().ok_or(anyhow!("Missing device id"))?;
        let device_name = device.name;

        self.transfer_playback(&device_id, None).await?;
        response.push(format!("Connected to {device_name}"));

        Ok(response)
    }

    async fn device_status(&self) -> Result<Vec<String>> {
        let mut response: Vec<String> = vec![];
        
        let device = self.active_device().await?;

        match device {
            Some(dev) => {
                let name = dev.name;
                let id = match dev.id {
                    Some(id) => id,
                    None => "None".to_string(),
                };
                let is_active = dev.is_active.to_string();
                let _type = dev._type;
                response.push(format!("Device: {name}"));
                response.push(format!("Id: {id}"));
                response.push(format!("Active: {is_active}"));
                response.push(format!("Type: {_type:?}"));
            },
            None => response.push(format!("No playback")),
        }

        Ok(response)
    }

    async fn set_volume(&self, level: u8) -> Result<Vec<String>> {
        self.volume(level, None).await?;
        Ok(vec![format!("Success")])
    }

    async fn set_shuffle(&self, state: bool) -> Result<Vec<String>> {
        self.shuffle(state, None).await?;
        Ok(vec![format!("Success")])
    }

    async fn set_repeat(&self, state: RepeatState) -> Result<Vec<String>> {
        self.repeat(state, None).await?;
        Ok(vec![format!("Success")])
    }
}

#[async_trait]
pub trait Fishify: OAuthClient + BaseClient {
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
        let id = ContentId::from_search(result).swap_remove(0);
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

