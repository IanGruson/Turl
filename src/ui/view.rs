use tui::{
    widgets::{ListItem},
    text::{Span, Spans, Text},
};

use crate::database::{container};

pub fn container_to_spans<T>(
    container : Vec<T>,
    ) -> Vec<Spans<'static>> 
where
T : container::Container
{

    let mut spans_vec = vec![];
    for item in container.iter() {
        spans_vec.push(Spans::from(Span::raw(item.name())));

    }

    spans_vec
}

pub fn container_to_ListItem<T>(
    container : Vec<T>,
    ) -> Vec<ListItem<'static>> 
where
T : container::Container,
{

    let mut list_items = vec![];
    for item in container.iter() {
        list_items.push(ListItem::new(item.name()));

    }

    list_items
}


pub fn request_to_ListItem<T>(
    request : Vec<T>,
    ) -> Vec<ListItem<'static>> 
where 
T : container::Protocol,
{

    let mut list_items = vec![];
    for item in request.iter() {
        list_items.push(ListItem::new(item.name()));

    }

    list_items
}
