use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

/// Recursively copy all contents from `from` to `to`. Symlinks will be resolved and copied.
pub fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    if !from.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Directory {} does not exist", from.display()),
        ));
    }
    if !from.is_dir() {
        return Err(io::Error::other(
            format!("Path {} is not a directory", from.display()),
        ));
    }

    if to.is_dir() {
        fs::remove_dir_all(to)?;
    }

    fs::create_dir_all(to)?;

    fs::read_dir(from)?.try_for_each(|result| {
        let entry = result?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&entry.path(), &to.join(entry.file_name()))
        } else if file_type.is_file() {
            fs::copy(entry.path(), to.join(entry.file_name())).map(|_| ())
        } else if file_type.is_symlink() {
            let link_file = resolve_symlink(entry.path())?;
            assert!(!link_file.is_symlink());
            let file = from.join(link_file);
            if file.is_dir() {
                copy_dir(&file, &to.join(entry.file_name()))
            } else {
                fs::copy(file, to.join(entry.file_name())).map(|_| ())
            }
        } else {
            Ok(())
        }
    })
}

fn resolve_symlink(path: PathBuf) -> io::Result<PathBuf> {
    if path.is_symlink() {
        resolve_symlink(path.read_link()?)
    } else {
        Ok(path)
    }
}

pub fn run_with_args<P, I>(program: P, dir: &Path, args: I) -> io::Result<ExitStatus>
where
    P: AsRef<OsStr>,
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    Command::new(program)
        .current_dir(dir)
        .args(args)
        .spawn()?
        .wait()
}

pub fn run_script<I>(script_name: &str, dir: &Path, args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
{
    let (shell, arg, ext) = if cfg!(windows) {
        ("cmd", Some("/C"), "bat")
    } else {
        ("sh", None, "sh")
    };
    let mut args: VecDeque<OsString> = args.into_iter().map(|arg| OsString::from(&arg)).collect();
    args.push_front(format!("{script_name}.{ext}").into());
    if let Some(arg) = arg {
        args.push_front(arg.into());
    }
    run_with_args(shell, dir, args)
}
