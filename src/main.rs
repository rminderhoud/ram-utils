extern crate clap;
extern crate failure;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;

enum LetterCase {
    UpperCase,
    LowerCase,
}

fn main() {
    let path_arg = Arg::with_name("path")
        .help("File or directory path")
        .required(true)
        .index(1);

    let recursive_arg = Arg::with_name("recursive")
        .short("r")
        .help("Convert directories recursively");

    let ignore_files_arg = Arg::with_name("ignore-files")
        .long("ignore-files")
        .conflicts_with("ignore-dirs")
        .help("Ignore files during conversion");

    let ignore_dirs_arg = Arg::with_name("ignore-dirs")
        .long("ignore-dirs")
        .conflicts_with("ignore-files")
        .help("Ignore directories during conversion");

    let args = App::new("RAM Utils")
        .version("0.1")
        .author("Ralph Minderhoud <mail@ralphminderhoud.com>")
        .about("Simple utilities")
        .subcommand(
            SubCommand::with_name("upper")
                .about("Convert files and/or directories to upper case")
                .arg(&path_arg)
                .arg(&recursive_arg)
                .arg(&ignore_files_arg)
                .arg(&ignore_dirs_arg),
        )
        .subcommand(
            SubCommand::with_name("lower")
                .about("Convert files and/or directories to lower case")
                .arg(&path_arg)
                .arg(&recursive_arg)
                .arg(&ignore_files_arg)
                .arg(&ignore_dirs_arg),
        )
        .subcommand(
            SubCommand::with_name("unique_ext")
                .about("Find all unique extensions in this directory")
                .arg(&path_arg),
        )
        .get_matches();

    match args.subcommand() {
        ("upper", Some(sub_args)) => {
            convert_case_command(sub_args, LetterCase::UpperCase);
        }
        ("lower", Some(sub_args)) => {
            convert_case_command(sub_args, LetterCase::LowerCase);
        }
        ("unique_ext", Some(sub_args)) => {
            let path = Path::new(args.value_of("path").unwrap_or("."));
            find_unique_extensions_command(path);
        }
        _ => {}
    }
}

fn convert_case_command(args: &ArgMatches, case: LetterCase) {
    let path = Path::new(args.value_of("path").unwrap_or(""));

    if !path.exists() {
        eprintln!("File/Directory does not exist");
        return;
    }

    if path.is_file() {
        if let Err(e) = convert_file_or_dir(path, &case) {
            eprintln!("Error: {}", e);
            return;
        }
    }

    if path.is_dir() {
        if args.is_present("recursive") {
            if let Err(e) = convert_children(
                path,
                &case,
                args.is_present("ignore-files"),
                args.is_present("ignore-dirs"),
            ) {
                eprintln!("Error: {}", e);
            }
        }

        if let Err(e) = convert_file_or_dir(path, &case) {
            eprintln!("Error: {}", e);
            return;
        }
    }
}

fn convert_children(
    path: &Path,
    case: &LetterCase,
    ignore_files: bool,
    ignore_dirs: bool,
) -> Result<(), Error> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() && !ignore_dirs {
            convert_children(&entry.path(), case, ignore_files, ignore_dirs)?;
            convert_file_or_dir(&entry.path(), case)?;
        }

        if (file_type.is_file() || file_type.is_symlink()) && !ignore_files {
            convert_file_or_dir(&entry.path(), case)?;
        }
    }

    Ok(())
}

/// Converts the final component in a path to the specified letter case
///
/// E.g.
/// `/home/ralph/test/12345/abcd` => `/home/ralph/test/12345/ABCD`
/// `/foo/bar/baz.zip` => `/foo/bar/BAZ.ZIP`
fn convert_file_or_dir(path: &Path, case: &LetterCase) -> Result<(), Error> {
    let filename = path
        .file_name()
        .unwrap_or(OsStr::new(""))
        .to_str()
        .unwrap_or("");

    if filename.is_empty() {
        return Ok(());
    }

    let target_filename = match case {
        LetterCase::UpperCase => filename.to_uppercase(),
        LetterCase::LowerCase => filename.to_lowercase(),
    };

    let target_path = path
        .parent()
        .unwrap_or(Path::new("."))
        .join(target_filename);

    println!("Converting {:?} => {:?}", path, target_path);
    fs::rename(path, target_path)?;
    Ok(())
}

fn find_unique_extensions_command(path: &Path) {
    if !path.exists() || !path.is_dir() {
        eprintln!(
            "Directory does not exist or is not a valid directory path: {}",
            path.display()
        );
        return;
    }

    if let Ok(extensions) = find_unique_extensions(path) {
        let mut exts: Vec<&String> = extensions.keys().collect();
        exts.sort();
        for ext in exts {
            println!("{} ({} files)", ext, extensions[ext]);
        }
    } else {
        eprintln!("Failed to find unique extensions");
    }
}

