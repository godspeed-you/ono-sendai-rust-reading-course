//! Release packaging (spec §66): deterministic `.zip` and `.tar.gz` archives of the generated
//! site, plus `SHA256SUMS`. Entries are sorted, timestamps fixed and owners/permissions
//! normalised, so the same `dist/` always produces byte-identical archives.

use crate::snippet::hex;
use flate2::write::GzEncoder;
use flate2::{Compression, GzBuilder};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const PRODUCT: &str = "ono-sendai-rust-reading-course";

/// Fixed archive timestamp: `SOURCE_DATE_EPOCH` if set, else 1980-01-01 (the ZIP epoch).
pub fn archive_epoch() -> u64 {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(315_532_800)
}

pub fn base_name(version: &str) -> String {
    format!("{PRODUCT}-v{version}")
}

#[derive(Debug)]
pub struct Packaged {
    pub zip: PathBuf,
    pub tar_gz: PathBuf,
    pub sums: PathBuf,
}

/// Files and directories under `dist`, relative, sorted, `/`-separated.
fn entries(dist: &Path) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let mut stack = vec![dist.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d)? {
            let p = e?.path();
            let rel = p
                .strip_prefix(dist)
                .expect("inside dist")
                .to_string_lossy()
                .replace('\\', "/");
            if p.is_dir() {
                dirs.push(rel);
                stack.push(p);
            } else {
                files.push(rel);
            }
        }
    }
    dirs.sort();
    files.sort();
    Ok((dirs, files))
}

fn zip_time(epoch: u64) -> zip::DateTime {
    // Convert a Unix timestamp to a civil UTC date (days-from-civil inverse).
    let secs = epoch.max(315_532_800);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let y = if m <= 2 { y + 1 } else { y } as u16;
    zip::DateTime::from_date_and_time(
        y,
        m,
        d,
        (rem / 3600) as u8,
        ((rem % 3600) / 60) as u8,
        (rem % 60) as u8,
    )
    .unwrap_or_default()
}

