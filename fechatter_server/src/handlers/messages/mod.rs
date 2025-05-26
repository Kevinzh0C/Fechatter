mod crud;
mod file;
mod search;
mod utils;

// Re-export handler functions
pub use crud::{list_messages_handler, send_message_handler};
pub use file::{file_handler, fix_file_storage_handler, upload_handler};
pub use search::search_messages;

// Re-export OpenAPI path structs
pub use crud::{__path_list_messages_handler, __path_send_message_handler};
pub use file::{__path_file_handler, __path_fix_file_storage_handler, __path_upload_handler};
pub use search::__path_search_messages;

// Re-export commonly used types
pub use utils::{process_uploaded_file, validate_filename};
