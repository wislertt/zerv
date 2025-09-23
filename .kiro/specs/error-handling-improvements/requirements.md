# Requirements Document

## Introduction

This feature improves error handling in the zerv CLI tool to provide clearer, more user-friendly error messages. Currently, users see generic technical errors like "VCS not found" and raw git command output, which makes it difficult to understand what went wrong and how to fix it. This improvement will provide source-specific, actionable error messages that help users quickly identify and resolve issues.

## Requirements

### Requirement 1

**User Story:** As a developer using zerv, I want to see clear error messages when no version tags are found, so that I understand the specific source that was checked and can take appropriate action.

#### Acceptance Criteria

1. WHEN zerv runs in a git repository with no tags THEN the system SHALL display "No version tags found in git repository"
2. WHEN zerv runs with --source git in a repository with no tags THEN the system SHALL reference "git repository" specifically in the error message
3. WHEN zerv encounters no tags THEN the system SHALL NOT display generic "IO error" or "VCS data" messages

### Requirement 2

**User Story:** As a developer using zerv, I want to see clear error messages when not in a git repository, so that I understand which source was attempted and can navigate to the correct directory.

#### Acceptance Criteria

1. WHEN zerv runs outside a git repository THEN the system SHALL display "Not in a git repository (--source git)"
2. WHEN zerv runs with --source git outside a repository THEN the system SHALL reference the specific source in the error message
3. WHEN zerv encounters no repository THEN the system SHALL NOT display generic "VCS not found" messages

### Requirement 3

**User Story:** As a developer using zerv, I want to see clear error messages when in an empty git repository, so that I understand the issue without seeing confusing git command output.

#### Acceptance Criteria

1. WHEN zerv runs in a git repository with no commits THEN the system SHALL display "No commits found in git repository"
2. WHEN git commands fail due to empty repository THEN the system SHALL NOT expose raw git error messages to the user
3. WHEN git HEAD is ambiguous due to no commits THEN the system SHALL translate this to a user-friendly message

### Requirement 4

**User Story:** As a developer using zerv, I want git command failures to be translated into understandable messages, so that I can resolve issues without needing to interpret technical git output.

#### Acceptance Criteria

1. WHEN git commands fail with common errors THEN the system SHALL translate them to user-friendly messages
2. WHEN git is not installed THEN the system SHALL display "Git command not found. Please install git and try again."
3. WHEN git commands fail with permission errors THEN the system SHALL display "Permission denied accessing git repository"
4. WHEN git commands fail with unknown errors THEN the system SHALL provide a generic but clear error message

### Requirement 5

**User Story:** As a developer using zerv, I want error messages to be consistent and source-aware, so that I can quickly understand which version control system was being used.

#### Acceptance Criteria

1. WHEN any VCS-related error occurs THEN the system SHALL include the specific source name in the error message
2. WHEN multiple sources might be supported in the future THEN the error system SHALL be extensible to other VCS types
3. WHEN errors are displayed THEN the system SHALL maintain consistent formatting and terminology
