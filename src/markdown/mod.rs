pub mod parser;
pub mod renderers;
pub mod serializer;

pub use parser::{create_or_get_link_tag, parse_markdown_to_buffer, setup_text_buffer_tags};
pub use renderers::{
    render_attachment_widget, render_image_widget, render_table_widget,
    resize_all_images_in_buffer, TableData,
};
pub use serializer::serialize_buffer_to_markdown;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatTag {
    Heading1,
    Heading2,
    Bold,
    Italic,
    Monospace,
    Strikethrough,
    BulletList,
}

impl FormatTag {
    pub fn name(&self) -> &'static str {
        match self {
            FormatTag::Heading1 => "heading-1",
            FormatTag::Heading2 => "heading-2",
            FormatTag::Bold => "bold",
            FormatTag::Italic => "italic",
            FormatTag::Monospace => "monospace",
            FormatTag::Strikethrough => "strikethrough",
            FormatTag::BulletList => "bullet-list",
        }
    }
}
