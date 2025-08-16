# Zerv Conversion System

## Overview

This document describes the conversion system between existing version objects (SemVerVersion, PEP440Version) and the Zerv universal format.

## Conversion Strategy

### From Existing Version Objects to Zerv

```rust
// SemVer -> Zerv
impl From<SemVerVersion> for Zerv {
    fn from(semver: SemVerVersion) -> Self {
        Zerv {
            format: ZervFormat {
                core: vec![
                    Component::VarField("major"),
                    Component::VarField("minor"),
                    Component::VarField("patch")
                ],
                extra_core: if semver.pre_release.is_some() {
                    vec![Component::VarField("pre_release")]
                } else { vec![] },
                build: semver.build.iter().map(|s| Component::String(s.clone())).collect(),
            },
            vars: ZervVars {
                major: Some(semver.major),
                minor: Some(semver.minor),
                patch: Some(semver.patch),
                pre_release: semver.pre_release.map(|pr| PreReleaseVar {
                    label: pr.label,
                    number: pr.number
                }),
                ..Default::default()
            },
        }
    }
}

// PEP440 -> Zerv
impl From<PEP440Version> for Zerv {
    fn from(pep440: PEP440Version) -> Self {
        let mut extra_core = vec![];
        if pep440.pre_release.is_some() {
            extra_core.push(Component::VarField("pre_release"));
        }
        if pep440.post.is_some() {
            extra_core.push(Component::VarField("post"));
        }
        if pep440.dev.is_some() {
            extra_core.push(Component::VarField("dev"));
        }

        Zerv {
            format: ZervFormat {
                core: vec![
                    Component::VarField("major"),
                    Component::VarField("minor"),
                    Component::VarField("patch")
                ],
                extra_core,
                build: pep440.local.iter().map(|s| Component::String(s.clone())).collect(),
            },
            vars: ZervVars {
                major: Some(pep440.major),
                minor: Some(pep440.minor),
                patch: Some(pep440.patch),
                epoch: pep440.epoch,
                pre_release: pep440.pre_release.map(|pr| PreReleaseVar {
                    label: pr.label,
                    number: pr.number
                }),
                post: pep440.post,
                dev: pep440.dev,
                ..Default::default()
            },
        }
    }
}
```

### From Zerv to Existing Version Objects

```rust
// Zerv -> SemVer
impl TryFrom<Zerv> for SemVerVersion {
    type Error = ConversionError;

    fn try_from(zerv: Zerv) -> Result<Self, Self::Error> {
        // Validate format matches SemVer structure
        if !is_semver_compatible(&zerv.format) {
            return Err(ConversionError::IncompatibleFormat);
        }

        Ok(SemVerVersion {
            major: zerv.vars.major.ok_or(ConversionError::MissingField("major"))?,
            minor: zerv.vars.minor.ok_or(ConversionError::MissingField("minor"))?,
            patch: zerv.vars.patch.ok_or(ConversionError::MissingField("patch"))?,
            pre_release: zerv.vars.pre_release.map(|pr| SemVerPreRelease {
                label: pr.label,
                number: pr.number
            }),
            build: extract_build_strings(&zerv.format.build, &zerv.vars)?,
        })
    }
}

// Zerv -> PEP440
impl TryFrom<Zerv> for PEP440Version {
    type Error = ConversionError;

    fn try_from(zerv: Zerv) -> Result<Self, Self::Error> {
        // Validate format matches PEP440 structure
        if !is_pep440_compatible(&zerv.format) {
            return Err(ConversionError::IncompatibleFormat);
        }

        Ok(PEP440Version {
            epoch: zerv.vars.epoch,
            major: zerv.vars.major.ok_or(ConversionError::MissingField("major"))?,
            minor: zerv.vars.minor.ok_or(ConversionError::MissingField("minor"))?,
            patch: zerv.vars.patch.ok_or(ConversionError::MissingField("patch"))?,
            pre_release: zerv.vars.pre_release.map(|pr| PEP440PreRelease {
                label: pr.label,
                number: pr.number
            }),
            post: zerv.vars.post,
            dev: zerv.vars.dev,
            local: extract_build_strings(&zerv.format.build, &zerv.vars)?,
        })
    }
}
```

## Helper Functions

```rust
fn is_semver_compatible(format: &ZervFormat) -> bool {
    // Check if format structure matches SemVer expectations
    format.core.len() == 3 &&
    matches!(format.core[0], Component::VarField(ref s) if s == "major") &&
    matches!(format.core[1], Component::VarField(ref s) if s == "minor") &&
    matches!(format.core[2], Component::VarField(ref s) if s == "patch")
}

fn is_pep440_compatible(format: &ZervFormat) -> bool {
    // Check if format structure matches PEP440 expectations
    format.core.len() >= 3 &&
    matches!(format.core[0], Component::VarField(ref s) if s == "major") &&
    matches!(format.core[1], Component::VarField(ref s) if s == "minor") &&
    matches!(format.core[2], Component::VarField(ref s) if s == "patch")
}

fn extract_build_strings(build_components: &[Component], vars: &ZervVars) -> Result<Vec<String>, ConversionError> {
    build_components.iter()
        .map(|comp| match comp {
            Component::String(s) => Ok(s.clone()),
            Component::Integer(i) => Ok(i.to_string()),
            Component::VarField(field) => resolve_var_field(field, vars),
            _ => Err(ConversionError::UnsupportedComponent)
        })
        .collect()
}

fn resolve_var_field(field: &str, vars: &ZervVars) -> Result<String, ConversionError> {
    match field {
        "major" => vars.major.map(|v| v.to_string()).ok_or(ConversionError::MissingField("major")),
        "minor" => vars.minor.map(|v| v.to_string()).ok_or(ConversionError::MissingField("minor")),
        "patch" => vars.patch.map(|v| v.to_string()).ok_or(ConversionError::MissingField("patch")),
        "current_branch" => vars.current_branch.clone().ok_or(ConversionError::MissingField("current_branch")),
        "distance" => vars.distance.map(|v| v.to_string()).ok_or(ConversionError::MissingField("distance")),
        "current_commit_hash" => vars.current_commit_hash.clone().ok_or(ConversionError::MissingField("current_commit_hash")),
        _ => Err(ConversionError::UnsupportedField(field.to_string()))
    }
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Incompatible format structure")]
    IncompatibleFormat,
    #[error("Missing required field: {0}")]
    MissingField(&'static str),
    #[error("Unsupported component type")]
    UnsupportedComponent,
    #[error("Unsupported field: {0}")]
    UnsupportedField(String),
}
```

## Usage Examples

```rust
// Convert existing version to Zerv
let semver = SemVerVersion::new(1, 2, 3);
let zerv: Zerv = semver.into();

// Convert Zerv back to existing version
let semver_back: SemVerVersion = zerv.try_into()?;

// Cross-format conversion via Zerv
let pep440 = PEP440Version::new(1, 2, 3);
let zerv: Zerv = pep440.into();
let semver: SemVerVersion = zerv.try_into()?;
```

## Benefits

1. **Lossless Conversion**: Preserves all semantic information
2. **Type Safety**: Compile-time guarantees with proper error handling
3. **Extensible**: Easy to add new version formats
4. **Interoperability**: Seamless conversion between different version systems
5. **Clean Separation**: Format structure separate from data values
