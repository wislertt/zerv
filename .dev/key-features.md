# Zerv Key Features

## Overview

Zerv is a dynamic versioning CLI that generates versions from VCS state with multi-format support and rich metadata. Unlike semantic-release (release automation) or npm version (manual bumping), zerv provides **continuous, automatic versioning** for every commit.

## Core Features

### 1. **Dynamic Version Generation from VCS State**

- Derives versions for **any commit** from VCS (Git, Mercurial, Darcs, SVN, etc.)
- Uses rich metadata: branch name, commit distance, dirty state, commit hash, timestamp
- Automatic pattern detection from tags (v1.2.3, 1.2.3, release-v1.2.3, etc.)
- Works with **current repository state** - no manual version bumping required

### 2. **Multi-Format Version Support**

- **Standard formats**: SemVer, PEP440, PVP with proper compliance
- **Template-based customization**: Jinja-like syntax for complete control
- **Built-in presets**: Common formats for different ecosystems and use cases
- **Format conversion**: Transform versions between different standards

### 3. **CI/CD & Build Automation**

- **Automatic versioning**: Every commit gets a unique, meaningful version
- **Version bumping**: Intelligent bumping based on repository state
- **Format conversion**: Use different formats for different purposes
- **Build metadata**: Rich context for identifying specific builds

### 4. **Cross-Ecosystem Compatibility**

- **Python**: PEP440 for PyPI packages (`1.2.3.post7.dev0+g29045e8`)
- **JavaScript/Node**: SemVer for npm packages (`1.2.3-post.7+g29045e8`)
- **Haskell**: PVP for Hackage packages (`1.2.3-post-7-g29045e8`)
- **Docker**: Custom formats for container tagging
- **Git**: Prefixed versions for git tags (`v1.2.3-post.7+g29045e8`)

### 5. **Smart State-Based Versioning**

- **Tier 1** (Tagged, clean): Clean release versions (`1.2.3`)
- **Tier 2** (Distance, clean): Post-release with build metadata (`1.2.3.post7+feature.auth.7.g29045e8`)
- **Tier 3** (Dirty): Development versions with timestamps (`1.2.3.post7.dev20231215+g29045e8`)

### 6. **Format Conversion Use Cases**

- **Git tagging**: Release with SemVer tags (`v1.2.3`)
- **Docker builds**: Use PVP format for container tags (`1.2.3-7-g29045e8`)
- **Python packages**: PEP440 for PyPI (`1.2.3.post7.dev0`)
- **Documentation**: Simple base versions (`1.2.3`)
- **Artifacts**: Rich metadata for traceability (`1.2.3+feature.auth.7.g29045e8`)

### 7. **Developer Experience**

- **Zero configuration**: Works out-of-the-box with sensible defaults
- **Flexible patterns**: Auto-detection + custom regex support
- **State overrides**: Manual control when needed
- **Validation**: Check version compliance against standards
- **Archive support**: Works with Git/Mercurial archives

## Complementary Tools

### Zerv + semantic-release: Perfect Together

Zerv is designed to be **complementary to semantic-release**, not competitive. They solve different problems:

- **semantic-release**: "Should I release? What official version should I release?" (Release decision making)
- **zerv**: "What version is this commit right now?" (Continuous version identification)

**Recommended workflow:**

1. Use **semantic-release** for official releases and git tagging
2. Use **zerv** for build versioning, artifacts, and development

```bash
# semantic-release creates official tags: v1.2.3
# zerv generates versions for any commit:
# - On tag v1.2.3: "1.2.3" (clean release)
# - 5 commits later: "1.2.3.post5+feature.auth.5.g29045e8" (development)
# - With changes: "1.2.3.post5.dev20231215+g29045e8.dirty" (work-in-progress)
```

### Tool Comparison

#### vs semantic-release (Complementary)

| Feature      | semantic-release    | zerv                                  | **Combined Use**                           |
| ------------ | ------------------- | ------------------------------------- | ------------------------------------------ |
| **Purpose**  | Release automation  | Version generation                    | Release decisions + Build versioning       |
| **Trigger**  | Manual/CI release   | Any commit                            | Official releases + Continuous builds      |
| **Formats**  | SemVer only         | SemVer, PEP440, PVP, custom           | SemVer tags + Multi-format builds          |
| **Metadata** | Basic prerelease    | Rich: distance, branch, dirty, commit | Clean releases + Rich development versions |
| **Use case** | "Should I release?" | "What version is this commit?"        | "When to release" + "How to version"       |

### vs npm version