pub fn write_zip(dist: &Path, out: &Path, base: &str, epoch: u64) -> Result<(), String> {
    let (dirs, files) = entries(dist).map_err(|e| e.to_string())?;
    let f = fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut zw = zip::ZipWriter::new(f);
    let t = zip_time(epoch);
    let dir_opts = zip::write::SimpleFileOptions::default()
        .last_modified_time(t)
        .unix_permissions(0o755);
    let file_opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(t)
        .unix_permissions(0o644);
    zw.add_directory(format!("{base}/"), dir_opts)
        .map_err(|e| e.to_string())?;
    for d in &dirs {
        zw.add_directory(format!("{base}/{d}/"), dir_opts)
            .map_err(|e| e.to_string())?;
    }
    for rel in &files {
        let data = fs::read(dist.join(rel)).map_err(|e| e.to_string())?;
        zw.start_file(format!("{base}/{rel}"), file_opts)
            .map_err(|e| e.to_string())?;
        zw.write_all(&data).map_err(|e| e.to_string())?;
    }
    zw.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn write_tar_gz(dist: &Path, out: &Path, base: &str, epoch: u64) -> Result<(), String> {
    let (dirs, files) = entries(dist).map_err(|e| e.to_string())?;
    let f = fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let gz: GzEncoder<fs::File> = GzBuilder::new().mtime(0).write(f, Compression::default());
    let mut tb = tar::Builder::new(gz);
    tb.mode(tar::HeaderMode::Deterministic);
    let header = |path: &str, size: u64, dir: bool| -> Result<tar::Header, String> {
        let mut h = tar::Header::new_ustar();
        h.set_path(path).map_err(|e| e.to_string())?;
        h.set_size(size);
        h.set_mode(if dir { 0o755 } else { 0o644 });
        h.set_mtime(epoch);
        h.set_uid(0);
        h.set_gid(0);
        h.set_entry_type(if dir {
            tar::EntryType::Directory
        } else {
            tar::EntryType::Regular
        });
        h.set_cksum();
        Ok(h)
    };
    let h = header(&format!("{base}/"), 0, true)?;
    tb.append(&h, std::io::empty()).map_err(|e| e.to_string())?;
    for d in &dirs {
        let h = header(&format!("{base}/{d}/"), 0, true)?;
        tb.append(&h, std::io::empty()).map_err(|e| e.to_string())?;
    }
    for rel in &files {
        let data = fs::read(dist.join(rel)).map_err(|e| e.to_string())?;
        let h = header(&format!("{base}/{rel}"), data.len() as u64, false)?;
        tb.append(&h, data.as_slice()).map_err(|e| e.to_string())?;
    }
    let gz = tb.into_inner().map_err(|e| e.to_string())?;
    gz.finish().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn sha256_file(p: &Path) -> Result<String, String> {
    let mut f = fs::File::open(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex(&h.finalize()))
}

/// Package `dist` into `out_dir`. The site must already have passed the site checks.
pub fn package(dist: &Path, out_dir: &Path, version: &str) -> Result<Packaged, String> {
    if !dist.join("index.html").is_file() {
        return Err(format!(
            "{} has no index.html; run `scripts/course build` first",
            dist.display()
        ));
    }
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let base = base_name(version);
    let epoch = archive_epoch();
    let zip = out_dir.join(format!("{base}.zip"));
    let tar_gz = out_dir.join(format!("{base}.tar.gz"));
    write_zip(dist, &zip, &base, epoch)?;
    write_tar_gz(dist, &tar_gz, &base, epoch)?;
    let sums = out_dir.join("SHA256SUMS");
    let mut text = String::new();
    for p in [&tar_gz, &zip] {
        let name = p.file_name().unwrap().to_string_lossy();
        text.push_str(&format!("{}  {name}\n", sha256_file(p)?));
    }
    fs::write(&sums, text).map_err(|e| e.to_string())?;
    verify(&zip, &tar_gz, &base)?;
    Ok(Packaged { zip, tar_gz, sums })
}

/// Check that both archives unpack into `<base>/index.html` and contain the same files.
pub fn verify(zip_path: &Path, tar_path: &Path, base: &str) -> Result<(), String> {
    let mut zip_names: Vec<String> = {
        let f = fs::File::open(zip_path).map_err(|e| e.to_string())?;
        let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
        (0..z.len())
            .map(|i| z.by_index(i).map(|e| e.name().to_string()))
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?
    };
    let mut tar_names: Vec<String> = {
        let f = fs::File::open(tar_path).map_err(|e| e.to_string())?;
        let mut a = tar::Archive::new(flate2::read::GzDecoder::new(f));
        let mut v = Vec::new();
        for e in a.entries().map_err(|e| e.to_string())? {
            let e = e.map_err(|e| e.to_string())?;
            let mut p = e
                .path()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();
            if e.header().entry_type().is_dir() && !p.ends_with('/') {
                p.push('/');
            }
            v.push(p);
        }
        v
    };
    zip_names.sort();
    tar_names.sort();
    let index = format!("{base}/index.html");
    if !zip_names.contains(&index) {
        return Err(format!("zip archive lacks {index}"));
    }
    if zip_names != tar_names {
        return Err("zip and tar.gz archives contain different entries".into());
    }
    if let Some(bad) = zip_names
        .iter()
        .find(|n| !n.starts_with(&format!("{base}/")))
    {
        return Err(format!(
            "archive entry `{bad}` is outside the top-level folder"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dist() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("index.html"), "<!doctype html>").unwrap();
        fs::create_dir_all(d.path().join("assets")).unwrap();
        fs::write(d.path().join("assets/course.css"), "body{}").unwrap();
        d
    }

    #[test]
    fn packages_are_deterministic_and_verified() {
        let d = dist();
        let o1 = tempfile::tempdir().unwrap();
        let o2 = tempfile::tempdir().unwrap();
        let p1 = package(d.path(), o1.path(), "1.2.3").unwrap();
        let p2 = package(d.path(), o2.path(), "1.2.3").unwrap();
        assert_eq!(fs::read(&p1.zip).unwrap(), fs::read(&p2.zip).unwrap());
        assert_eq!(fs::read(&p1.tar_gz).unwrap(), fs::read(&p2.tar_gz).unwrap());
        let sums = fs::read_to_string(&p1.sums).unwrap();
        assert!(sums.contains("  ono-sendai-rust-reading-course-v1.2.3.zip\n"));
        assert!(sums.contains("  ono-sendai-rust-reading-course-v1.2.3.tar.gz\n"));
        assert_eq!(sums.lines().count(), 2);
    }

    #[test]
    fn refuses_to_package_without_index() {
        let d = tempfile::tempdir().unwrap();
        let o = tempfile::tempdir().unwrap();
        assert!(package(d.path(), o.path(), "1.0.0").is_err());
    }

    #[test]
    fn zip_time_converts_epoch() {
        let t = zip_time(1_700_000_000); // 2023-11-14 22:13:20 UTC
        assert_eq!((t.year(), t.month(), t.day()), (2023, 11, 14));
        assert_eq!((t.hour(), t.minute(), t.second()), (22, 13, 20));
    }
}
