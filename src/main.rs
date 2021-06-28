use std::{error::Error, io};

mod util;
mod database;

use util::event::{Event, Events};
use util::dbhandler::Database;
use termion::{event::Key, raw::IntoRawMode};
use termion::clear::*;
use tui::Terminal;
use tui::backend::TermionBackend;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};

use database::container;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let db = &Database {
        filename : String::from("./.database"),
        connection : sqlite::open("./.database").unwrap(),
    };

    let user = &database::user::get_user(1, db).unwrap().unwrap();
    println!("the user id is : {}", user.id);


    terminal.clear()?;
    loop {

        //call of the input event handler
        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('w') => {
                    container::create_workspace(user, "test", db)?;
                }
                Key::Char('i') => {
                    // container::create_collection(String::from("test"), db);
                }
                Key::Char('q') => {
                    break;
                }
                _ => {}
            }
        }
        
        //terminal.autoresize or not it seems to resize automatically anyway.
        terminal.autoresize()?;

        //render UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                    Constraint::Percentage(30),
                    Constraint::Percentage(100),
                    Constraint::Percentage(90)
                    ].as_ref()
                    )
                .split(f.size());
            let block = Block::default()
                .title("Collections")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            let block = Block::default()
                .title("Edit Request")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[1]);
        })?;
    }
    Ok(())
}
