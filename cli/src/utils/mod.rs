pub mod git;
pub mod target;
pub mod user;

pub const fn is_hidden_dir(_name: &str) -> bool {
    false
}
