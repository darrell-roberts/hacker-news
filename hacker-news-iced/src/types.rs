use hacker_news_api::Item;
use html_sanitizer::Element;
use std::rc::Rc;

pub struct ParsedItem<'a> {
    item: Rc<Item>,
    parsed_text: Rc<Vec<Element<'a>>>,
}

impl<'a> From<Item> for ParsedItem<'a> {
    fn from(item: Item) -> Self {
        let item = Rc::new(item);

        let item_ref = Rc::clone(&item);
        let text = item_ref.text.as_deref().map(Rc::new);
        let parsed_text = text
            .map(|s| html_sanitizer::parse_elements(*s))
            .map(Rc::new)
            .unwrap_or_default();

        Self { item, parsed_text }
    }
}
