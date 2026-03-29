use std::{io::Write};

use crossterm::{QueueableCommand, cursor::MoveTo, event::{Event, KeyCode}, style::Print, terminal::{self, Clear, ClearType}};

mod app;
use app::App;

mod lyrics;
use lyrics::Lyrics;

mod loader;

mod render;
use render::*;

use crate::{client::Client, loader::load_lyrics};

mod client;

pub struct AppData {
    lyrics: Option<Lyrics>,
    current_line: usize,
    client: Client,
    song_info: Option<(String, String)>,

    redraw_screen: bool,
    redraw_lyrics: bool,
    redraw_title: bool,
    visible_lines: u16,
}

type LyricsApp = App<AppData>;

impl AppData {
    fn new() -> anyhow::Result<Self> {
        let visible_lines = terminal::size()?.1.saturating_sub(8);
        let client = Client::new()?;
        let lyrics = None;
        let song_info = None;
        Ok(Self {
            lyrics,
            current_line: 0,
            client,
            song_info,

            redraw_screen: true,
            redraw_lyrics: false,
            redraw_title: false,
            visible_lines
        })
    }

    fn update(&mut self) -> anyhow::Result<()> {
        self.client.update(|new_song_info| {
            if self.song_info != new_song_info {
                self.song_info = new_song_info;

                self.current_line = 0;
                let lyrcis_source = self.song_info.as_ref().and_then(|(artist, title)| {
                    load_lyrics(artist, title)
                });
                self.lyrics = lyrcis_source.map(Lyrics::from);
                self.redraw_lyrics = true;
                self.redraw_title = true;
            }
        })?;
        if let Some(lyrics) = &self.lyrics {
            let time = self.client.get_position();
            let current_line = lyrics.get_current_line_number(time);
            if self.current_line != current_line {
                self.current_line = current_line;
                self.redraw_lyrics = true;
            }
        }
        Ok(())
    }
}

fn render(app: &mut LyricsApp) -> anyhow::Result<()> {
    let (width, height) = app.term_size;
    if width < 30 || height < 9 {
        app.stdout
            .queue(Clear(ClearType::All))?
            .queue(MoveTo(0, 0))?
            .queue(Print("Terminal is too small!!"))?
            .flush()?;
        return Ok(());
    }

    let mut flush = false;
    if app.data.redraw_screen {
        draw_borders(app)?;
        draw_title(app)?;
        draw_lyrics(app)?;
        flush = true;
    }
    if app.data.redraw_title {
        draw_title(app)?;
        flush = true;
    }
    if app.data.redraw_lyrics {
        draw_lyrics(app)?;
        flush = true;
    }

    if flush {
        app.stdout.flush()?;
    }
    Ok(())
}

fn main_loop(app: &mut LyricsApp) -> anyhow::Result<()> {
    app.data.update()?;

    render(app)?;

    Ok(())
}

fn event_handler(app: &mut LyricsApp, event: Event) -> anyhow::Result<()> {
    match event {
        Event::Key(key_event) if key_event.is_press() => {
            match key_event.code {
                KeyCode::Char('Q') | KeyCode::Char('q') => app.exit(),
                _ => ()
            }
        },
        Event::Resize(_, _) => {
            app.data.redraw_screen = true;
        },
        _ => ()
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let data = AppData::new()?;
    let mut app = App::new(data)?;
    app.main(main_loop, event_handler)
}

