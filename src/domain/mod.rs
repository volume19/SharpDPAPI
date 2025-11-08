// Domain module - core application logic

pub mod argument_parser;
pub mod command_collection;
pub mod info;
pub mod version;

pub use argument_parser::ArgumentParser;
pub use command_collection::CommandCollection;
pub use info::Info;
pub use version::VERSION;
