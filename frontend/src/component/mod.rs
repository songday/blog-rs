pub mod posts_list;
pub mod unauthorized;

pub use posts_list::PostsListComponent;
pub use unauthorized::Unauthorized;

pub(crate) fn blank_node() -> yew::Html {
    let div = gloo_utils::document().create_element("p").unwrap();
    div.set_inner_html("&nbsp;");
    yew::Html::VRef(div.into())
}