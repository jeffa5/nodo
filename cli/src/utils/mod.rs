use log::debug;
use std::path::{Path, PathBuf};

pub fn find_nodo(p: &Path) -> PathBuf {
    let p = match p.extension() {
        None => {
            debug!("file didn't have an extension");
            p.with_extension("md")
        }
        Some(e) => {
            if e == "md" {
                debug!("file already had the md extension");
                p.to_owned()
            } else {
                debug!("found other extension: {:?}", e);
                p.with_extension("md")
            }
        }
    };

    p
}
