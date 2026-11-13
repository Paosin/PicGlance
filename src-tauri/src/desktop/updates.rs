use semver::Version;

pub fn newer_tag(current: &str, tag: &str) -> Option<String> {
    let current = Version::parse(current.trim_start_matches('v')).ok()?;
    let release = Version::parse(tag.strip_prefix('v').unwrap_or(tag)).ok()?;
    (release.pre.is_empty() && release.cmp_precedence(&current).is_gt()).then(|| tag.to_owned())
}
