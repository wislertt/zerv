# BEST_PRACTICES.md

## CI/CD

### GitHub Actions Cache Key Pattern

**Use a consistent cache key pattern to avoid race conditions and ensure proper cache isolation:**

```
<name>-<workflow>-<job>-<os>-[<matrix>...]-[<dep>...]-<run_id>-<run_attempt>
```

**Cache Key Components:**

| Component       | Purpose                                         | Example                          |
| --------------- | ----------------------------------------------- | -------------------------------- |
| `<name>`        | Identifies what's cached (unique per job)       | `venv`, `deps`, `cargo`          |
| `<workflow>`    | Isolates caches per workflow                    | `${{ github.workflow }}`         |
| `<job>`         | Isolates caches per job                         | `${{ github.job }}`              |
| `<os>`          | Isolates by operating system                    | `${{ runner.os }}`               |
| `[<matrix>...]` | Isolates by matrix dimensions (optional)        | `py${{ matrix.python-version }}` |
| `[<dep>...]`    | Invalidates when dependencies change (optional) | `${{ hashFiles('uv.lock') }}`    |
| `<run_id>`      | Unique per workflow run                         | `${{ github.run_id }}`           |
| `<run_attempt>` | Unique per retry attempt                        | `${{ github.run_attempt }}`      |

### Single Cache Step (Fast Dependencies)

For dependencies that are quick to reinstall (Python venv, npm):

**Use step-level env vars to reduce repetition and keep lines short:**

```yaml
jobs:
    test:
        strategy:
            matrix:
                os: [ubuntu-latest, windows-latest, macos-latest]
        runs-on: ${{ matrix.os }}

        steps:
            - name: cache-venv
              env:
                  CACHE_KEY_PREFIX: venv-${{ github.workflow }}-${{ github.job }}-${{ runner.os }}-py${{ matrix.python-version }}
                  CACHE_KEY_DEPS: ${{ hashFiles('uv.lock') }}
              uses: actions/cache@cdf6c1fa76f9f475f3d7449005a359c84ca0f306 # v5.0.3
              with:
                  path: ${{ github.workspace }}/.venv
                  key: ${{ env.CACHE_KEY_PREFIX }}-${{ env.CACHE_KEY_DEPS }}-${{ github.run_id }}-${{ github.run_attempt }}
                  restore-keys: |
                      ${{ env.CACHE_KEY_PREFIX }}-${{ env.CACHE_KEY_DEPS }}-${{ github.run_id }}-
                      ${{ env.CACHE_KEY_PREFIX }}-${{ env.CACHE_KEY_DEPS }}-
                      ${{ env.CACHE_KEY_PREFIX }}-
```

### Combined Cache Step (Multiple Related Paths)

For related caches that should be invalidated together:

**Use `hashFiles()` with multiple files to get a single combined hash:**

```yaml
jobs:
    pre-commit:
        runs-on: ubuntu-latest

        steps:
            - name: cache-deps
              env:
                  CACHE_KEY_PREFIX: deps-${{ github.workflow }}-${{ github.job }}-${{ runner.os }}-py${{ inputs.python_version }}
                  CACHE_KEY_DEPS: ${{ hashFiles('uv.lock', '.pre-commit-config.yaml') }}
              uses: actions/cache@cdf6c1fa76f9f475f3d7449005a359c84ca0f306 # v5.0.3
              with:
                  path: |
                      ${{ github.workspace }}/.venv
                      ~/.cache/pre-commit
                      ~/.cache/ruff
                      ~/.bun/install/cache
                  key: ${{ env.CACHE_KEY_PREFIX }}-${{ env.CACHE_KEY_DEPS }}-${{ github.run_id }}-${{ github.run_attempt }}
                  restore-keys: |
                      ${{ env.CACHE_KEY_PREFIX }}-${{ env.CACHE_KEY_DEPS }}-${{ github.run_id }}-
                      ${{ env.CACHE_KEY_PREFIX }}-${{ env.CACHE_KEY_DEPS }}-
                      ${{ env.CACHE_KEY_PREFIX }}-${{ hashFiles('uv.lock') }}-
                      ${{ env.CACHE_KEY_PREFIX }}-
```

