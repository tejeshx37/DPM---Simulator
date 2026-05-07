use std::path::PathBuf;
use std::env;

pub fn headers() -> PathBuf {
    if let Ok(dir) = env::var("BOOST_INCLUDE_DIR") {
        return PathBuf::from(dir);
    }

    if cfg!(target_os = "windows") {
        PathBuf::from("C:\\msys64\\mingw64\\include")
    } else if cfg!(target_os = "macos") {
        if std::path::Path::new("/opt/homebrew/include").exists() {
            PathBuf::from("/opt/homebrew/include")
        } else {
            PathBuf::from("/usr/local/include")
        }
    } else {
        PathBuf::from("/usr/include")
    }
}
