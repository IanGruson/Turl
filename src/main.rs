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
    widgets::{Widget, Block, Borders, Paragraph, Wrap, Tabs, List, ListState, ListItem},
    layout::{Layout, Constraint, Direction, Rect, Alignment},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans},
};

use unicode_width::UnicodeWidthStr;

use database::container::*;
use ui::view;

enum InputMode {
    Normal,
    Command,
    Editing,
}

enum SelectionMode {

    Collections,
    Requests,
}

struct App<'a> {
    input : String,
    input_mode : InputMode,
    selection_mode : SelectionMode,
    selected_tab : usize,
    tab_len : usize,
    col_state : ListState,
    req_state : ListState,
    collection_list :  Vec<ListItem<'a>>,
    request_list :  Vec<ListItem<'a>>,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            input : String::new(),
            input_mode : InputMode::Normal,
            selection_mode : SelectionMode::Collections,
            selected_tab : 0,
            tab_len : 0,
            col_state : ListState::default(),
            req_state : ListState::default(),
            collection_list : Vec::new(),
            request_list : Vec::new(),
        }
    }

}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    let events = Events::new();

    app.col_state.select(Some(0));
    app.req_state.select(Some(0));

    let db = &Database {
        filename : String::from("./.database"),
        connection : sqlite::open("./.database").unwrap(),
    };

    // This is here for tests mainly. 
    let user = &database::user::get_user(1, db).unwrap().unwrap();


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
                             Constraint::Percentage(10),
                             Constraint::Percentage(80),
                             Constraint::Percentage(10),
                ].as_ref()
                )
                .split(f.size());

            let workspaces = get_all_workspaces(1, db).unwrap();
            app.tab_len = workspaces.len();
            let workspace_spans = view::container_to_spans(workspaces);

            // tabs for Workspaces
            let tabs = Tabs::new(workspace_spans)
                .block(Block::default().title("Workspaces").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::White))
                .select(app.selected_tab)
                .divider(DOT);
            f.render_widget(tabs, chunks[0]);

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                    Constraint::Percentage(10),
                    Constraint::Percentage(90)
                    ].as_ref()
                    )
                .split(chunks[1]);


            let left_bar_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                    ].as_ref()
                    )
                .split(horizontal_chunks[0]);
            // fetch the collections corresponding to the workspace. 
            let mut collections = get_all_collections(app.selected_tab as i64 + 1 , db).unwrap();
            let mut collections_for_id = collections.clone();
            if collections.len() < 1 {
                collections.push(Collection::new(0, String::from("Empty")));
            }
            let collection_items = view::container_to_ListItem(collections);
            app.collection_list = collection_items.clone();

            // Render the collections of a workspace in a Widget::List
            let collection_list = List::new(app.collection_list.clone())
                .block(Block::default().title("Collections").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            f.render_stateful_widget(collection_list, left_bar_chunks[0],&mut app.col_state);

            // let mut collections_for_id = get_all_collections(app.selected_tab as i64 + 1, db).unwrap();
            if collections_for_id.len() < 1 {
                collections_for_id.push(Collection::new(0, String::from("Empty")));
            }
            let id_selected_col = collections_for_id[app.col_state.selected().unwrap()].id;

            //fetch all requests
            //TODO fix the id when getting all requests. 
            let requests = get_all_requests(id_selected_col, db).unwrap();
            let mut r = requests.clone();

            let request_items = view::request_to_ListItem(requests);
            app.request_list = request_items;
            let request_list = List::new(app.request_list.clone())
                .block(Block::default().title("Requests").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            f.render_stateful_widget(request_list, left_bar_chunks[1],&mut app.req_state);

            // render request method and name.
            if r.len() < 1 {
                r.push(Request::new(0,String::from("add Request"),Methods::GET,String::from("This collection is empty, add a request !"), String::from(" "), String::from(" ")));
            }
            let request = &r[app.req_state.selected().unwrap()];

            let request_paragraph = Paragraph::new(vec![
                                                   Spans::from(vec![Span::styled(request.method.to_string(), Style::default().add_modifier(Modifier::ITALIC)),
                                                   Span::styled("   ", Style::default()),
                                                   Span::styled(request.url.clone(), Style::default()),
                                                   ]),
            ])
                .block(Block::default()
                       .title("Edit Request")
                       .borders(Borders::ALL));
            f.render_widget(request_paragraph, horizontal_chunks[1]);

            let request_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                    Constraint::Percentage(20),
                    Constraint::Percentage(80),
                    ])
                .split(horizontal_chunks[1]);

            let block = Block::default()
                .title("Response")
                .borders(Borders::TOP);
            f.render_widget(block, request_chunks[1]);

            //input chunk (block ? I don't know)

            let input_chunk = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                    Constraint::Percentage(100),
                    ].as_ref()
                    )
                .split(chunks[2]);

            let input = Paragraph::new(app.input.as_ref())
                .style(match app.input_mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Command => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
            .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, input_chunk[0]);

            //Move cursor to the bottom of the page.
            match app.input_mode {
                InputMode::Normal => {}
                InputMode::Command => {
                    f.set_cursor(
                        // Place the cursor at the end of the input as you are 
                        // typing.
                        input_chunk[0].x + app.input.width() as u16 +1,
                        // Move one line down to leave a border.
                        input_chunk[0].y + 1, 
                        )
                }
                InputMode::Editing => {}

            };
        })?;

        //call of the input event handler
        if let Event::Input(input) = events.next()? {
            match app.input_mode {

                InputMode::Normal => match input {


                    //TODO 
                    //Get rid of this
                    Key::Char('w') => {
                        create_workspace(user, "test", db)?;
                    }
                    Key::Char('r') => {
                        // create_request(collection.name(), 1, String::from("GET"), String::from("http://localhost"),db)?;
                    }
                    Key::Char('i') => {
                        create_collection("test", 1, db)?;
                    }
                    //
                    
                    Key::Char(' ') => {
                        match app.selection_mode {
                            SelectionMode::Collections => {
                                app.selection_mode = SelectionMode::Requests;
                            }
                            SelectionMode::Requests => {
                                app.selection_mode = SelectionMode::Collections;
                            }
                        }

                    }
                    Key::Char(':') => {
                        app.input_mode = InputMode::Command;
                    }


                    // ---- Workspaces -----
                    Key::Char('1') => {
                        // Go to workspace 1
                        app.selected_tab = 0;
                    }
                    Key::Char('2') => {
                        // Go to workspace 2
                        app.selected_tab = 1;
                    }
                    Key::Char('3') => {
                        // Go to workspace 2
                        app.selected_tab = 2;
                    }
                    Key::Char('4') => {
                        // Go to workspace 2
                        app.selected_tab = 3;
                    }

                    Key::Right => {
                        app.col_state.select(Some(0));
                        if app.selected_tab < app.tab_len -1 {
                            app.selected_tab += 1;
                        }
                        else {
                            app.selected_tab = 0;
                        }
                    }
                    Key::Left => {
                        app.col_state.select(Some(0));
                        if app.selected_tab == 0 {
                            app.selected_tab = app.tab_len-1;
                        }
                        else {
                            app.selected_tab -= 1;
                        }
                    }

                    // ----- Collections & Requests ----
                    Key::Char('k') => {
                        match app.selection_mode {
                            SelectionMode::Requests => {
                                let i = app.req_state.selected().unwrap();
                                if i == 0 {
                                    app.req_state.select(Some(app.request_list.len() - 1));
                                }
                                else {
                                    app.req_state.select(Some(i -1));
                                }
                            }
                            SelectionMode::Collections => {
                                let i = app.col_state.selected().unwrap();
                                if i == 0 {
                                    app.col_state.select(Some(app.collection_list.len() - 1));
                                }
                                else {
                                    app.col_state.select(Some(i -1));
                                }
                            }
                        }
                    }
                    Key::Char('j') => {
                        match app.selection_mode {
                            SelectionMode::Requests => {
                                let i = app.req_state.selected().unwrap();
                                if i >= app.request_list.len() - 1 {
                                    app.req_state.select(Some(0));
                                }
                                else {
                                    app.req_state.select(Some(i +1));
                                }
                            }
                            SelectionMode::Collections => {
                                let i = app.col_state.selected().unwrap();
                                if i >= app.collection_list.len() - 1 {
                                    app.col_state.select(Some(0));
                                }
                                else {
                                    app.col_state.select(Some(i +1));
                                }
                            }
                        }
                    }
                    // Quit the application
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
                                                // let id = args[i+2].parse()?;
                                                create_collection(name, app.selected_tab as i64 + 1, db)?;

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
