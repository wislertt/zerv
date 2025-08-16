# Zerv: Universal Version Structure

## Overview

`Zerv` is the universal version representation that acts as a bridge between different version formats (PEP440, SemVer, CalVer, etc.). It uses a variable reference system where components can be literals or references to variables, providing maximum flexibility while maintaining semantic meaning.

## Core Structure

```rust
// Universal version representation combining format and data
#[derive(Debug, Clone, PartialEq)]
pub struct Zerv {
    pub format: ZervFormat,
    pub vars: ZervVars,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZervFormat {
    pub core: Vec<Component>,
    pub extra_core: Vec<Component>,
    pub build: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    String(String),
    Integer(u64),
    VarField(String),     // Direct ZervVars field (major, minor, patch, etc.)
    VarTimestamp(String), // Timestamp pattern derived from tag_timestamp (YYYY, YYYYMMDD, etc.)
    VarCustom(String),    // Custom variable from ZervVars.custom HashMap
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZervVars {
    // Semantic version components
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,

    // Git/VCS components
    pub tag_timestamp: Option<u64>,           // Unix timestamp from the latest version tag
    pub tag_branch: Option<String>,           // Branch from the latest version tag
    pub current_branch: Option<String>,       // Current branch of HEAD/commit
    pub distance: Option<u64>,
    pub dirty: Option<bool>,
    pub tag_commit_hash: Option<String>,      // Hash of the tag commit
    pub current_commit_hash: Option<String>,  // Hash of current commit

    // Extra components
    pub epoch: Option<u64>,                 // PEP440 epoch (2!1.2.3)
    pub pre_release: Option<PreReleaseVar>, // alpha.1, beta.2, rc.3
    pub post: Option<u64>,                  // PEP440 post-release (.post2)
    pub dev: Option<u64>,                   // PEP440 development (.dev3)

    // Custom variables (extensible)
    pub custom: std::collections::HashMap<String, VarValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreReleaseVar {
    pub label: String,  // "alpha", "beta", "rc"
    pub number: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarValue {
    String(String),
    Integer(u64),
    Boolean(bool),
}
```

## Design Principles

### 1. Unified Structure

- **Zerv**: Single struct containing both format template and data
- **Component vectors**: Define version structure (core, extra_core, build)
- **ZervVars**: Embedded data to populate the template
- Clean API while maintaining internal separation of concerns

### 2. Variable Reference System

- Components can be literals (`String`, `Integer`) or variable references
- Variables stored in `ZervVars` provide semantic context and reusability
- Format acts as template, data provides values

### 3. Format-Agnostic Components

- `core`: Core version components [major, minor, patch] or [YYYY, MM, DD, patch]
- `extra_core`: Extra core identifiers [alpha, 1, post, 2, dev, 3]
- `build`: Build metadata [tag_branch, current_branch, distance, commit_hash]

### 4. Extensible Variables

- Common patterns have dedicated fields (major, minor, timestamp, etc.)
- Custom variables in HashMap for format-specific needs
- Type-safe variable values (String, Integer, Boolean)

### 5. Semantic Preservation

- Variables maintain semantic meaning across format conversions
- Component order preserved from source format
- Lossless representation of version information

## Usage Examples

### SemVer: `1.2.3-alpha.1+build.123`

```rust
let zerv = Zerv {
    format: ZervFormat {
        core: vec![
            Component::VarField("major"),
            Component::VarField("minor"),
            Component::VarField("patch")
        ],
        extra_core: vec![
            Component::VarField("pre_release"),
        ],
        build: vec![
            Component::String("build"),
            Component::Integer(123)
        ],
    },
    vars: ZervVars {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: Some(PreReleaseVar {
            label: "alpha".to_string(),
            number: Some(1)
        }),
        ..Default::default()
    },
};

// Usage: zerv.serialize() -> "1.2.3-alpha.1+build.123"
```

### PEP440: `2!1.2.3a1.post2.dev3+local.meta`

