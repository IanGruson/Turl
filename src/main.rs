use std::{error::Error, io};

mod util;
mod database;

use util::event::{Event, Events};
use util::dbhandler::Database;
use termion::{event::Key, raw::IntoRawMode};
use termion::clear::*;
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{Widget, Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction, Rect},
    style::{Color, Modifier, Style},
};

use unicode_width::UnicodeWidthStr;

use database::container;

enum InputMode {
    Normal,
    Command,
    Editing,
}

struct App {
    input : String,
    input_mode : InputMode,
}

impl Default for App {
    fn default() -> App {
        App {
            input : String::new(),
            input_mode : InputMode::Normal,
        }
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    
    let events = Events::new();

    let db = &Database {
        filename : String::from("./.database"),
        connection : sqlite::open("./.database").unwrap(),
    };

    let user = &database::user::get_user(1, db).unwrap().unwrap();
    println!("the user id is : {}", user.id);


    terminal.clear()?;
    loop {

        
        //terminal.autoresize or not it seems to resize automatically anyway.
        //So this is probably not needed.
        terminal.autoresize()?;


        //render UI
        terminal.draw(|f| {

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                Constraint::Percentage(90),
                Constraint::Percentage(10)
                ].as_ref()
                )
                .split(f.size());

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                    Constraint::Percentage(20),
                    Constraint::Percentage(70)
                    ].as_ref()
                    )
                .split(chunks[0]);

            let block = Block::default()
                .title("Collections")
                .borders(Borders::ALL);
            f.render_widget(block, horizontal_chunks[0]);

            let block = Block::default()
                .title("Edit Request")
                .borders(Borders::ALL);
            f.render_widget(block, horizontal_chunks[1]);

            //input chunk (block ? I don't know)
            
            let chunks2 = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                    Constraint::Percentage(100),
                    ].as_ref()
                    )
                .split(chunks[1]);

            let input = Paragraph::new(app.input.as_ref())
                .style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Command => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks2[0]);

            //Move cursor to the bottom of the page.
            match app.input_mode {
                InputMode::Normal => {}
                InputMode::Command => {
                    f.set_cursor(
                        // Place the cursor at the end of the input as you are 
                        // typing.
                        chunks2[0].x + app.input.width() as u16 +1,
                        // Move one line down to leave a border.
                        chunks2[0].y + 1, 
                        )
                }
                InputMode::Editing => {}
            
        };

        })?;
        
        //call of the input event handler
        if let Event::Input(input) = events.next()? {
            match app.input_mode {

                InputMode::Normal => match input {


                    Key::Char('w') => {
                        container::create_workspace(user, "test", db)?;
                    }
                    Key::Char('i') => {
                        container::create_collection(String::from("test"), 1, db)?;
                    }
                    Key::Char(':') => {
                        // println!("You pressed the : key");
                        app.input_mode = InputMode::Command;

                    }
                    Key::Char('q') => {
                        break;
                    }
                    _ => {}
                },
                InputMode::Command => match input {
                    Key::Char('\n') => {
                        app.input.drain(..);
                    }
                    Key::Char(c) => {

                        app.input.push(c);
                    }
                    Key::Backspace => {
                        app.input.pop();
                    }
                    Key::Esc => {

                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                InputMode::Editing => {},
                    
            }
        }
    }
    Ok(())
}
