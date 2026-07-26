pub mod attachment;
pub mod image;
pub mod table;

pub use attachment::render_attachment_widget;
pub use image::{render_image_widget, resize_all_images_in_buffer};
pub use table::{render_table_widget, TableData};
