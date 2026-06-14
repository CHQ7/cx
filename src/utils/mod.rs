// Utils module - shared utility functions
pub mod html;
pub mod format;
pub mod paths;

pub use html::{simplify_html, extract_text, clean_tracking};
pub use paths::{project_root, assets_dir, memory_dir, temp_dir};
pub use paths::{read_global_memory, write_global_memory, append_global_memory};
