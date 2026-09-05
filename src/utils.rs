use crate::config::Config;

pub fn is_admin(username: Option<&str>, config: &Config) -> bool {
    match username {
        Some(uname) => uname.eq_ignore_ascii_case(&config.admin_username),
        None => false,
    }
}