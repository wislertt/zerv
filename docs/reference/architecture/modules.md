# Module Architecture

## Key Modules

- **`src/vcs/`**: Version Control System abstraction (Git only) - extracts metadata
- **`src/version/`**: Version format implementations (PEP440, SemVer, Zerv)
- **`src/pipeline/`**: Data transformation layer
- **`src/schema/`**: Schema and preset management (RON-based)
- **`src/cli/`**: Command-line interface
- **`src/test_utils/`**: Shared testing utilities and infrastructure
