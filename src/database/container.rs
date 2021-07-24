use std::str::FromStr;
use crate::util::dbhandler::*;

use std::fmt;

use sqlite::*;
use curl::easy::Easy;
use super::user::*;

#[derive(Debug, PartialEq)]
pub enum Methods {
    GET,
    POST,
    DELETE,
    MODIFY,
    PUT,
}

impl FromStr for Methods {
    type Err = ();
    fn from_str(input : &str) -> std::result::Result<Methods, Self::Err> {
        match input {
            "GET" => Ok(Methods::GET),
            "POST" => Ok(Methods::POST),
            "DELETE" => Ok(Methods::DELETE),
            "MODIFY" => Ok(Methods::MODIFY),
            "PUT" => Ok(Methods::PUT),
            _      => Err(()),
        }
    }
}

// Convert enum to String
impl fmt::Display for Methods {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Workspace {

    pub id : i64,
    pub name : String,
    pub collections : Vec<Collection>, 
}

pub struct Collection {
    pub id : i64,
    pub name : String,
    pub queries : Vec<Request>,
}

pub trait Container {

    fn new(id :i64, name : String) -> Self;
    fn name(&self) -> String;
}

impl Container for Workspace {
    fn new(id : i64, name : String) -> Workspace {
        Workspace {id : id, name : name, collections : vec![]}
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Container for Collection {
    fn new(id :i64, name : String) -> Self {
        Self {id : id, name : name, queries : vec![]}
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub trait Protocol {
    fn new(id : i64,
               name : String,
               method : Methods,
               url : String,
               params : String,
               body : String) -> Request; 

    fn name(&self) -> String;

}

pub struct Request {
    id : i64,
    pub name : String,
    pub method : Methods,
    pub url : String,
    pub params : String,
    pub body : String,
}

impl Protocol for Request {

    fn new(id : i64,
               name : String,
               method : Methods,
               url : String,
               params : String,
               body : String) -> Request {
        Request { id : id,
        name : name ,
        method : method,
        url : url,
        params : params,
        body : body}
    }

    fn name(&self) -> String {
        self.name.clone()
    }

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
    name : &str,
    workspace_id : i64,
    db : &Database) -> Result<()> { 

    let mut statement = db.connection.prepare("INSERT INTO Collection(name, id_workspace) VALUES (:name, :id_workspace);")?;
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":id_workspace", Value::Integer(workspace_id.into()))
    ])?;
    cursor.next()?;
    Ok(())

}

/// Creates a new HTTP Request for a Collection.
/// 
/// params : 
///
/// name - the name of the request. 
/// id_collection - the collection the request will be attached to. 
/// method - the HTTP method (GET, POST ...).
/// url - the url of the request. 
/// db - a database object.
pub fn create_request(
    name : &str,
    id_collection : i64,
    method : &str,
    url : &str,
    db : &Database) -> Result<()> {

    let mut statement = db.connection.prepare("INSERT INTO Request(name, id_collection, method, url) VALUES (:name, :id_collection, :method, :url);")?;
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":id_collection", Value::Integer(id_collection.into())),
    (":method", Value::String(method.into())),
    (":url", Value::String(url.into()))
    ])?;
    cursor.next()?;
    Ok(())
}

/// Delete a workspace from it's name.
///
/// * `name` - &str of the workspace to delete.
/// * `db` - Database to work on.
pub fn delete_workspace(
    name : &str,
    db : &Database,) -> Result<()> {

    let mut statement = db.connection.prepare("DELETE FROM User_Workspace 
                                              WHERE id_workspace IN (
                                                  SELECT id_workspace FROM User_Workspace uw
                                                  INNER JOIN Workspace w ON w.id = uw.id_workspace
                                                  WHERE w.name = :name)")?;
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":name", Value::String(name.into()))
    ])?;
    cursor.next()?;

    statement = db.connection.prepare("DELETE FROM Workspace WHERE name = :name;")?;
    cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":name", Value::String(name.into()))
    ])?;
    cursor.next()?;
    Ok(())

}

/// Delete a collection from it's name.
///
/// * `name` - &str of the collection to delete.
/// * `db` - Database to work on.
pub fn delete_collection(
    name : &str,
    db : &Database,) -> Result<()> {

    let mut statement = db.connection.prepare("DELETE FROM Collection WHERE name = :name;")?;
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":name", Value::String(name.into()))
    ])?;
    cursor.next()?;
    Ok(())

}

