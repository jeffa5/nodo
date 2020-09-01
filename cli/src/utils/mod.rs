pub mod git;
pub mod target;
pub mod user;

pub fn is_hidden_dir(name: &str) -> bool {
    name == "archive"
}
