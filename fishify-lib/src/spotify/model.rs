use anyhow::{ /* anyhow ,*/ Result, };

use time::Duration;

use rspotify::{
    model::{
        track::{ SimplifiedTrack, FullTrack, },
        album::{ SimplifiedAlbum, FullAlbum, },
        playlist::{ SimplifiedPlaylist, FullPlaylist, },
        artist::{ SimplifiedArtist, FullArtist, },
        show::{ SimplifiedShow, FullShow, SimplifiedEpisode, FullEpisode, },
        idtypes::{
            TrackId, AlbumId, PlaylistId, ArtistId, ShowId, EpisodeId, 
            PlayableId, PlayContextId, 
            IdError, 
        },
        search::SearchResult,
        PlayableItem,
    },
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ContentType {
    SimplifiedTrack(SimplifiedTrack),
    SimplifiedAlbum(SimplifiedAlbum),
    SimplifiedPlaylist(SimplifiedPlaylist),
    SimplifiedArtist(SimplifiedArtist),
    SimplifiedShow(SimplifiedShow),
    SimplifiedEpisode(SimplifiedEpisode),
    FullTrack(FullTrack),
    FullAlbum(FullAlbum),
    FullPlaylist(FullPlaylist),
    FullArtist(FullArtist),
    FullShow(FullShow),
    FullEpisode(FullEpisode),
}

impl ContentType {
    pub fn ids(&self) -> Option<Vec<PlayableId>> {
        match self {
            Self::SimplifiedTrack(_item) => None,
            Self::SimplifiedAlbum(_item) => None,
            Self::SimplifiedPlaylist(_item) => None,
            Self::SimplifiedArtist(_item) => None,
            Self::SimplifiedShow(_item) => None,
            Self::SimplifiedEpisode(_item) => None,
            Self::FullTrack(_item) => None,
            Self::FullAlbum(item) => Some(item.tracks.items.iter().map(|x| PlayableId::from(x.id.clone().expect("missing id"))).collect()),
            Self::FullPlaylist(item) => Some(item.tracks.items.iter().map(|x| x.track.as_ref().expect("missing track").id().expect("missing id")).collect()),
            Self::FullArtist(_item) => None,
            Self::FullShow(item) => Some(item.episodes.items.iter().map(|x| PlayableId::from(x.id.clone())).collect()),
            Self::FullEpisode(_item) => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ContentId<'a> {
    Track(TrackId<'a>),
    Album(AlbumId<'a>),
    Playlist(PlaylistId<'a>),
    Artist(ArtistId<'a>),
    Show(ShowId<'a>),
    Episode(EpisodeId<'a>),
}

impl<'a> ContentId<'a> {
    pub fn from_uri(uri: &'a str) -> Result<Self> {
        let _type = uri.split(':').nth(1).ok_or(IdError::InvalidFormat)?;
        
        match _type {
            "track" => Ok(ContentId::from(TrackId::from_uri(uri)?)),
            "album" => Ok(ContentId::from(AlbumId::from_uri(uri)?)),
            "playlist" => Ok(ContentId::from(PlaylistId::from_uri(uri)?)),
            "artist" => Ok(ContentId::from(ArtistId::from_uri(uri)?)),
            "show" => Ok(ContentId::from(ShowId::from_uri(uri)?)),
            "episode" => Ok(ContentId::from(EpisodeId::from_uri(uri)?)),
            _ => Err(IdError::InvalidType.into()),
        }
    }

    // id is only ever missing for local content
    pub fn from_search(result: SearchResult) -> Vec<Self> {
        match result {
            SearchResult::Tracks(page) => page.items.into_iter().map(|x| Self::from(x.id.expect("missing id"))).collect(),
            SearchResult::Albums(page) => page.items.into_iter().map(|x| Self::from(x.id.expect("missing id"))).collect(),
            SearchResult::Playlists(page) => page.items.into_iter().map(|x| Self::from(x.id)).collect(),
            SearchResult::Artists(page) => page.items.into_iter().map(|x| Self::from(x.id)).collect(),
            SearchResult::Shows(page) => page.items.into_iter().map(|x| Self::from(x.id)).collect(),
            SearchResult::Episodes(page) => page.items.into_iter().map(|x| Self::from(x.id)).collect(),
        }
    }
}

pub trait ContentInfo {
    fn name(&self) -> String;
    fn artists(&self) -> Vec<SimplifiedArtist>;
    fn duration(&self) -> Option<Duration>;
}

impl ContentInfo for ContentType {
    fn name(&self) -> String {
        match self {
            Self::SimplifiedTrack(item) => item.name.clone(),
            Self::SimplifiedAlbum(item) => item.name.clone(),
            Self::SimplifiedPlaylist(item) => item.name.clone(),
            Self::SimplifiedArtist(item) => item.name.clone(),
            Self::SimplifiedShow(item) => item.name.clone(),
            Self::SimplifiedEpisode(item) => item.name.clone(),
            Self::FullTrack(item) => item.name.clone(),
            Self::FullAlbum(item) => item.name.clone(),
            Self::FullPlaylist(item) => item.name.clone(),
            Self::FullArtist(item) => item.name.clone(),
            Self::FullShow(item) => item.name.clone(),
            Self::FullEpisode(item) => item.name.clone(),
        }
    }

    fn artists(&self) -> Vec<SimplifiedArtist> {
        match self {
            Self::SimplifiedTrack(item) => item.artists.clone(),
            Self::SimplifiedAlbum(item) => item.artists.clone(),
            Self::SimplifiedPlaylist(_item) => vec![],
            Self::SimplifiedArtist(item) => vec![item.clone()],
            Self::SimplifiedShow(_item) => vec![],
            Self::SimplifiedEpisode(_item) => vec![],
            Self::FullTrack(item) => item.artists.clone(),
            Self::FullAlbum(item) => item.artists.clone(),
            Self::FullPlaylist(_item) => vec![],
            Self::FullArtist(item) => vec![SimplifiedArtist { 
                external_urls: item.external_urls.clone(), 
                href: Some(item.href.clone()),
                id: Some(item.id.clone()),
                name: item.name.clone(),
            }],
            Self::FullShow(_item) => vec![],
            Self::FullEpisode(_item) => vec![],
        }
    }

    fn duration(&self) -> Option<Duration> {
        match self {
            Self::SimplifiedTrack(item) => Some(item.duration.clone()),
            Self::SimplifiedAlbum(_item) => None,
            Self::SimplifiedPlaylist(_item) => None,
            Self::SimplifiedArtist(_item) => None,
            Self::SimplifiedShow(_item) => None,
            Self::SimplifiedEpisode(item) => Some(item.duration.clone()),
            Self::FullTrack(item) => Some(item.duration.clone()),
            Self::FullAlbum(_item) => None,
            Self::FullPlaylist(_item) => None,
            Self::FullArtist(_item) => None,
            Self::FullShow(_item) => None,
            Self::FullEpisode(item) => Some(item.duration.clone()),
        }
    }
}

impl ContentInfo for PlayableItem {
    fn name(&self) -> String {
        match self {
            Self::Track(item) => item.name.clone(),
            Self::Episode(item) => item.name.clone(),
        }
    }

    fn artists(&self) -> Vec<SimplifiedArtist> {
        match self {
            Self::Track(item) => item.artists.clone(),
            Self::Episode(_item) => vec![],
        }
    }

    fn duration(&self) -> Option<Duration> {
        match self {
            Self::Track(item) => Some(item.duration.clone()),
            Self::Episode(item) => Some(item.duration.clone()),
        }
    }
}

// Froms

impl<'a> From<TrackId<'a>> for ContentId<'a> {
    fn from(id: TrackId<'a>) -> Self {
        return Self::Track(id);
    }
}

impl<'a> From<AlbumId<'a>> for ContentId<'a> {
    fn from(id: AlbumId<'a>) -> Self {
        return Self::Album(id);
    }
}

impl<'a> From<PlaylistId<'a>> for ContentId<'a> {
    fn from(id: PlaylistId<'a>) -> Self {
        return Self::Playlist(id);
    }
}

impl<'a> From<ArtistId<'a>> for ContentId<'a> {
    fn from(id: ArtistId<'a>) -> Self {
        return Self::Artist(id);
    }
}

impl<'a> From<ShowId<'a>> for ContentId<'a> {
    fn from(id: ShowId<'a>) -> Self {
        return Self::Show(id);
    }
}

impl<'a> From<EpisodeId<'a>> for ContentId<'a> {
    fn from(id: EpisodeId<'a>) -> Self {
        return Self::Episode(id);
    }
}

impl<'a> From<PlayableId<'a>> for ContentId<'a> {
    fn from(id: PlayableId<'a>) -> Self {
        match id {
            PlayableId::Track(id) => Self::Track(id),
            PlayableId::Episode(id) => Self::Episode(id),
        }
    }
}

impl<'a> From<PlayContextId<'a>> for ContentId<'a> {
    fn from(id: PlayContextId<'a>) -> Self {
        match id {
            PlayContextId::Artist(id) => Self::Artist(id),
            PlayContextId::Album(id) => Self::Album(id),
            PlayContextId::Playlist(id) => Self::Playlist(id),
            PlayContextId::Show(id) => Self::Show(id),
        }
    }
}

impl From<SimplifiedTrack> for ContentType {
    fn from(item: SimplifiedTrack) -> Self {
        return Self::SimplifiedTrack(item);
    }
}

impl From<SimplifiedAlbum> for ContentType {
    fn from(item: SimplifiedAlbum) -> Self {
        return Self::SimplifiedAlbum(item);
    }
}

impl From<SimplifiedPlaylist> for ContentType {
    fn from(item: SimplifiedPlaylist) -> Self {
        return Self::SimplifiedPlaylist(item);
    }
}

impl From<SimplifiedArtist> for ContentType {
    fn from(item: SimplifiedArtist) -> Self {
        return Self::SimplifiedArtist(item);
    }
}

impl From<SimplifiedShow> for ContentType {
    fn from(item: SimplifiedShow) -> Self {
        return Self::SimplifiedShow(item);
    }
}

impl From<SimplifiedEpisode> for ContentType {
    fn from(item: SimplifiedEpisode) -> Self {
        return Self::SimplifiedEpisode(item);
    }
}

impl From<FullTrack> for ContentType {
    fn from(item: FullTrack) -> Self {
        return Self::FullTrack(item);
    }
}

impl From<FullAlbum> for ContentType {
    fn from(item: FullAlbum) -> Self {
        return Self::FullAlbum(item);
    }
}

impl From<FullPlaylist> for ContentType {
    fn from(item: FullPlaylist) -> Self {
        return Self::FullPlaylist(item);
    }
}

impl From<FullArtist> for ContentType {
    fn from(item: FullArtist) -> Self {
        return Self::FullArtist(item);
    }
}

impl From<FullShow> for ContentType {
    fn from(item: FullShow) -> Self {
        return Self::FullShow(item);
    }
}

impl From<FullEpisode> for ContentType {
    fn from(item: FullEpisode) -> Self {
        return Self::FullEpisode(item);
    }
}
