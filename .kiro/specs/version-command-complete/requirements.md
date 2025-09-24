# Requirements Document

## Introduction

This specification defines the requirements for implementing a complete version command for Zerv that matches the ideal design outlined in `.dev/05-version-command-complete-spec.md`. The version command should support multiple input sources, comprehensive VCS overrides, enhanced error handling, and piping workflows with full data preservation through Zerv RON format.

## Requirements

### Requirement 1: Enhanced Source Support

**User Story:** As a developer, I want to use different input sources for version generation, so that I can integrate Zerv into complex workflows and pipelines.

#### Acceptance Criteria

1. WHEN I run `zerv version` without source flags THEN the system SHALL default to git source
2. WHEN I run `zerv version --source git` THEN the system SHALL extract version data from the git repository
3. WHEN I run `zerv version --source stdin` THEN the system SHALL read Zerv RON format from stdin
4. WHEN I provide simple version strings to stdin THEN the system SHALL reject them with helpful error message
5. WHEN I provide invalid RON format to stdin THEN the system SHALL provide clear parsing error messages
6. WHEN no stdin input is available with `--source stdin` THEN the system SHALL report "No input provided via stdin"

### Requirement 2: VCS Override Capabilities

**User Story:** As a CI/CD engineer, I want to override VCS-detected values for testing and simulation purposes, so that I can validate version generation under different scenarios.

#### Acceptance Criteria

1. WHEN I use `--tag-version <TAG>` THEN the system SHALL override the detected tag version
2. WHEN I use `--distance <NUM>` THEN the system SHALL override the calculated distance from tag
3. WHEN I use `--dirty <BOOL>` THEN the system SHALL override the detected dirty state
4. WHEN I use `--clean` THEN the system SHALL set distance=0 and dirty=false
5. WHEN I use `--current-branch <BRANCH>` THEN the system SHALL override the detected branch name
6. WHEN I use `--commit-hash <HASH>` THEN the system SHALL override the detected commit hash
7. WHEN I use conflicting flags like `--clean` with `--distance` THEN the system SHALL report a clear error

### Requirement 3: Enhanced Boolean Parsing

**User Story:** As a CLI user, I want flexible boolean input options, so that I can use natural language values for boolean flags.

#### Acceptance Criteria

1. WHEN I use `--dirty true` THEN the system SHALL accept it as true
2. WHEN I use `--dirty t`, `--dirty yes`, `--dirty y`, `--dirty 1`, or `--dirty on` THEN the system SHALL accept them as true (case-insensitive)
3. WHEN I use `--dirty false` THEN the system SHALL accept it as false
4. WHEN I use `--dirty f`, `--dirty no`, `--dirty n`, `--dirty 0`, or `--dirty off` THEN the system SHALL accept them as false (case-insensitive)
5. WHEN I provide invalid boolean values THEN the system SHALL report clear error with supported values

### Requirement 4: Comprehensive Error Handling

**User Story:** As a developer, I want clear and actionable error messages, so that I can quickly understand and resolve issues.

#### Acceptance Criteria

1. WHEN I'm not in a git repository THEN the system SHALL report "Not in a git repository (--source git)"
2. WHEN no tags are found THEN the system SHALL report "No version tags found in git repository"
3. WHEN no commits exist THEN the system SHALL report "No commits found in git repository"
4. WHEN git command is not found THEN the system SHALL report "Git command not found. Please install git and try again."
5. WHEN permission is denied THEN the system SHALL report "Permission denied accessing git repository"
6. WHEN unknown output format is used THEN the system SHALL list supported formats
7. WHEN shallow clone is detected THEN the system SHALL warn about inaccurate distance calculations

### Requirement 5: Zerv RON Piping Support

**User Story:** As a power user, I want to pipe Zerv RON format between commands, so that I can create complex transformation workflows with full data preservation.

#### Acceptance Criteria

1. WHEN I use `--output-format zerv` THEN the system SHALL output complete Zerv RON format with schema and vars
2. WHEN I pipe Zerv RON to `zerv version --source stdin` THEN the system SHALL parse and process it correctly
3. WHEN I chain multiple Zerv commands with RON format THEN all version data SHALL be preserved through the pipeline
4. WHEN I apply different schemas in a pipeline THEN the transformations SHALL work correctly
5. WHEN RON format is malformed THEN the system SHALL provide specific parsing error messages

### Requirement 6: Input Format Validation

**User Story:** As a user, I want clear validation of input formats, so that I understand what inputs are supported and how to use them correctly.

#### Acceptance Criteria

1. WHEN using `--source stdin` THEN the system SHALL only accept Zerv RON format
2. WHEN simple version strings are provided to stdin THEN the system SHALL suggest using `--tag-version` instead
3. WHEN PEP440 or SemVer strings are provided to stdin THEN the system SHALL reject them with clear guidance
4. WHEN RON structure is invalid THEN the system SHALL provide line and column error information
5. WHEN Zerv RON is missing required fields THEN the system SHALL report specific missing fields

### Requirement 7: Output Format Enhancement

**User Story:** As an integrator, I want consistent and clean output formats, so that I can reliably parse version information in scripts and tools.

#### Acceptance Criteria

1. WHEN I request any output format THEN the system SHALL produce single-line clean output
2. WHEN I use `--output-format pep440` THEN the system SHALL produce valid PEP440 format
3. WHEN I use `--output-format semver` THEN the system SHALL produce valid SemVer format
4. WHEN I use `--output-format zerv` THEN the system SHALL produce valid RON format
5. WHEN unknown format is requested THEN the system SHALL list all supported formats
6. WHEN output prefix is requested THEN the system SHALL apply it correctly

### Requirement 8: State-Based Version Tiers

**User Story:** As a release engineer, I want version output to reflect repository state accurately, so that I can distinguish between clean releases, development versions, and dirty builds.

#### Acceptance Criteria

1. WHEN on a tagged commit with clean working tree THEN the system SHALL output clean version (Tier 1)
2. WHEN commits exist after tag with clean working tree THEN the system SHALL include distance and branch info (Tier 2)
3. WHEN working tree is dirty THEN the system SHALL include development timestamp and dirty indicators (Tier 3)
4. WHEN overrides are used THEN the system SHALL respect the override values for tier calculation
5. WHEN `--clean` flag is used THEN the system SHALL force Tier 1 output regardless of actual state

### Requirement 9: Command Line Interface Consistency

**User Story:** As a CLI user, I want consistent command-line interface patterns, so that the tool behaves predictably and follows standard conventions.

#### Acceptance Criteria

1. WHEN I use `--help` THEN the system SHALL show comprehensive help with all options
2. WHEN I use invalid flag combinations THEN the system SHALL report specific conflicts
3. WHEN I use `--version` THEN the system SHALL show Zerv version information
4. WHEN command succeeds THEN the system SHALL exit with code 0
5. WHEN command fails THEN the system SHALL exit with code 1
6. WHEN I use short flags THEN they SHALL work equivalently to long flags where applicable

### Requirement 10: Performance and Reliability

**User Story:** As a CI/CD system, I want fast and reliable version generation, so that build pipelines remain efficient and stable.

#### Acceptance Criteria

1. WHEN processing version data THEN the system SHALL complete in under 100ms for typical repositories
2. WHEN git operations fail THEN the system SHALL retry appropriately or fail fast with clear errors
3. WHEN large repositories are processed THEN memory usage SHALL remain reasonable
4. WHEN concurrent executions occur THEN the system SHALL handle them safely without interference
5. WHEN network issues affect git operations THEN the system SHALL provide appropriate error messages
