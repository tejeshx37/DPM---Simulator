use std::path::PathBuf;
use std::env;

pub fn headers() -> PathBuf {
    if let Ok(dir) = env::var("BOOST_INCLUDE_DIR") {
        return PathBuf::from(dir);
    }

    if PathBuf::from("/opt/homebrew/include").exists() {
        return PathBuf::from("/opt/homebrew/include");
    }

    PathBuf::from("/usr/include")
}
