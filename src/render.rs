use crossterm::{QueueableCommand, cursor::MoveTo, style::{Attribute, Color, Print, SetAttribute, SetForegroundColor}, terminal::{Clear, ClearType}};

use crate::LyricsApp;

const LYRICS_POS: (u16, u16) = (6, 5);

const BORDER_COLOR: Color = Color::AnsiValue(92);
const BLOCK_COLOR: Color = Color::AnsiValue(248);

const FADE_PALLETE: [u8; 5] = [237, 239, 242, 245, 15];

pub fn draw_lyrics(app: &mut LyricsApp) -> anyhow::Result<()> {
    let width = app.term_size.0 - 12;

    let draw_lines = app.data.visible_lines.min(9);
    let start_line = (app.data.visible_lines - draw_lines) / 2;
    let clear_str = " ".repeat(width as usize);
    if let Some(lyrics) = &app.data.lyrics {
        for i in 0..draw_lines {
            let line = (app.data.current_line + i as usize).saturating_sub(draw_lines as usize / 2);
            let color = Color::AnsiValue(FADE_PALLETE[4 - i.abs_diff(draw_lines / 2) as usize]);
            let lyric = lyrics.get_line(line);
            let x = (width.saturating_sub(lyric.chars().count() as u16)) / 2;
            app.stdout
                .queue(MoveTo(LYRICS_POS.0, LYRICS_POS.1 + start_line + i))?
                .queue(Print(&clear_str))?
                .queue(MoveTo(LYRICS_POS.0 + x, LYRICS_POS.1 + start_line + i))?
                .queue(SetAttribute(Attribute::Bold))?
                .queue(SetForegroundColor(color))?
                .queue(Print(lyric))?
                .queue(SetAttribute(Attribute::Reset))?;
        }
    } else {
        for i in 0..draw_lines {
            app.stdout
                .queue(MoveTo(LYRICS_POS.0, LYRICS_POS.1 + start_line + i))?
                .queue(Print(&clear_str))?;
        }
        let text = "No lyrics";
        let x = (width.saturating_sub(text.chars().count() as u16)) / 2;
        app.stdout
            .queue(SetForegroundColor(Color::AnsiValue(FADE_PALLETE[3])))?
            .queue(MoveTo(LYRICS_POS.0 + x, LYRICS_POS.1 + start_line + draw_lines / 2))?
            .queue(Print(text))?
            .queue(SetAttribute(Attribute::Reset))?;
    }

    app.data.redraw_lyrics = false;
    Ok(())
}

pub fn render_block(app: &mut LyricsApp, x0: u16, y0: u16, width: u16, height: u16) -> anyhow::Result<()> {
    let x1 = x0 + width;
    let y1 = y0 + height;
    app.stdout
        .queue(MoveTo(x0, y0))?
        .queue(Print("┌"))?
        .queue(Print("╌".repeat(width as usize - 2)))?
        .queue(Print("┐"))?;

    for y in (y0 + 1)..(y1 - 1) {
        app.stdout
            .queue(MoveTo(x0, y))?
            .queue(Print("╎"))?
            .queue(MoveTo(x1 - 1, y))?
            .queue(Print("╎"))?;
    }

    app.stdout
        .queue(MoveTo(x0, y1 - 1))?
        .queue(Print("└"))?
        .queue(Print("╌".repeat(width as usize - 2)))?
        .queue(Print("┘"))?;

    Ok(())
}

pub fn draw_borders(app: &mut LyricsApp) -> anyhow::Result<()> {
    let (width, height) = app.term_size;
    app.stdout
        .queue(Clear(ClearType::All))?
        .queue(MoveTo(1, 0))?
        .queue(SetForegroundColor(BORDER_COLOR))?
        .queue(Print("╓"))?
        .queue(Print("─".repeat(width as usize - 4)))?
        .queue(Print("╖"))?

        .queue(MoveTo(1, 1))?
        .queue(Print("║"))?
        .queue(MoveTo(width - 2, 1))?
        .queue(Print("║"))?

        .queue(MoveTo(1, 2))?
        .queue(Print("╠"))?
        .queue(Print("═".repeat(width as usize - 4)))?
        .queue(Print("╣"))?

        .queue(MoveTo(1, 3))?
        .queue(Print("║ ┌"))?
        .queue(Print("─".repeat(width as usize - 8)))?
        .queue(Print("┐ ║"))?;

    for y in 4..(height - 2) {
        app.stdout
            .queue(MoveTo(1, y))?
            .queue(Print("║ │"))?
            .queue(MoveTo(width - 4, y))?
            .queue(Print("│ ║"))?;
    }

    app.stdout
        .queue(MoveTo(1, height - 2))?
        .queue(Print("║ └"))?
        .queue(Print("─".repeat(width as usize - 8)))?
        .queue(Print("┘ ║"))?

        .queue(MoveTo(1, height - 1))?
        .queue(Print("╚"))?
        .queue(Print("═".repeat(width as usize - 4)))?
        .queue(Print("╝"))?

        .queue(SetForegroundColor(BLOCK_COLOR))?;

    render_block(app, 5, 4, width - 10, height - 6)?;
    app.data.visible_lines = height - 8;

    app.data.redraw_screen = false;
    Ok(())
}

pub fn draw_title(app: &mut LyricsApp) -> anyhow::Result<()> {
    let width = app.term_size.0 - 12;
    let height = app.data.visible_lines;
    let draw_lines = app.data.visible_lines.min(9);
    let y = (height.saturating_sub(draw_lines)) / 4;
    app.stdout
        .queue(MoveTo(LYRICS_POS.0, LYRICS_POS.1 + y))?
        .queue(Print(" ".repeat(width as usize)))?;
    if let Some((artist, title)) = app.data.song_info.as_ref() {
        let len = artist.chars().count() + 3 + title.chars().count();
        let x = (width.saturating_sub(len as u16)) / 2;
        app.stdout
            .queue(SetForegroundColor(Color::White))?
            .queue(SetAttribute(Attribute::Bold))?
            .queue(MoveTo(LYRICS_POS.0 + x, LYRICS_POS.1 + y))?
            .queue(Print(artist))?
            .queue(Print(" - "))?
            .queue(Print(title))?
            .queue(SetAttribute(Attribute::Reset))?;
    } else {
        let text = "No song is playing";
        let x = (width.saturating_sub(text.chars().count() as u16)) / 2;
        app.stdout
            .queue(SetForegroundColor(Color::AnsiValue(208)))?
            .queue(MoveTo(LYRICS_POS.0 + x, LYRICS_POS.1 + y))?
            .queue(Print(text))?
            .queue(SetAttribute(Attribute::Reset))?;
    }

    app.data.redraw_title = false;
    Ok(())
}