/// Delete a collection from it's name.
///
/// * `name` - &str of the collection to delete.
/// * `db` - Database to work on.
pub fn delete_request(
    name : &str,
    db : &Database,) -> Result<()> {

    let mut statement = db.connection.prepare("DELETE FROM Request WHERE name = :name;")?;
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned())),
    (":name", Value::String(name.into()))
    ])?;
    cursor.next()?;
    Ok(())

}


/// Fetches all workspaces from a user's id. 
///
/// Returns a Vec of Workspaces Struct wrapped in sqlite's Result.
pub fn get_all_workspaces(
    id_user : i64,
    db : &Database) -> Result<Vec<Workspace>> {

    let mut workspaces : Vec<Workspace> = vec![]; 
    let mut cursor = db.connection.prepare("SELECT * FROM Workspace w
                                           INNER JOIN User_Workspace uw ON uw.id_workspace = w.id
                                           AND uw.id_user = :id_user")
        .unwrap()
        .into_cursor();

    cursor.bind_by_name(vec![(":id_user", Value::Integer(id_user.into()))])?;
    while let Some(row) = cursor.next().unwrap() {
        let workspace = Workspace {
            id : row[0].as_integer().unwrap().to_owned(),
            name : row[1].as_string().unwrap().to_owned(),
            collections : vec![],
        };
        workspaces.push(workspace);
    }

    Ok(workspaces)
}

/// Fetches all Collections from a Workspace id
///
/// * `id_workspace` - a i64 id to the corresponding workspace
/// * `db` - Database to work on.
///Returns a Vec of Collections 
pub fn get_all_collections(
    id_workspace : i64,
    db : &Database) -> Result<Vec<Collection>> {

    let mut collections : Vec<Collection> = vec![];

    let mut cursor = db.connection.prepare("SELECT * FROM Collection 
                                           WHERE id_workspace = :id_workspace")
        .unwrap()
        .into_cursor();
    cursor.bind_by_name(vec![(":id_workspace", Value::Integer(id_workspace.into()))])?;

    while let Some(row) = cursor.next().unwrap() {
        let collection = Collection {
            id : row[0].as_integer().unwrap().to_owned(),
            name : row[1].as_string().unwrap().to_owned(),
            queries : vec![],
        };
        collections.push(collection);
    }
    Ok(collections)
}

/// Fetches a request from it's id 
///
/// * `id` - the i64 id of the request
/// * `db` - Database to work on.
pub fn get_request(
    id : i64,
    db : &Database) -> Result<Request> {
    let mut cursor = db.connection.prepare("SELECT * FROM Request 
                                           WHERE id = :id")
        .unwrap()
        .into_cursor();
    cursor.bind_by_name(vec![(":id", Value::Integer(id.into()))])?;


    match cursor.next().unwrap() {
        Some(row) => {

            let request = Request {

                id : row[0].as_integer().unwrap().to_owned(),
                name : row[1].as_string().unwrap().to_owned(),
                method : Methods::from_str(row[2].as_string().unwrap()).unwrap(),
                url : row[3].as_string().unwrap().to_owned(),
                params : row[4].as_string().unwrap_or_default().to_owned(),
                body : row[5].as_string().unwrap_or_default().to_owned(),
            };
            Ok(request)
        }
        None => Err(Error { 
            code : Some(1),
            message : Some(String::from("Something went wrong")),
        })
    }


}


/// Fetches all Requests from a Workspace id
///
/// * `id_collection` - a i64 id to the corresponding workspace
/// * `db` - Database to work on.
///Returns a Vec of Requests 
pub fn get_all_requests(
    id_collection : i64,
    db : &Database) -> Result<Vec<Request>> {

    let mut requests : Vec<Request> = vec![];

    let mut cursor = db.connection.prepare("SELECT * FROM Request 
                                           WHERE id_collection = :id_collection")
        .unwrap()
        .into_cursor();
    cursor.bind_by_name(vec![(":id_collection", Value::Integer(id_collection.into()))])?;

    while let Some(row) = cursor.next().unwrap() {
        let request = Request {
            id : row[0].as_integer().unwrap().to_owned(),
            name : row[1].as_string().unwrap().to_owned(),
            method : Methods::from_str(row[2].as_string().unwrap()).unwrap(),
            url : row[3].as_string().unwrap().to_owned(),
            params : row[4].as_string().unwrap_or_default().to_owned(),
            body : row[5].as_string().unwrap_or_default().to_owned(),
        };
        requests.push(request);
    }
    Ok(requests)
}
