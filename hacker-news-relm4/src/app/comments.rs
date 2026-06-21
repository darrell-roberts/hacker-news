//! Comment model and factory component.
use friendly_duration::parse_friendly_age;
use hacker_news_search::api::Comment;
use relm4::{
    gtk::{
        TextBuffer, TextTag, TextTagTable,
        pango::{Style, Underline},
        prelude::{
            BoxExt as _, ButtonExt as _, OrientableExt as _, TextBufferExt as _, TextViewExt as _,
            WidgetExt as _,
        },
    },
    prelude::{FactoryComponent, *},
};

pub struct CommentModel {
    comment: Comment,
    text_buffer: TextBuffer,
    by_string: String,
    comment_count: String,
    age_string: String,
}

#[derive(Debug)]
pub enum CommentOutMsg {
    OpenComment(u64),
    // Close(u64),
}

#[relm4::factory(pub)]
impl FactoryComponent for CommentModel {
    type ParentWidget = gtk::Box;
    type CommandOutput = ();
    type Input = ();
    type Output = CommentOutMsg;
    type Init = Comment;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            add_css_class: "comment",

            gtk::TextView {
                set_editable: false,
                set_cursor_visible: false,
                set_wrap_mode: gtk::WrapMode::WordChar,
                add_css_class: "comment_text_view",

                set_buffer: Some(&self.text_buffer)
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 5,
                add_css_class: "by_author",

                gtk::Label {
                    set_label: &self.by_string
                },

                gtk::Button {
                    set_label: &self.comment_count,
                    set_visible: !self.comment.kids.is_empty(),
                    connect_clicked[sender, id = self.comment.id] => move |_| {
                        if let Err(err) = sender.output(CommentOutMsg::OpenComment(id)) {
                            eprintln!("Failed to send open comment message: {err:?}");
                        }
                    }
                },

                gtk::Label {
                    set_label: &self.age_string
                }
            }
        }
    }

    fn init_model(
        comment: Self::Init,
        _index: &Self::Index,
        _sender: relm4::FactorySender<Self>,
    ) -> Self {
        Self {
            text_buffer: create_text_view(&comment.body),
            by_string: format!("by: {}", comment.by),
            comment_count: format!("{} ", comment.kids.len()),
            age_string: parse_friendly_age(comment.time).unwrap_or_default(),
            comment,
        }
    }
}

/// Creates a [`TextBuffer`] for a [`TextView`].
///
/// <https://docs.gtk.org/gtk4/section-text-widget.html>
pub fn create_text_view(raw: &str) -> TextBuffer {
    let italic_tag = TextTag::builder()
        .name("italic")
        .style(Style::Italic)
        .build();
    let bold_tag = TextTag::builder().name("bold").weight(700).build();
    let code_tag = TextTag::builder().name("code").family("monospace").build();
    let link_tag = TextTag::builder()
        .name("link")
        .underline(Underline::Single)
        .build();

    let table = TextTagTable::new();
    table.add(&italic_tag);
    table.add(&bold_tag);
    table.add(&code_tag);
    table.add(&link_tag);

    let buffer = TextBuffer::new(Some(&table));

    for element in html_sanitizer::parse_elements(raw) {
        match element {
            html_sanitizer::Element::Text(s) => {
                buffer.insert(&mut buffer.end_iter(), s);
            }

            html_sanitizer::Element::Paragraph => {
                buffer.insert(&mut buffer.end_iter(), "\n\n");
            }

            html_sanitizer::Element::Escaped(c) => {
                buffer.insert(&mut buffer.end_iter(), &c.to_string());
            }

            html_sanitizer::Element::Code(s) => {
                let mark = buffer.create_mark(None, &buffer.end_iter(), true);

                buffer.insert(&mut buffer.end_iter(), &s);

                let start = buffer.iter_at_mark(&mark);
                let end = buffer.end_iter();

                buffer.apply_tag(&code_tag, &start, &end);
                buffer.delete_mark(&mark);
            }

            html_sanitizer::Element::Link(anchor) => {
                let mark = buffer.create_mark(None, &buffer.end_iter(), true);

                buffer.insert(&mut buffer.end_iter(), &anchor.children);

                let start = buffer.iter_at_mark(&mark);
                let end = buffer.end_iter();

                buffer.apply_tag(&link_tag, &start, &end);
                buffer.delete_mark(&mark);
            }

            html_sanitizer::Element::Italic(children) => {
                let mark = buffer.create_mark(None, &buffer.end_iter(), true);

                for child in children {
                    nested_element(&buffer, child);
                }

                let start = buffer.iter_at_mark(&mark);
                let end = buffer.end_iter();

                buffer.apply_tag(&italic_tag, &start, &end);
                buffer.delete_mark(&mark);
            }

            html_sanitizer::Element::Bold(children) => {
                let mark = buffer.create_mark(None, &buffer.end_iter(), true);

                for child in children {
                    nested_element(&buffer, child);
                }

                let start = buffer.iter_at_mark(&mark);
                let end = buffer.end_iter();

                buffer.apply_tag(&bold_tag, &start, &end);
                buffer.delete_mark(&mark);
            }
        }
    }

    buffer
}

fn nested_element(buffer: &TextBuffer, child: html_sanitizer::Element<'_>) {
    match child {
        html_sanitizer::Element::Text(s) => {
            buffer.insert(&mut buffer.end_iter(), s);
        }
        html_sanitizer::Element::Escaped(c) => {
            buffer.insert(&mut buffer.end_iter(), &c.to_string());
        }
        html_sanitizer::Element::Paragraph => {
            buffer.insert(&mut buffer.end_iter(), "\n\n");
        }
        html_sanitizer::Element::Code(_)
        | html_sanitizer::Element::Link(_)
        | html_sanitizer::Element::Italic(_)
        | html_sanitizer::Element::Bold(_) => {}
    }
}
