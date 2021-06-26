use crate::util::dbhandler::*;

use sqlite::*;
use curl::easy::Easy;


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

pub struct Collection {
    name : String,
    queries : Vec<HttpQuery>,
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
    name : &str,
    db : &Database) {

    println!("creating new workspace");
    //This methods works and is not vulnerable to SQL injections because of the vec! I think.
    //Also doesn't work without a cursor and I don't get why.
    let mut statement = db.connection.prepare("INSERT INTO Workspace(name) VALUES (:name);").unwrap();
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned()))]);
    cursor.next();

    // This works but is vulnerable to SQL injections !!
    // db.connection.execute(format!("INSERT INTO Workspace(name) VALUES ('{}');", name))
    // db.connection.execute(statement)

}
    
/// Creates a Collection in the database.
///
/// params : 
/// name - the name of the collection that will be created in the database.
/// db - a database object.
pub fn create_collection(
    name : String,
    db : &Database) { 
    
    println!("creating new collection");
    //This methods works and is not vulnerable to SQL injections because of the vec! I think.
    //Also doesn't work without a cursor and I don't get why.
    let mut statement = db.connection.prepare("INSERT INTO Collection(name) VALUES (:name);").unwrap();
    let mut cursor = statement.into_cursor();
    cursor.bind_by_name(vec![(":name", Value::String(name.to_owned()))]);
    cursor.next();

}



    