fn find_unique_extensions(path: &Path) -> Result<HashMap<String, u32>, Error> {
    let mut res = HashMap::new();

    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            let child_entries = find_unique_extensions(&entry.path())?;
            for (ext, count) in child_entries.iter() {
                let c = res.entry(String::from(ext)).or_insert(0);
                *c += count;
            }
        }

        if file_type.is_file() || file_type.is_symlink() {
            if let Some(ext) = entry.path().extension() {
                let e = String::from(ext.to_str().unwrap());
                let count = res.entry(e).or_insert(0);
                *count += 1;
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    fn test_convert_file() {
        let lower_path = env::temp_dir().join("test.file");
        let upper_path = env::temp_dir().join("TEST.FILE");

        if lower_path.exists() {
            fs::remove_file(&lower_path).unwrap();
        }

        if upper_path.exists() {
            fs::remove_file(&upper_path).unwrap();
        }

        // -- Test to upper case
        let _f = File::create(&lower_path).unwrap();
        convert_file_or_dir(&lower_path, &LetterCase::UpperCase).unwrap();

        assert_eq!(upper_path.exists(), true);

        fs::remove_file(&upper_path).unwrap();

        // -- Test to lower case
        let _f = File::create(&upper_path).unwrap();
        convert_file_or_dir(&upper_path, &LetterCase::LowerCase).unwrap();

        assert_eq!(lower_path.exists(), true);

        fs::remove_file(&lower_path).unwrap();
    }

    #[test]
    fn test_convert_children() {
        let root = env::temp_dir().join("ram-utils-convert-test-convert-children");

        let mut lower_paths: Vec<PathBuf> = Vec::new();
        let mut upper_paths: Vec<PathBuf> = Vec::new();

        for name in ["one", "two", "three"].iter() {
            let lower_dir = root.join(name);
            let upper_dir = root.join(name.to_uppercase());

            let lower_file = lower_dir.with_extension("file");
            let upper_file = upper_dir.with_extension("FILE");

            lower_paths.push(lower_file);
            upper_paths.push(upper_file);

            lower_paths.push(lower_dir);
            upper_paths.push(upper_dir);
        }

        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }

        // -- Test to upper case
        fs::create_dir(&root).unwrap();

        for path in &lower_paths {
            if path.is_dir() {
                fs::create_dir(path).unwrap();
            } else {
                File::create(path).unwrap();
            }
        }

        convert_children(&root, &LetterCase::UpperCase, false, false).unwrap();

        for path in &upper_paths {
            assert_eq!(path.exists(), true);
        }

        fs::remove_dir_all(&root).unwrap();

        // -- Test to lower case
        fs::create_dir(&root).unwrap();

        for path in &upper_paths {
            if path.is_dir() {
                fs::create_dir(path).unwrap();
            } else {
                File::create(path).unwrap();
            }
        }

        convert_children(&root, &LetterCase::LowerCase, false, false).unwrap();

        for path in &lower_paths {
            assert_eq!(path.exists(), true);
        }

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_convert_children_ignores() {
        let root = env::temp_dir().join("ram-utils-convert-test-ignores");

        let lower_dir = root.join("test");
        let upper_dir = root.join("TEST");

        let lower_file = &lower_dir.with_extension("file");
        let upper_file = &upper_dir.with_extension("FILE");

        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }

        // -- Test ignore file
        fs::create_dir_all(&lower_dir).unwrap();
        fs::File::create(&lower_file).unwrap();

        convert_children(&root, &LetterCase::UpperCase, true, false).unwrap();

        assert_eq!(upper_dir.exists(), true);
        assert_eq!(lower_file.exists(), true);

        fs::remove_dir_all(&root).unwrap();

        // -- Test ignore directory
        fs::create_dir_all(&lower_dir).unwrap();
        fs::File::create(&lower_file).unwrap();

        convert_children(&root, &LetterCase::UpperCase, false, true).unwrap();

        assert_eq!(lower_dir.exists(), true);
        assert_eq!(upper_file.exists(), true);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_convert_dir_recursive() {
        let root = env::temp_dir().join("ram-utils-convert-test-recursive");
        let lower_file = root.join("test").join("bar").join("baz.file");
        let upper_file = root.join("TEST").join("BAR").join("BAZ.FILE");

        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }

        fs::create_dir_all(&lower_file.parent().unwrap()).unwrap();
        fs::File::create(&lower_file).unwrap();

        convert_children(&root, &LetterCase::UpperCase, false, false).unwrap();

        assert_eq!(upper_file.exists(), true);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_find_extensions() {
        let root = env::temp_dir().join("ram-utils-test-find-extensions");

        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }

        let extensions = ["foo", "bar", "baz123"];
        for ext in extensions {
            let mut filepath = root.join("testfile");
            filepath.set_extension(ext);
            fs::create_dir_all(&filepath.parent().unwrap()).unwrap();
            fs::File::create(&filepath).unwrap();
        }

        let exts = find_unique_extensions(&root).unwrap();
        for (ext, count) in exts.iter() {
            assert!(extensions.contains(&ext.as_str()));
            assert_eq!(*count, 1);
        }

        fs::remove_dir_all(&root).unwrap();
    }
}
