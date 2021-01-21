pub(crate) mod about;
pub(crate) mod blog;
pub(crate) mod contribute;
pub mod error;
pub(crate) mod header;
pub(crate) mod index;
pub(crate) mod user;

use web_sys::Node;
use yew::{html::Html, virtual_dom::VNode};

pub(crate) fn raw_html(tag: &str, raw_html: &str) -> Html {
    let div = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element(tag)
        .unwrap();
    div.set_inner_html(raw_html);
    let node = Node::from(div);
    VNode::VRef(node)
}
