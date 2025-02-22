mod cmd;
mod error;
mod opts;
mod progress;

// package loading modules
mod installed;
mod repology;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::OldeError;
use crate::opts::Opts;
use crate::progress::TaskProgress;

use clap::Parser;

fn main() -> Result<(), OldeError> {
    let o = Opts::parse();
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .filter_level(o.verbose.log_level_filter())
        .init();

    let (r, i) = {
        let mut r: Result<BTreeSet<repology::Package>, OldeError> = Ok(BTreeSet::new());
        let mut i: Result<BTreeSet<installed::Package>, OldeError> = Ok(BTreeSet::new());

        // If an error occured in other (faster) threads then this
        // flag is raised to signal cancellation.
        let cancel_flag = &AtomicBool::new(false);
        let cancel = || {
            cancel_flag.store(true, Ordering::Relaxed);
        };
        let poll_cancel = || cancel_flag.load(Ordering::Relaxed);

        // The repology thread is slow to proceed as it is network-bound:
        std::thread::scope(|s| {
            s.spawn(|| {
                let mut p = TaskProgress::new("repology");
                r = repology::get_packages(&poll_cancel, o.full_repo);
                if r.is_err() {
                    cancel();
                    p.fail();
                }
            });
            s.spawn(|| {
                let mut p = TaskProgress::new("installed");
                i = installed::get_packages();
                if i.is_err() {
                    cancel();
                    p.fail();
                }
            });
        });

        (r, i)
    };
    eprintln!();

    // Report all encountered errors
    if r.is_err() || i.is_err() {
        let mut errs = Vec::new();
        if r.is_err() {
            errs.push(r.err().unwrap())
        }
        if i.is_err() {
            errs.push(i.err().unwrap())
        }

        return Err(OldeError::MultipleErrors(errs));
    }
    let (repology_ps, installed_ps) = (r?, i?);

    // Packages not found in Repology database. Usually a package rename.
    let mut missing_repology: Vec<&str> = Vec::new();

    let mut found_repology: BTreeMap<(&str, &str), (&Option<String>, &str)> = BTreeMap::new();

    // Map installed => repology.
    for lp in &installed_ps {
        let mut found = false;
        for rp in &repology_ps {
            if lp.name != rp.name {
                continue;
            }
            found = true;

            match found_repology.get_mut(&(&rp.repology_name as &str, lp.name.as_str())) {
                None => {
                    found_repology.insert((&rp.repology_name, &lp.name), (&rp.latest, &lp.version));
                }
                Some(_) => unreachable!("Duplicate package: {}", lp.name),
            }
        }
        if !found {
            missing_repology.push(&lp.name);
        }
    }

    eprintln!(
        "Number of packages found in repology: {}\n",
        found_repology.len()
    );

    let mut found_outdated: isize = 0;
    let mut found_nolatest: Vec<((&str, &str), &str)> = Vec::new();
    for (rn, (olv, v)) in &found_repology {
        match olv {
            Some(lv) => {
                // Do not print outdated versions if there is use of most recent package
                use vercomp::Version;
                let v = Version::from(*v).number;
                let lv = Version::from(lv.as_str()).number;
                if v >= lv {
                    continue;
                }
            }
            None => {
                found_nolatest.push((*rn, v));
                continue;
            }
        }
        println!(
            "repology {} {:?} | archlinux {} {:?}",
            rn.0,
            (*olv).clone().unwrap_or("<none>".to_string()),
            rn.1,
            v,
        );
        found_outdated += 1;
    }

    if found_outdated > 0 {
        let ratio: f64 = found_outdated as f64 * 100.0 / installed_ps.len() as f64;
        eprintln!(
            "{} of {} ({:.2}%) installed packages are outdated according to https://repology.org.",
            found_outdated,
            installed_ps.len(),
            ratio
        );
    }

    if !found_nolatest.is_empty() {
        eprintln!();
        for (rn, v) in &found_nolatest {
            println!("repology {} <none> | archlinux {} {:?}", rn.0, rn.1, v);
        }
        let ratio: f64 = found_nolatest.len() as f64 * 100.0 / installed_ps.len() as f64;
        eprintln!(
            "{} of {} ({:.2}%) installed packages have no latest version at https://repology.org.",
            found_nolatest.len(),
            installed_ps.len(),
            ratio
        );
    }

    // Only show the missing packages when the full repology is used,
    // as otherwise it simply shows the non-outdated packages
    if o.full_repo {
        missing_repology.sort();
        eprintln!();
        let mut out = format!(
            "Installed packages missing in repology list: {}",
            missing_repology.len()
        );
        if log::log_enabled!(log::Level::Debug) {
            out.push_str(&format!(" - {:?}", missing_repology));
        } else {
            out.push_str(&format!("\nAdd '--verbose' to get it's full list."));
        }
        eprintln!("{out}");
    };

    Ok(())
}