```rust
let zerv = Zerv {
    format: ZervFormat {
        core: vec![
            Component::VarField("major"),
            Component::VarField("minor"),
            Component::VarField("patch")
        ],
        extra_core: vec![
            Component::VarField("pre_release"),
            Component::VarField("post"),
            Component::VarField("dev")
        ],
        build: vec![
            Component::String("local"),
            Component::String("meta")
        ],
    },
    vars: ZervVars {
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: Some(PreReleaseVar {
            label: "alpha".to_string(),
            number: Some(1)
        }),
        epoch: Some(2),
        post: Some(2),
        dev: Some(3),
        ..Default::default()
    },
};

// Usage: zerv.serialize() -> "2!1.2.3a1.post2.dev3+local.meta"
```

### CalVer: `2024.03.15.1-rc.2`

```rust
let zerv = Zerv {
    format: ZervFormat {
        core: vec![
            Component::VarTimestamp("YYYY"),
            Component::VarTimestamp("0M"),
            Component::VarTimestamp("DD"),
            Component::VarField("patch")
        ],
        extra_core: vec![
            Component::VarField("pre_release")
        ],
        build: vec![],
    },
    vars: ZervVars {
        patch: Some(1),
        pre_release: Some(PreReleaseVar {
            label: "rc".to_string(),
            number: Some(2)
        }),
        tag_timestamp: Some(1710547200), // Unix timestamp for 2024-03-15
        ..Default::default()
    },
};

// Usage: zerv.serialize() -> "2024.03.15.1-rc.2"
```

### CalVer Compact: `20240315.1-rc.2`

```rust
// Different format, same data as above
let mut zerv_compact = zerv.clone();
zerv_compact.format.core = vec![
    Component::VarTimestamp("YYYYMMDD"),
    Component::VarField("patch")
];

// Usage: zerv_compact.serialize() -> "20240315.1-rc.2"
```

### Custom Git-based: `1.2.main.5.abc123-dirty`

```rust
let zerv = Zerv {
    format: ZervFormat {
        core: vec![
            Component::VarField("major"),
            Component::VarField("minor"),
            Component::VarField("current_branch"),
            Component::VarField("distance"),
            Component::VarField("current_commit_hash")
        ],
        extra_core: vec![
            Component::String("dirty")
        ],
        build: vec![],
    },
    vars: ZervVars {
        major: Some(1),
        minor: Some(2),
        current_branch: Some("main".to_string()),
        distance: Some(5),
        current_commit_hash: Some("abc123".to_string()),
        dirty: Some(true),
        ..Default::default()
    },
};

// Usage: zerv.serialize() -> "1.2.main.5.abc123-dirty"
```

### Format Reusability Example

```rust
// Base data
let base_vars = ZervVars {
    major: Some(1),
    minor: Some(2),
    patch: Some(3),
    tag_timestamp: Some(1710547200),
    pre_release: Some(PreReleaseVar {
        label: "rc".to_string(),
        number: Some(1)
    }),
    ..Default::default()
};

// Different formats using same data
let semver = Zerv {
    format: ZervFormat {
        core: vec![VarField("major"), VarField("minor"), VarField("patch")],
        extra_core: vec![VarField("pre_release")],
        build: vec![],
    },
    vars: base_vars.clone(),
};
semver.serialize(); // "1.2.3-rc.1"

let calver = Zerv {
    format: ZervFormat {
        core: vec![VarTimestamp("YYYY"), VarTimestamp("0M"), VarTimestamp("DD"), VarField("patch")],
        extra_core: vec![VarField("pre_release")],
        build: vec![],
    },
    vars: base_vars.clone(),
};
calver.serialize(); // "2024.03.15.1-rc.1"

let compact = Zerv {
    format: ZervFormat {
        core: vec![VarTimestamp("YYYYMMDD"), VarField("patch")],
        extra_core: vec![VarField("pre_release")],
        build: vec![],
    },
    vars: base_vars.clone(),
};
compact.serialize(); // "20240315.1-rc.1"
```

## JSON Representation

