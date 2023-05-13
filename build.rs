use std::error::Error;
use std::fs;
use std::fs::DirEntry;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let generated = "src/protogen";
    tonic_build::configure()
        .out_dir(generated)
        .build_server(true)
        .compile(
            &["proto/extension.proto"],
            &["proto", "proto/thirdparty"],
        )?;

    fix_generated_code(generated)?;

    Ok(())
}

fn fix_generated_code(root: &str) -> Result<(), Box<dyn Error>> {
    let entries = fs::read_dir(root)?
        .filter(|res| res.is_ok())
        .map(|res| res.unwrap());

    let workdir = root.parse()?;
    for entry in entries {
        handle_entry(entry, &workdir)?
    }
    add_mod_line("extension", &workdir)?;

    Ok(())
}

fn handle_entry(entry: DirEntry, root: &PathBuf) -> Result<(), Box<dyn Error>> {
    let metadata = entry.metadata()?;
    if metadata.is_file() {
        let filename = entry.file_name().into_string().unwrap();
        if filename.starts_with("k8s.io") && filename.ends_with(".rs") {
            let path = entry.path();
            handle_dir(filename, root, path)?;
        }
    }

    Ok(())
}

fn handle_dir(filename: String, workdir: &PathBuf, original: PathBuf) -> Result<(), Box<dyn Error>> {
    let split = filename.split_once('.');
    match split {
        None => {
            Ok(())
        }
        Some((first, rest)) => {
            add_mod_line(first, &workdir)?;
            let mod_dir = workdir.join(first);
            fs::create_dir_all(&mod_dir)?;
            if count_dots(rest) == 1 {
                handle_file(rest.to_string(), mod_dir, original)
            } else {
                handle_dir(rest.to_string(), &mod_dir, original)
            }
        }
    }
}

fn handle_file(filename: String, workdir: PathBuf, original: PathBuf) -> Result<(), Box<dyn Error>> {
    let split = filename.split_once('.');
    match split {
        None => {
            Ok(())
        }
        Some((first, _)) => {
            add_mod_line(first, &workdir)?;
            let targetpath = workdir.join(filename);
            fs::rename(original, targetpath).map_err(|e| e.into())
        }
    }
}

fn count_dots(name: &str) -> usize {
    name.split('.').count()-1
}

fn add_mod_line(mod_name: &str, workdir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mod_line = format!("pub mod {mod_name};\n");
    let mod_file = workdir.join("mod.rs");
    let contents = if mod_file.exists() && mod_file.is_file() {
        String::from_utf8_lossy(&fs::read(&mod_file)?).to_string()
    } else {
        String::from("")
    };
    if !contents.contains(&mod_line) {
        let mut file = fs::OpenOptions::new().append(true).create(true).open(mod_file)?;
        file.write(mod_line.as_bytes())?;
    }
    Ok(())
}