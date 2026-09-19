use regex::Regex;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
//  HiTechCloud Versioning — SemVer, Range Resolution, Yanking
// ═══════════════════════════════════════════════════════════════

/// Parsed semantic version
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build: Option<String>,
}

impl SemVer {
    /// Parse a semver string (e.g., "1.2.3", "1.2.3-beta.1+build.123")
    pub fn parse(version: &str) -> anyhow::Result<Self> {
        let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9.]+))?(?:\+([a-zA-Z0-9.]+))?$")?;
        let caps = re
            .captures(version.trim())
            .ok_or_else(|| anyhow::anyhow!("Invalid semver: {}", version))?;

        Ok(SemVer {
            major: caps[1].parse()?,
            minor: caps[2].parse()?,
            patch: caps[3].parse()?,
            pre_release: caps.get(4).map(|m| m.as_str().to_string()),
            build: caps.get(5).map(|m| m.as_str().to_string()),
        })
    }

    /// Check if this version is a pre-release
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    /// Format as string
    pub fn to_version_string(&self) -> String {
        let mut s = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.pre_release {
            s.push_str(&format!("-{}", pre));
        }
        if let Some(ref build) = self.build {
            s.push_str(&format!("+{}", build));
        }
        s
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_version_string())
    }
}

/// Version range specifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionRange {
    /// Exact match: =1.2.3
    Exact(SemVer),
    /// Caret range: ^1.2.3 (allows >=1.2.3, <2.0.0)
    Caret(SemVer),
    /// Tilde range: ~1.2.3 (allows >=1.2.3, <1.3.0)
    Tilde(SemVer),
    /// Greater than or equal: >=1.2.3
    Gte(SemVer),
    /// Greater than: >1.2.3
    Gt(SemVer),
    /// Less than or equal: <=1.2.3
    Lte(SemVer),
    /// Less than: <1.2.3
    Lt(SemVer),
    /// Any version: *
    Any,
}

impl VersionRange {
    /// Parse a version range string
    pub fn parse(range: &str) -> anyhow::Result<Self> {
        let range = range.trim();

        if range == "*" {
            return Ok(VersionRange::Any);
        }

        if range.starts_with("^") {
            let v = SemVer::parse(&range[1..])?;
            return Ok(VersionRange::Caret(v));
        }

        if range.starts_with("~") {
            let v = SemVer::parse(&range[1..])?;
            return Ok(VersionRange::Tilde(v));
        }

        if range.starts_with(">=") {
            let v = SemVer::parse(&range[2..])?;
            return Ok(VersionRange::Gte(v));
        }

        if range.starts_with(">") && !range.starts_with(">=") {
            let v = SemVer::parse(&range[1..])?;
            return Ok(VersionRange::Gt(v));
        }

        if range.starts_with("<=") {
            let v = SemVer::parse(&range[2..])?;
            return Ok(VersionRange::Lte(v));
        }

        if range.starts_with("<") && !range.starts_with("<=") {
            let v = SemVer::parse(&range[1..])?;
            return Ok(VersionRange::Lt(v));
        }

        if range.starts_with("=") {
            let v = SemVer::parse(&range[1..])?;
            return Ok(VersionRange::Exact(v));
        }

        // Default: exact match
        let v = SemVer::parse(range)?;
        Ok(VersionRange::Exact(v))
    }

    /// Check if a version satisfies this range
    pub fn satisfies(&self, version: &SemVer) -> bool {
        match self {
            VersionRange::Any => true,
            VersionRange::Exact(v) => version == v,
            VersionRange::Caret(base) => {
                // ^1.2.3 allows >=1.2.3, <2.0.0
                // ^0.2.3 allows >=0.2.3, <0.3.0
                // ^0.0.3 allows >=0.0.3, <0.0.4
                version >= base && {
                    if base.major > 0 {
                        version.major == base.major
                    } else if base.minor > 0 {
                        version.major == 0 && version.minor == base.minor
                    } else {
                        version.major == 0 && version.minor == 0 && version.patch == base.patch
                    }
                }
            }
            VersionRange::Tilde(base) => {
                // ~1.2.3 allows >=1.2.3, <1.3.0
                version >= base && version.major == base.major && version.minor == base.minor
            }
            VersionRange::Gte(base) => version >= base,
            VersionRange::Gt(base) => version > base,
            VersionRange::Lte(base) => version <= base,
            VersionRange::Lt(base) => version < base,
        }
    }
}

impl std::fmt::Display for VersionRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionRange::Any => write!(f, "*"),
            VersionRange::Exact(v) => write!(f, "={}", v),
            VersionRange::Caret(v) => write!(f, "^{}", v),
            VersionRange::Tilde(v) => write!(f, "~{}", v),
            VersionRange::Gte(v) => write!(f, ">={}", v),
            VersionRange::Gt(v) => write!(f, ">{}", v),
            VersionRange::Lte(v) => write!(f, "<={}", v),
            VersionRange::Lt(v) => write!(f, "<{}", v),
        }
    }
}