### SemVer: `1.2.3-alpha.1+build.123`

**JSON:**

```json
{
    "format": {
        "core": [{ "VarField": "major" }, { "VarField": "minor" }, { "VarField": "patch" }],
        "extra_core": [{ "VarField": "pre_release" }],
        "build": [{ "String": "build" }, { "Integer": 123 }]
    },
    "vars": {
        "major": 1,
        "minor": 2,
        "patch": 3,
        "tag_timestamp": null,
        "tag_branch": null,
        "current_branch": null,
        "distance": null,
        "dirty": null,
        "tag_commit_hash": null,
        "current_commit_hash": null,
        "epoch": null,
        "pre_release": {
            "label": "alpha",
            "number": 1
        },
        "post": null,
        "dev": null,
        "custom": {}
    }
}
```

**RON:**

```ron
(
    format: (
        core: [VarField("major"), VarField("minor"), VarField("patch")],
        extra_core: [VarField("pre_release")],
        build: [String("build"), Integer(123)],
    ),
    vars: (
        major: Some(1),
        minor: Some(2),
        patch: Some(3),
        pre_release: Some((
            label: "alpha",
            number: Some(1)
        )),
        tag_timestamp: None,
        tag_branch: None,
        current_branch: None,
        distance: None,
        dirty: None,
        tag_commit_hash: None,
        current_commit_hash: None,
        epoch: None,
        post: None,
        dev: None,
        custom: {}
    )
)
```

### CalVer: `2024.03.15.1-rc.2`

```json
{
    "core": [
        { "VarTimestamp": "YYYY" },
        { "VarTimestamp": "0M" },
        { "VarTimestamp": "DD" },
        { "VarField": "patch" }
    ],
    "extra_core": [{ "VarField": "pre_release" }],
    "build": [],
    "vars": {
        "major": null,
        "minor": null,
        "patch": 1,
        "tag_timestamp": 1710547200,
        "tag_branch": null,
        "current_branch": null,
        "distance": null,
        "dirty": null,
        "tag_commit_hash": null,
        "current_commit_hash": null,
        "epoch": null,
        "pre_release": {
            "label": "rc",
            "number": 2
        },
        "post": null,
        "dev": null,
        "custom": {}
    }
}
```

## Conversion Strategy

See [zerv-conversions.md](./zerv-conversions.md) for detailed conversion logic between existing version objects (SemVerVersion, PEP440Version) and Zerv format.

## Component Resolution

When processing components:

1. **String/Integer**: Use literal value
2. **VarField**: Look up direct field in ZervVars:
    - `major`, `minor`, `patch` → use field value directly
    - `pre_release`, `post`, `dev` → use field value directly
    - `current_branch`, `distance`, etc. → use field value directly
3. **VarTimestamp**: Derive from `tag_timestamp` Unix timestamp:
    - Parse variable name as timestamp format pattern
    - Support any combination of timestamp components:
        - `YYYY` → "2024"
        - `YY` → "24"
        - `MM` → "03"
        - `0M` → "03" (zero-padded)
        - `DD` → "15"
        - `0D` → "15" (zero-padded)
        - `HH` → "14"
        - `YYYYMM` → "202403"
        - `YYMMDD` → "240315"
        - `YYYYMMDDHH` → "2024031514"
        - `YY0MDD` → "240315"
        - Any other combination
4. **VarCustom**: Look up in `custom` HashMap
    - Error if key not found in custom variables

## Benefits

1. **Clean Structure**: Separate component ordering from variable storage
2. **Variable Reuse**: Same variable can appear in multiple component lists
3. **Extensible**: Easy to add new variable types without changing core structure
4. **Type Safety**: Strongly typed variables with semantic meaning
5. **Format Agnostic**: Component lists can represent any version structure
6. **Efficient**: No duplication of variable values across components

## Future Extensions

- Additional timestamp formats (week-based, fiscal year, etc.)
- More VCS integration (tag distance, branch patterns)
- Custom variable validation rules
- Template-based component generation
- Version comparison and ordering logic
