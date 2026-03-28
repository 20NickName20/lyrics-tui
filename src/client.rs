use std::{time::{Duration, Instant}};

use mpris::{Metadata, PlaybackStatus, Player, PlayerFinder};

const FETCH_INTERVAL: Duration = Duration::from_millis(1000);

struct State {
    is_playing: bool,
    position: Duration,
    metadata: Metadata
}

fn fetch_state(player: &Player) -> anyhow::Result<State> {
    Ok(State {
        is_playing: player.get_playback_status()? == PlaybackStatus::Playing,
        position: player.get_position()?,
        metadata: player.get_metadata()?
    })
}

pub struct Client {
    finder: PlayerFinder,
    player: Option<Player>,
    state: Option<State>,
    last_fetch: Instant,
    last_pos_fetch: Instant,
}

impl Client {
    pub fn new() -> anyhow::Result<Self> {
        let finder = PlayerFinder::new()?;
        let player = finder.find_active().ok();
        let state = match &player {
            Some(p) => Some(fetch_state(p)?),
            None => None
        };

        Ok(Self {
            finder,
            player,
            state,
            last_fetch: Instant::now(),
            last_pos_fetch: Instant::now()
        })
    }

    pub fn get_position(&self) -> Duration {
        let Some(state) = &self.state else {
            return Duration::ZERO;
        };
        state.position
    }

    fn get_song_info(&self) -> Option<(String, String)> {
        let metadata = &self.state.as_ref()?.metadata;
        let artists = metadata.artists()?;
        let artist = artists.join(", ");
        let title = metadata.title()?.to_string();
        Some((artist, title))
    }

    pub fn update<F>(&mut self, mut f: F) -> anyhow::Result<()>
    where F: FnMut(Option<(String, String)>) {
        if self.last_fetch.elapsed() > FETCH_INTERVAL {
            self.last_fetch = Instant::now();

            if let Ok(new_player) = self.finder.find_active() {
                let replace = match &self.player {
                    Some(old) => old.identity() != new_player.identity(),
                    None => true,
                };

                if replace {
                    self.player = Some(new_player);
                }
            }

            self.last_pos_fetch = Instant::now();
            if let Some(player) = &self.player {
                self.state = Some(fetch_state(player)?);
                f(self.get_song_info());
            }
        } else {
            if let Some(state) = self.state.as_mut() && let Some(player) = &self.player {
                state.is_playing = player.get_playback_status()? == PlaybackStatus::Playing;
                state.position = player.get_position()?;
            }
        }

        Ok(())
    }
}

