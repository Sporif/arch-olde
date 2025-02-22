use std::collections::BTreeSet;

use crate::cmd::*;
use crate::error::*;

/// Installed packages with available 'pname' and 'version' attributes.
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) struct Package {
    /// Full not-quite-'pname' + 'version' from package environment.
    pub(crate) name: String,
    /// 'version' attribute from package environment. Most trusted.
    pub(crate) version: String,
}

/// Returns list of all used packages in parsed form.
// TODO: add parameters like system expression.
pub(crate) fn get_packages() -> Result<BTreeSet<Package>, OldeError> {
    let pkgs = run_cmd(&["pacman", "-Q"])?;
    // 7zip 24.09-3
    // acl 2.3.2-1
    let pkgs_str = String::from_utf8_lossy(&pkgs);

    let mut r = BTreeSet::new();
    for line in pkgs_str.lines() {
        if let Some((n, ver)) = line.split_once(' ') {
            // Remove epoch
            let ver = match ver.split_once(':') {
                Some((_, ver)) => ver,
                None => ver,
            };
            r.insert(Package {
                name: n.to_string(),
                // Assume pkgrel is present
                version: ver.split_once('-').expect("pkgrel not found").0.to_string(),
            });
        }
    }

    // Misconfigured system, not an Arch Linux system?
    if r.is_empty() {
        return Err(OldeError::EmptyOutput(String::from("pacman -Q")));
    }

    Ok(r)
}
