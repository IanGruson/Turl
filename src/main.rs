use std::{error::Error, io};

mod util;
mod database;
mod ui;

use util::event::{Event, Events};
use util::dbhandler::Database;
use termion::{event::Key, raw::IntoRawMode};
use termion::clear::*;
use tui::{
    Terminal,
    backend::TermionBackend,
    widgets::{Widget, Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction, Rect, Alignment},
    style::{Color, Modifier, Style},
};

use unicode_width::UnicodeWidthStr;

use database::container::*;
use ui::view;

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

    // This is here for tests mainly. 
    let user = &database::user::get_user(1, db).unwrap().unwrap();
    let collection = Collection::new(1, String::from("collection_test"));
    let workspace = Workspace::new(1, String::from("workspace"));


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

            let workspaces = get_all_workspaces(1, db).unwrap();
            let workspace_spans = &view::container_to_spans(workspaces);

            let collections =get_all_collections(1, db).unwrap();
            let collection_spans = &view::container_to_spans(collections);

            // let block = Block::default()
            //     .title("Collections")
            //     .borders(Borders::ALL);
            // f.render_widget(block, horizontal_chunks[0]);

            // renders a list of Workspaces in the left block
            let custom_list = Paragraph::new(workspace_spans.clone())
                .block(Block::default().title("Collections").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            f.render_widget(custom_list, horizontal_chunks[0]);

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
                        create_workspace(user, "test", db)?;
                    }
                    Key::Char('r') => {
                        // create_request(collection.name(), 1, String::from("GET"), String::from("http://localhost"),db)?;
                    }
                    Key::Char('i') => {
                        create_collection("test", 1, db)?;
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

                // Command line to add/delete stuff in the database
                InputMode::Command => match input {
                    // Enter key press
                    Key::Char('\n') => {
                        let v : Vec<&str> = app.input.split_whitespace().collect();
                        if let Some((&name, args)) = v.split_first() {
                            match name {
                                "add" => {
                                    for (i, &arg) in args.iter().enumerate() {
                                        match arg {
                                            "workspace" => {

                                                let name = args[i+1];
                                                create_workspace(user, name, db)?;
                                            }

                                            "collection" => {

                                                let name = args[i+1];
                                                let id = args[i+2].parse()?;
                                                create_collection(name, id, db)?;

                                            }

                                            "request" => {

                                                let name = args[i+1];
                                                let id = args[i+2].parse()?;
                                                let method = args[i+3];
                                                let url = args[i+4];
                                                create_request(name, id, method, url,db)?;
                                            }

                                            &_ => ()
                                        }
                                    }

                                },
                                "rm" => {
                                    for (i, &arg) in args.iter().enumerate() {
                                        match arg {
                                            "workspace" => {
                                                let name = args[i+1];
                                                delete_workspace(name, db)?;
                                            }

                                            "collection" => {
                                                let name = args[i+1];
                                                delete_collection(name, db)?;

                                            }

                                            "request" => {
                                                let name = args[i+1];
                                                delete_request(name, db)?;

                                            }

                                            &_ => ()
                                        }
                                    }
                                },
                                &_ => println!("command {} not found", name)

                            }
                        }
                        app.input.drain(..);
                        app.input_mode = InputMode::Normal;
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
