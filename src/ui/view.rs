use tui::{
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
    println!("Converting to spans");
    for item in container.iter() {
        println!("workspace name = {}" , item.name());
        spans_vec.push(Spans::from(Span::raw(item.name())));

    }

    spans_vec
}