/// Resolve the best matching version from a list
pub fn resolve_best_version(range: &VersionRange, versions: &[SemVer]) -> Option<SemVer> {
    let mut candidates: Vec<&SemVer> = versions
        .iter()
        .filter(|v| !v.is_pre_release() && range.satisfies(v))
        .collect();

    candidates.sort_by(|a, b| b.cmp(a)); // Sort descending (newest first)
    candidates.first().map(|v| (*v).clone())
}

/// Package version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageVersion {
    pub version: SemVer,
    pub yanked: bool,
    pub yanked_reason: Option<String>,
    pub deprecated: bool,
    pub deprecated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub changelog: Option<String>,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

impl PackageVersion {
    pub fn new(version: SemVer) -> Self {
        Self {
            version,
            yanked: false,
            yanked_reason: None,
            deprecated: false,
            deprecated_at: None,
            changelog: None,
            published_at: chrono::Utc::now(),
        }
    }

    /// Mark as yanked (withdrawn)
    pub fn yank(&mut self, reason: Option<String>) {
        self.yanked = true;
        self.yanked_reason = reason;
    }

    /// Mark as deprecated
    pub fn deprecate(&mut self) {
        self.deprecated = true;
        self.deprecated_at = Some(chrono::Utc::now());
    }

    /// Is this version installable?
    pub fn is_installable(&self) -> bool {
        !self.yanked
    }
}

/// Version store for a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionStore {
    pub package_name: String,
    pub versions: Vec<PackageVersion>,
}

impl VersionStore {
    pub fn new(package_name: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            versions: vec![],
        }
    }

    /// Add a new version
    pub fn add_version(&mut self, version: PackageVersion) -> anyhow::Result<()> {
        if self.versions.iter().any(|v| v.version == version.version) {
            return Err(anyhow::anyhow!(
                "Version {} already exists",
                version.version
            ));
        }
        self.versions.push(version);
        self.versions.sort_by(|a, b| b.version.cmp(&a.version));
        Ok(())
    }

    /// Get the latest version
    pub fn latest(&self) -> Option<&PackageVersion> {
        self.versions.iter().find(|v| v.is_installable())
    }

    /// Resolve a version range
    pub fn resolve(&self, range: &VersionRange) -> Option<&PackageVersion> {
        let installable: Vec<SemVer> = self
            .versions
            .iter()
            .filter(|v| v.is_installable())
            .map(|v| v.version.clone())
            .collect();

        resolve_best_version(range, &installable)
            .and_then(|v| self.versions.iter().find(|pv| pv.version == v))
    }

    /// Get all installable versions
    pub fn installable_versions(&self) -> Vec<&PackageVersion> {
        self.versions
            .iter()
            .filter(|v| v.is_installable())
            .collect()
    }

    /// Check if a mandatory changelog is present for new versions
    pub fn validate_changelog(&self, version: &PackageVersion) -> bool {
        // Require changelog for versions after 1.0.0
        if version.version.major >= 1 && version.changelog.is_none() {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.to_version_string(), "1.2.3");
    }

    #[test]
    fn test_parse_semver_prerelease() {
        let v = SemVer::parse("1.0.0-beta.1").unwrap();
        assert_eq!(v.pre_release, Some("beta.1".to_string()));
        assert!(v.is_pre_release());
    }

    #[test]
    fn test_caret_range() {
        let range = VersionRange::parse("^1.2.3").unwrap();
        assert!(range.satisfies(&SemVer::parse("1.2.3").unwrap()));
        assert!(range.satisfies(&SemVer::parse("1.9.9").unwrap()));
        assert!(!range.satisfies(&SemVer::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_tilde_range() {
        let range = VersionRange::parse("~1.2.3").unwrap();
        assert!(range.satisfies(&SemVer::parse("1.2.3").unwrap()));
        assert!(range.satisfies(&SemVer::parse("1.2.9").unwrap()));
        assert!(!range.satisfies(&SemVer::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_resolve_best_version() {
        let versions = vec![
            SemVer::parse("1.0.0").unwrap(),
            SemVer::parse("1.1.0").unwrap(),
            SemVer::parse("1.2.0").unwrap(),
            SemVer::parse("2.0.0").unwrap(),
        ];
        let range = VersionRange::parse("^1.0.0").unwrap();
        let best = resolve_best_version(&range, &versions).unwrap();
        assert_eq!(best, SemVer::parse("1.2.0").unwrap());
    }

    #[test]
    fn test_version_store() {
        let mut store = VersionStore::new("test-package");
        store
            .add_version(PackageVersion::new(SemVer::parse("1.0.0").unwrap()))
            .unwrap();
        store
            .add_version(PackageVersion::new(SemVer::parse("1.1.0").unwrap()))
            .unwrap();

        let latest = store.latest().unwrap();
        assert_eq!(latest.version, SemVer::parse("1.1.0").unwrap());

        let range = VersionRange::parse("^1.0.0").unwrap();
        let resolved = store.resolve(&range).unwrap();
        assert_eq!(resolved.version, SemVer::parse("1.1.0").unwrap());
    }
}