### Split Restore and Save for Expensive Caches

For caches that are **slow to create** (Rust compilation, large builds), use split restore/save steps to ensure cache is saved even if intermediate steps fail:

```yaml
jobs:
    build:
        runs-on: ${{ matrix.os }}
        env:
            CARGO_CACHE_PATH: |
                ~/.cargo/registry
                ~/.cargo/git
                target

        steps:
            # 1. Restore cache (fails gracefully if not found)
            - name: restore-cargo-cache
              id: restore-cargo
              env:
                  CACHE_KEY_PREFIX: cargo-${{ github.workflow }}-${{ github.job }}-${{ runner.os }}
              uses: actions/cache/restore@0c907a7517f239e4053e11f1aee0df0fd0823747 # v4.2.1
              with:
                  path: ${{ env.CARGO_CACHE_PATH }}
                  key: ${{ env.CACHE_KEY_PREFIX }}-${{ hashFiles('**/Cargo.lock') }}-${{ github.run_id }}-${{ github.run_attempt }}
                  restore-keys: |
                      ${{ env.CACHE_KEY_PREFIX }}-${{ hashFiles('**/Cargo.lock') }}-

            # 2. Build step (may fail, but cache will still be saved)
            - name: build
              run: cargo build --release

            # 3. Save cache (runs even if build fails)
            - name: save-cargo-cache
              if: always()
              uses: actions/cache/save@8c838cbe8e9c4b41d7be8ca7bcc388df19aa43b1 # v4.2.1
              with:
                  path: ${{ env.CARGO_CACHE_PATH }}
                  key: ${{ steps.restore-cargo.outputs.cache-primary-key }}
```

**When to use split restore/save:**

| Cache Type   | Use Split? | Reason                                           |
| ------------ | ---------- | ------------------------------------------------ |
| Rust cargo   | ✅ Yes     | Compilation is expensive - save partial progress |
| Large builds | ✅ Yes     | Don't waste hours of build time                  |
| Python venv  | ❌ No      | `uv sync` is fast enough                         |
| npm deps     | ❌ No      | `npm install` is relatively fast                 |

### Cache Step Placement

**Correct order for cache steps:**

```yaml
steps:
    # 1. ALWAYS FIRST - Need code to know what to cache
    - name: checkout
      uses: actions/checkout@...

    # 2. Tool setup with built-in caching
    - name: setup-uv
      uses: astral-sh/setup-uv@...
      with:
          enable-cache: true # Handles ~/.cache/uv

    # 3. YOUR cache - Restore if exists
    - name: cache-deps
      uses: actions/cache@...
      with:
          path: .venv
          key: ...

    # 4. Install - Uses cache if available, creates if not
    - name: install-dependencies
      run: uv sync --all-extras --all-groups --frozen
```

### Naming Convention

Keep step name and cache prefix consistent:

| Step Name     | Key Prefix |
| ------------- | ---------- |
| `cache-venv`  | `venv-`    |
| `cache-deps`  | `deps-`    |
| `cache-cargo` | `cargo-`   |

### Key points:

- Use **full commit SHA** (not `@v4`) for security - version tags can move
- Each cache in a job must have a unique `<name>` to avoid collisions
- `run_id` + `run_attempt` ensure cache can be saved after every run (no cross-run exact hits)
- `restore-keys` provide fallback to older caches by removing elements from the back
- Multiple caches per job are allowed and recommended for different purposes
- For split pattern: use `if: always()` on save step, reference `steps.<id>.outputs.cache-primary-key`
- Use **step-level `env:`** for cache key prefixes to access `runner.os` and `hashFiles()` while keeping lines short
- Use **job-level `env:`** only for truly static values (cache paths, fixed OS names)
- Use `hashFiles('file1', 'file2')` with **comma-separated files** to get a single combined hash (shorter cache keys)
- The `id:` field cannot use expressions - must be static strings
