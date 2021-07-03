use tui::{
    text::Span
};

use crate::database::{container};

fn container_to_span<T>(
    container : Vec<T>,
    ) -> Vec<Span<'static>> 
where
T : container::Container<'static>
{

    let mut spans_vec = vec![];
    for item in container.iter() {
        spans_vec.push(Span::raw(item.name()));

    }

    spans_vec
}
