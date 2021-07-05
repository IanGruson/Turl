use crate::util::dbhandler::*;

use sqlite::*;
use curl::easy::Easy;
use super::user::*;

pub trait Migration {
    fn add(&self) -> Result<()>;
    fn remove(&self) -> Result<()>; 
}

enum Methods {
    GET,
    POST,
    DELETE,
    MODIFY,
}

pub struct Workspace {

    name : String,
    collections : Vec<Collection>, 
}

pub struct Collection {
    name : String,
    queries : Vec<HttpQuery>,
}

pub trait Container {

    fn new(name : String) -> Self;
    fn name(&self) -> String;
}

impl Container for Workspace {
    fn new(name : String) -> Workspace {
        Workspace {name : name, collections : vec![]}
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct HttpQuery {
    method : Methods,
    url : String,
    port : String,
}

/// Creates a Workspace in the database.
///
/// params : 
/// name - the name of the workspace that will be created in the database.
/// db - a database object.
pub fn create_workspace(
    user : &User,
    name : &str,
    db : &Database) -> Result<()> {

    println!("creating new workspace");
    //This methods works and is not vulnerable to SQL injections because of the vec! I think.
    //Also doesn't work without a cursor and I don't get why.
    let mut statement = db.connection.prepare("INSERT INTO Workspace(name) VALUES (:name);").unwrap();
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned()))])?;
    cursor.next()?;

    // Need to get the id of the workspace that's just been created.
    // 
    statement = db.connection.prepare("SELECT rowid FROM Workspace WHERE name = :name;").unwrap();
    cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned()))])?;
    match cursor.next().unwrap() {
        Some(row) => {
            //get the id of the last created Workspace.
            let workspace_id = row[0].as_integer().unwrap();
            println!("the created workspace id is : {}", workspace_id);

            // Join table insert statement
            statement = db.connection.prepare("INSERT INTO User_Workspace(id_user, id_workspace) VALUES (:id_user, :id_workspace);")?;
            let mut cursor = statement.into_cursor();
            cursor.bind_by_name(vec![(":id_user" ,Value::Integer(user.id.into())),
            (":id_workspace", Value::Integer(workspace_id.into()))])?;
            cursor.next()?;
            Ok(())
        }
        None => Err(Error { 
            code : Some(1),
            message : Some(String::from("Something went wrong, User_Workspace entry not created.")),
        })
        
    }



    // This works but is vulnerable to SQL injections !!
    // db.connection.execute(format!("INSERT INTO Workspace(name) VALUES ('{}');", name))
    // db.connection.execute(statement)

}

/// Creates a Collection in the database.
///
/// params : 
/// name - the name of the collection that will be created in the database.
/// workspace_id : the workspace id this collection will be attached to.
/// db - a database object.
pub fn create_collection(
    name : String,
    workspace_id : i32,
    db : &Database) -> Result<()> { 

    println!("creating new collection");
    //This methods works and is not vulnerable to SQL injections because of the vec! I think.
    //Also doesn't work without a cursor and I don't get why.
    let mut statement = db.connection.prepare("INSERT INTO Collection(name, id_workspace) VALUES (:name, :id_workspace);")?;
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":id_workspace", Value::Integer(workspace_id.into()))
    ])?;
    cursor.next()?;
    Ok(())

}

/// Fetches all workspaces from a user's id. 
///
/// Returns a Vec of Workspaces Struct wrapped in sqlite's Result.
pub fn get_all_workspaces(
    user_id : i32,
    db : &Database) -> Result<Vec<Workspace>> {

    let mut workspaces : Vec<Workspace> = vec![]; 
    let mut cursor = db.connection.prepare("SELECT * FROM Workspace WHERE user_id = :user_id")
        .unwrap()
        .into_cursor();

    cursor.bind_by_name(vec![(":user_id", Value::Integer(user_id.into()))])?;
    while let Some(row) = cursor.next().unwrap() {
        let workspace = Workspace {
            name : row[0].as_string().unwrap().to_owned(),
            collections : vec![],
        };
        workspaces.push(workspace);
    }

    Ok(workspaces)
}
