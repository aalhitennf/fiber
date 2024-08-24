use std::any::Any;

use fiber::state::Viewable;
use fiber::{App, StateCtx, StyleCss};
use floem::views::{h_stack, text};
use floem::{IntoView, View, ViewId};

fn main() {
    App::from_path("./examples/list")
        .enable_logging()
        .handlers(vec![add_item()])
        .state(|state| {
            let items = (1..=5).into_iter().map(ListItem::new).collect::<Vec<_>>();
            state.insert_view("list_items", items);
        })
        .run();
}

#[fiber::task]
fn add_item(state: StateCtx) {
    state.update_view::<ListItem>("list_items", |items| {
        items.push(ListItem::new(items.len() + 1));
    })
}

#[derive(Clone)]
struct ListItem {
    id: ViewId,
    name: String,
    value: String,
}

impl ListItem {
    pub fn new(idx: usize) -> Self {
        ListItem {
            id: ViewId::new(),
            name: format!("Item #{idx}"),
            value: idx.to_string(),
        }
    }
}

impl View for ListItem {
    fn id(&self) -> ViewId {
        self.id
    }
}

impl Viewable for ListItem {
    fn into_anyview(&self) -> floem::AnyView {
        let name = text(&self.name);
        let value = text(&self.value);
        h_stack((name, value)).css(&["list-item"]).into_any()
    }
}