| Feature      | npm version       | zerv                 |
| ------------ | ----------------- | -------------------- |
| **Method**   | Manual bumping    | Automatic from VCS   |
| **Scope**    | package.json only | Any project/language |
| **Metadata** | None              | Rich VCS metadata    |
| **Formats**  | SemVer only       | Multiple standards   |

### vs git describe

| Feature             | git describe        | zerv                          |
| ------------------- | ------------------- | ----------------------------- |
| **Output**          | `v1.2.3-7-g29045e8` | Multiple formats              |
| **Standards**       | Git-specific        | PEP440, SemVer, PVP compliant |
| **Customization**   | Limited             | Full template control         |
| **Dirty detection** | Basic `--dirty`     | Rich dirty state handling     |

## Real-World Examples

### CI/CD Pipeline

```bash
# Get version for current commit
VERSION=$(zerv --format semver)
docker build -t myapp:$VERSION .

# Different formats for different purposes
DOCKER_TAG=$(zerv --format pvp)           # 1.2.3-7-g29045e8
PYTHON_VERSION=$(zerv --format pep440)    # 1.2.3.post7.dev0+g29045e8
GIT_TAG=$(zerv --format zerv-git)         # v1.2.3-post.7+g29045e8
```

### Multi-Language Project

```bash
# Update package.json (Node.js)
npm version $(zerv --format base)

# Update setup.py (Python)
sed -i "s/version=.*/version='$(zerv --format pep440)'/" setup.py

# Update Cargo.toml (Rust)
sed -i "s/version = .*/version = \"$(zerv --format semver)\"/" Cargo.toml
```

### Zerv + semantic-release Workflow

```bash
# 1. semantic-release handles official releases
# Creates tags like v1.2.3 based on commit analysis
semantic-release

# 2. zerv handles all other versioning needs
# Build artifacts with rich metadata
DOCKER_TAG=$(zerv --format docker)        # 1.2.3-5-g29045e8
PYTHON_VERSION=$(zerv --format pep440)    # 1.2.3.post5.dev0+g29045e8
NPM_VERSION=$(zerv --format semver)       # 1.2.3-post.5+g29045e8

# Build with appropriate versions
docker build -t myapp:$DOCKER_TAG .
echo "version = '$PYTHON_VERSION'" > version.py
npm version $NPM_VERSION --no-git-tag-version
```

### Traditional Release Workflow (without semantic-release)

```bash
# Manual tagging approach
git tag v$(zerv --format base)

# Build Docker with rich metadata
docker build -t myapp:$(zerv --format docker) .

# Generate changelog entry
echo "Version $(zerv --format pep440) - $(date)" >> CHANGELOG.md
```

## Performance & Security

### Performance Benefits

- **Fast**: Rust implementation, 2-5x faster than Python equivalents
- **Minimal VCS calls**: Optimized command execution
- **Compiled patterns**: Pre-compiled regex for pattern matching
- **Zero dependencies**: Single binary, no runtime dependencies

### Security Features

- **No command injection**: Safe command execution
- **Input validation**: Strict pattern validation
- **Minimal attack surface**: Stateless operation, no network calls
- **Reproducible**: Same input always produces same output

## Integration Examples

### GitHub Actions with semantic-release

```yaml
# Official releases
- name: Release
  uses: cycjimmy/semantic-release-action@v4
  id: semantic
  env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

# Build versioning for all commits
- name: Get build version
  id: version
  run: |
      echo "docker=$(zerv --format docker)" >> $GITHUB_OUTPUT
      echo "python=$(zerv --format pep440)" >> $GITHUB_OUTPUT
      echo "semver=$(zerv --format semver)" >> $GITHUB_OUTPUT

- name: Build with versions
  run: |
      docker build -t myapp:${{ steps.version.outputs.docker }} .
      echo "Python version: ${{ steps.version.outputs.python }}"
```

### GitHub Actions (without semantic-release)

```yaml
- name: Get version
  id: version
  run: echo "version=$(zerv)" >> $GITHUB_OUTPUT

- name: Build with version
  run: docker build -t myapp:${{ steps.version.outputs.version }} .
```

### Makefile

```makefile
VERSION := $(shell zerv --format semver)
DOCKER_TAG := $(shell zerv --format docker)

build:
	docker build -t myapp:$(DOCKER_TAG) .

release:
	git tag v$(VERSION)
	git push origin v$(VERSION)
```

### Package Scripts

```json
{
    "scripts": {
        "version": "zerv --format base",
        "version:docker": "zerv --format docker",
        "version:python": "zerv --format pep440"
    }
}
```

This comprehensive feature set makes zerv a powerful tool for modern development workflows that require consistent, automatic versioning across multiple formats and ecosystems.
