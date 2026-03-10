# Plan: Lock Artifact Aggregation for Matrix Jobs

## Background & Context

### Related Repositories

- **zerv** (this repo): `/Users/wisl/Desktop/vault/personal-repo/zerv` - Contains shared reusable workflows including `shared-lock.yml` and `shared-unlock.yml`
- **glz-projects** (sibling repo): `/Users/wisl/Desktop/vault/abcs-repo/glz-projects` - Uses the shared lock workflows from zerv via `uses: wislertt/zerv/.github/workflows/shared-lock.yml@<ref>`

### Current Architecture

The `glz-projects` repository has a workflow `terraform-run-all.yml` that:

1. Calls `shared-lock.yml` with a **matrix strategy** to lock multiple environments (d, n, p) in parallel
2. Runs terraform jobs for each business unit and environment combination
3. Calls `shared-unlock.yml` to release the locks

Current workflow structure in `glz-projects/.github/workflows/terraform-run-all.yml`:

```yaml
jobs:
    lock:
        strategy:
            matrix:
                env: [d, n, p]
        uses: wislertt/zerv/.github/workflows/shared-lock.yml@main
        with:
            key: ${{ matrix.env }}
            # ... other params

    terraform:
        needs: lock
        if: always() && needs.lock.result != 'failure' # PROBLEM: This checks overall result, not per-env
        # ... terraform jobs
```

### The Problem: Fine-Grained Conditional Execution

**Requirement**: Each terraform matrix job should only run if its corresponding environment's lock succeeded.

For example:

- `example-bu-development` (env_code: d) should only run if `lock-d` succeeded
- `example-bu-nonproduction` (env_code: n) should only run if `lock-n` succeeded
- `example-bu-production` (env_code: p) should only run if `lock-p` succeeded

**Current limitation**: With reusable workflows + matrix, we can only check `needs.lock.result` which is the overall result, not per-environment results.

### GitHub Actions Limitation (Documented)

From [GitHub Docs: Reusing workflows](https://docs.github.com/en/actions/using-workflows/reusing-workflows):

> "If a reusable workflow that sets an output is executed with a matrix strategy, the output will be the output set by the last successful completing reusable workflow of the matrix which actually sets a value."

**What this means**:

- If a reusable workflow is called with `matrix: [d, n, p]`, only ONE output is returned
- The output comes from whichever matrix instance completes LAST
- We cannot access `needs.lock.outputs.d-status`, `needs.lock.outputs.n-status`, etc.
- Each matrix instance overwrites the same output name

**Contrast with regular matrix jobs**:
Regular (non-reusable) matrix jobs CAN generate multiple outputs that get combined:

```yaml
jobs:
    job1:
        outputs:
            output_1: ${{ steps.gen_output.outputs.output_1 }}
            output_2: ${{ steps.gen_output.outputs.output_2 }}
            output_3: ${{ steps.gen_output.outputs.output_3 }}
        strategy:
            matrix:
                version: [1, 2, 3]
        steps:
            - run: echo "output_${{ matrix.version }}=${{ matrix.version }}" >> $GITHUB_OUTPUT
```

This works because each instance generates a **uniquely-named** output.

### Why Not Other Approaches?

| Approach                                                 | Why It Doesn't Work                                      |
| -------------------------------------------------------- | -------------------------------------------------------- |
| Access matrix outputs by name                            | Reusable workflows with matrix only return one output    |
| Hardcode d-status, n-status, p-status in shared-lock.yml | Not generic - defeats purpose of reusable workflow       |
| Use separate lock jobs                                   | Loses matrix benefits - more verbose, harder to maintain |
| Check lock results inside terraform job                  | Too late - job already started, wastes resources         |

### The Solution: Artifact Aggregation

**Key insight**: While reusable workflow outputs are limited, **artifacts work independently** for each matrix instance.

**Flow**:

```
┌─────────────────────────────────────────────────────────────┐
│ lock (matrix: d, n, p)                                       │
│  ├─ lock-d → uploads lock-result-d.json                     │
│  ├─ lock-n → uploads lock-result-n.json                     │
│  └─ lock-p → uploads lock-result-p.json                     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ aggregate-lock-results                                      │
│  ├─ Downloads all lock-result-*.json artifacts             │
│  ├─ Combines into: {"d":"success","n":"skipped","p":"failure"} │
│  └─ Sets output: lock-status = {...}                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ dummy-job (matrix: d, n, p) - Test only                    │
│  if: fromJson(needs.aggregate-lock-results.outputs.lock-status)[matrix.env] != 'failure' │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ unlock (matrix: d, n, p)                                    │
└─────────────────────────────────────────────────────────────┘

Note: In production (glz-projects), dummy-job is replaced by terraform jobs
with matrix.include combining subdir + env_code
```

**Benefits**:

- ✅ Generic: `shared-lock.yml` doesn't need to know about specific environments
- ✅ Works with reusable workflows: Uses artifacts, not outputs
- ✅ Fine-grained control: Each terraform job checks only its relevant environment
- ✅ Debuggable: Artifacts can be inspected individually

## Problem Statement

When using a reusable workflow with a matrix strategy in GitHub Actions, only ONE output is returned (from the last completing instance). This makes it impossible to get per-environment lock results when calling `shared-lock.yml` with a matrix.

**Goal**: Enable per-environment lock status access by using artifacts to aggregate results.

## Solution Overview

1. Update `shared-lock.yml` to upload an artifact with lock result
2. Create a dummy caller workflow to test the artifact-based aggregation
3. Add an aggregate job that downloads all artifacts and creates a combined JSON output
4. Verify the approach works before using in production workflows

---

## Technical Concepts Reference

### GitHub Actions Artifacts

- **Purpose**: Share files between jobs in a workflow or store files after workflow completion
- **Upload**: `actions/upload-artifact@v4` - each artifact has a unique name
- **Download**: `actions/download-artifact@v4` - can download by name or pattern
- **Key advantage**: Each matrix instance can upload with a unique name; all are preserved

### GitHub Actions Outputs

- **Job outputs**: Defined in `jobs.<job_id>.outputs`, must be strings
- **Reusable workflow outputs**: Defined in `on.workflow_call.outputs`, map to job outputs
- **Limitation**: With matrix strategy in reusable workflows, only one output value is returned

### GitHub Actions Matrix Strategy

- **Purpose**: Run multiple jobs with different configurations in parallel
- **Syntax**: `strategy.matrix` with lists of values
- **Each instance**: Gets access to `matrix.*` context with current values

### GitHub Actions Expressions Used

- `fromJson()`: Parse JSON string to object
- `format()`: String formatting
- `always()`: Returns true regardless of previous job status
- `needs.<job_id>.outputs.<name>`: Access output from dependent job

### GitHub Lock Action

- **Repository**: `actions/lock@v3.0.1`
- **Purpose**: Provide mutual exclusion for deployments using GitHub's built-in environment locks
- **Modes**: `check` (query status), `lock` (acquire), `unlock` (release)
- **Key format**: Environment names become lock keys

---

## Phase 1: Update `shared-lock.yml`

### File: `.github/workflows/shared-lock.yml`

#### Add artifact upload step

After the final status step, add a new step to create and upload a JSON artifact:

```yaml
- name: create-lock-result-json
  if: always()
  run: |
      # Determine final status
      STATUS="${{ steps.lock.outcome }}"
      if [ "$STATUS" = "success" ]; then
        FINAL_STATUS="success"
      elif [ "$STATUS" = "skipped" ] || [ "${{ inputs.enabled }}" != "true" ]; then
        FINAL_STATUS="skipped"
      else
        FINAL_STATUS="failure"
      fi

      # Create JSON file with lock result
      cat > lock-result.json << EOF
      {
        "key": "${{ inputs.key }}",
        "full_key": "${{ inputs.key_prefix }}${{ inputs.key }}",
        "status": "$FINAL_STATUS",
        "outcome": "$STATUS",
        "enabled": ${{ inputs.enabled }},
        "owner": "${{ inputs.key_owner }}"
      }
      EOF

      echo "Lock result:"
      cat lock-result.json

- name: upload-lock-result-artifact
  uses: actions/upload-artifact@v4
  if: always()
  with:
      name: lock-result-${{ inputs.key }}
      path: lock-result.json
      retention-days: 1
```

**Where to add**: After the `final-lock-status` step, before the `outputs` section.

**Why**: This ensures each matrix instance uploads its result as a uniquely-named artifact (`lock-result-d`, `lock-result-n`, `lock-result-p`).

---

## Phase 2: Create Dummy Caller Workflow

### File: `.github/workflows/test-lock-aggregation.yml`

Create a new workflow that simulates `terraform-run-all.yml` but with minimal steps:

```yaml
name: test-lock-aggregation

on:
    workflow_dispatch:
        inputs:
            deploy_labels:
                description: 'JSON object with deploy flags (e.g., {"deploy-d": true, "deploy-n": true})'
                required: false
                type: string
                default: '{"deploy-d": true, "deploy-n": true}'
            lock_key_owner:
                description: "Key owner for lock"
                required: false
                type: string
                default: "test-run"

permissions:
    contents: write

jobs:
    # ====================================================
    # Phase 1: Lock with matrix
    # ====================================================
    lock:
        if: ${{ inputs.deploy_labels != '' && inputs.deploy_labels != '{}' }}
        strategy:
            fail-fast: false
            matrix:
                env: [d, n, p]
        uses: ./.github/workflows/shared-lock.yml@main
        with:
            enabled: ${{ fromJson(inputs.deploy_labels || '{}')[format('deploy-{0}', matrix.env)] != null }}
            key: ${{ matrix.env }}
            key_prefix: test/
            key_owner: ${{ inputs.lock_key_owner }}
            job_name: lock-${{ matrix.env }}

    # ====================================================
    # Phase 2: Aggregate lock results
    # ====================================================
    aggregate-lock-results:
        needs: lock
        if: always()
        runs-on: ubuntu-latest
        steps:
            - name: download-all-lock-artifacts
              uses: actions/download-artifact@v4
              with:
                  pattern: lock-result-*
                  path: lock-results

            - name: list-downloaded-files
              run: |
                  echo "=== Downloaded files ==="
                  find lock-results -type f -name "*.json" -exec echo {} \;

            - name: display-individual-results
              run: |
                  echo "=== Individual lock results ==="
                  for file in lock-results/lock-result-*/lock-result.json; do
                    if [ -f "$file" ]; then
                      echo "---"
                      cat "$file"
                    fi
                  done

            - name: aggregate-results
              id: aggregate
              run: |
                  # Combine all lock results into single JSON
                  echo '{' > lock-status.json

                  first=true
                  for file in lock-results/lock-result-*/lock-result.json; do
                    if [ -f "$file" ]; then
                      if [ "$first" = true ]; then
                        first=false
                      else
                        echo ',' >> lock-status.json
                      fi

                      # Extract key and status from individual file
                      key=$(jq -r '.key' "$file")
                      status=$(jq -r '.status' "$file")

                      echo "Processing: key=$key, status=$status"

                      # Append to combined JSON
                      echo -n "\"$key\": \"$status\"" >> lock-status.json
                    fi
                  done

                  echo '}' >> lock-status.json

                  echo ""
                  echo "=== Aggregated lock status ==="
                  cat lock-status.json

                  # Validate JSON
                  if jq empty lock-status.json 2>/dev/null; then
                    echo "✓ JSON is valid"
                  else
                    echo "✗ JSON is invalid"
                    exit 1
                  fi

            - name: upload-aggregated-status
              uses: actions/upload-artifact@v4
              with:
                  name: lock-status-aggregated
                  path: lock-status.json
                  retention-days: 1

            - name: set-output
              id: output
              run: |
                  CONTENT=$(cat lock-status.json)
                  echo "status=$CONTENT" >> $GITHUB_OUTPUT

        outputs:
            lock-status: ${{ steps.output.outputs.status }}

    # ====================================================
    # Phase 3: Dummy jobs that check lock status (MATRIX VERSION)
    # ====================================================
    # Note: This matches the pattern used in terraform-run-all.yml where
    # jobs use matrix.env to dynamically check their specific environment's status
    dummy-job:
        needs: [lock, aggregate-lock-results]
        if: |
            always() &&
            fromJson(needs.aggregate-lock-results.outputs.lock-status)[matrix.env] != 'failure'
        strategy:
            fail-fast: false
            matrix:
                env: [d, n, p]
        runs-on: ubuntu-latest
        steps:
            - name: check-status
              run: |
                  LOCK_STATUS='${{ needs.aggregate-lock-results.outputs.lock-status }}'
                  ENV="${{ matrix.env }}"
                  echo "Lock status: $LOCK_STATUS"
                  ENV_STATUS=$(echo "$LOCK_STATUS" | jq -r --arg env "$ENV" '.[$env]')
                  echo "Environment $ENV lock status: $ENV_STATUS"
                  if [ "$ENV_STATUS" = "success" ] || [ "$ENV_STATUS" = "skipped" ]; then
                      echo "✓ Can proceed with $ENV environment"
                  else
                      echo "✗ Cannot proceed with $ENV environment"
                      exit 1
                  fi

    # ====================================================
    # Phase 4: Unlock after dummy jobs
    # ====================================================
    unlock:
        if: always()
        strategy:
            fail-fast: false
            matrix:
                env: [d, n, p]
        uses: ./.github/workflows/shared-unlock.yml@main
        needs:
            - lock
            - dummy-job
            - aggregate-lock-results
        with:
            enabled: ${{ fromJson(inputs.deploy_labels)[format('deploy-{0}', matrix.env)] != null }}
            key: ${{ matrix.env }}
            key_prefix: test/
            key_owner: ${{ inputs.lock_key_owner }}
            job_name: unlock-${{ matrix.env }}
```

---

## Phase 3: Implementation Steps

### Step 1: Update `shared-lock.yml`

1. Open `.github/workflows/shared-lock.yml`
2. Add the artifact upload steps (see Phase 1)
3. Commit and push changes

### Step 2: Create test workflow

1. Create `.github/workflows/test-lock-aggregation.yml` with content from Phase 2
2. Commit and push

### Step 3: Run initial test

1. Go to Actions tab in GitHub
2. Select "test-lock-aggregation" workflow
3. Click "Run workflow"
4. Use default inputs: `{"deploy-d": true, "deploy-n": true}`
5. Observe results

### Step 4: Verify artifact uploads

1. Check that lock artifacts are uploaded:
    - `lock-result-d`
    - `lock-result-n`
    - `lock-result-p` (should be skipped since not in deploy_labels)
2. Download and inspect artifact contents

### Step 5: Verify aggregation

1. Check `aggregate-lock-results` job logs
2. Verify combined JSON format: `{"d":"success","n":"success","p":"skipped"}`

### Step 6: Verify conditional execution

1. Check that `dummy-job` matrix instances ran for d and n (since they're in deploy_labels)
2. Check that `dummy-job` matrix instance for p was skipped (not in deploy_labels)
3. Verify each matrix instance checks only its own environment's status

---

## Phase 4: Test Scenarios

### Scenario 1: All environments enabled

```json
{ "deploy-d": true, "deploy-n": true, "deploy-p": true }
```

**Expected**: All lock jobs succeed, all 3 `dummy-job` matrix instances run

### Scenario 2: Only development enabled

```json
{ "deploy-d": true }
```

**Expected**: Only d locks, n and p matrix instances are skipped in aggregation

### Scenario 3: Simulate lock conflict

1. Manually acquire lock for environment `d` using GitHub API or UI
2. Run workflow
3. **Expected**: `dummy-job` matrix instance for d should be skipped due to lock failure

### Scenario 4: Empty deploy_labels

```json
{}
```

**Expected**: Lock jobs are skipped, `dummy-job` matrix instances should handle empty/missing status

---

## Phase 5: Debugging Checklist

If something doesn't work:

- [ ] Check if `shared-lock.yml` artifact upload step runs
- [ ] Verify artifact names match pattern `lock-result-*`
- [ ] Check if `download-artifact` pattern matches uploaded artifacts
- [ ] Verify JSON is valid (use `jq` in logs)
- [ ] Check matrix job names in Actions UI
- [ ] Verify `needs` dependencies are correct
- [ ] Check if `fromJson()` is parsing correctly
- [ ] Look for any error in workflow logs

---

## Phase 6: Success Criteria

The implementation is successful when:

1. ✅ Each lock instance uploads its result as an artifact
2. ✅ The aggregate job downloads all artifacts successfully
3. ✅ The aggregated JSON contains all environment statuses
4. ✅ Dummy-job (matrix) uses `fromJson(needs.aggregate-lock-results.outputs.lock-status)[matrix.env]` successfully
5. ✅ Each matrix instance of dummy-job runs conditionally based on its environment's lock status
6. ✅ Unlock jobs run after dummy-job matrix completes

---

## Phase 7: Apply to Production

Once verified in zerv:

1. Update `glz-projects` repo to pull updated `shared-lock.yml`
2. Update `terraform-run-all.yml` with aggregate job and conditional logic
3. Test with actual terraform runs
4. Monitor for any issues

---

## Notes

- Artifact retention is set to 1 day to minimize storage
- The `test/` key prefix is used to avoid conflicts with production locks
- Use `workflow_dispatch` for manual testing during development
- The dummy jobs can be expanded to include actual operations once verified
- `jq` is pre-installed on GitHub Actions runners and used for JSON manipulation
- The `if: always()` condition ensures jobs run even if upstream jobs fail
- `fromJson()` requires the input to be valid JSON - proper error handling is included

---

## Related Files in This Repository

### `.github/workflows/shared-lock.yml`

**Current state**: Reusable workflow that acquires locks using GitHub's lock action
**Inputs**:

- `enabled`: Whether to run (for matrix filtering)
- `key`: Lock key (e.g., "d", "n", "p")
- `key_prefix`: Prefix for lock key (e.g., "hotfix/", "test/")
- `key_owner`: Identifier for who holds the lock
- `job_name`: Custom job name
- `delay_before_start_in_sec`: Delay before starting (for propagation)

**Current outputs**: Environment-specific status outputs (d-status, n-status, p-status)
**Issue**: These don't work with matrix strategy in reusable workflows

### `.github/workflows/shared-unlock.yml`

**Purpose**: Releases locks acquired by shared-lock.yml
**Note**: May need similar artifact handling if we want to track unlock status

---

## Expected JSON Format

### Individual lock result (uploaded by each matrix instance):

```json
{
    "key": "d",
    "full_key": "test/d",
    "status": "success", // or "skipped" or "failure"
    "outcome": "success", // GitHub Actions step outcome
    "enabled": true,
    "owner": "test-run"
}
```

### Aggregated lock status (output from aggregate job):

```json
{
    "d": "success",
    "n": "success",
    "p": "skipped"
}
```

### Usage in conditional:

```yaml
# In matrix job with matrix.env: [d, n, p]
if: fromJson(needs.aggregate-lock-results.outputs.lock-status)[matrix.env] != 'failure'

# For specific environment (not recommended, use matrix instead)
if: fromJson(needs.aggregate-lock-results.outputs.lock-status).d != 'failure'
```

### Matrix vs Separate Jobs

The test workflow uses a **matrix strategy** for dummy jobs (same pattern as `terraform-run-all.yml`):

**Benefits**:

- ✅ Single job definition instead of 3 separate jobs
- ✅ Dynamic status lookup using `matrix.env`
- ✅ Easy to add more environments
- ✅ Matches the pattern used in production terraform workflow

**Key expression**: `jq -r --arg env "$ENV" '.[$env]'` dynamically looks up status by environment key.

---

## Common Errors and Solutions

| Error                                   | Cause                                       | Solution                                      |
| --------------------------------------- | ------------------------------------------- | --------------------------------------------- |
| `unexpected end of file`                | JSON is malformed                           | Check `jq` validation in aggregate step       |
| `Cannot read property 'd' of undefined` | fromJson() received invalid JSON            | Verify lock-status output is set correctly    |
| `Artifact not found`                    | Download pattern doesn't match upload names | Ensure both use `lock-result-*` pattern       |
| `Job needs: lock not found`             | Matrix job dependency issue                 | Use `needs: lock` (singular) not `needs.lock` |

---

## Testing Strategy

1. **Start simple**: Test with just 2 environments (d, n)
2. **Add complexity**: Add third environment (p) after basics work
3. **Test failures**: Intentionally cause lock failures to verify error handling
4. **Test edge cases**: Empty deploy_labels, all disabled, etc.
5. **Visual inspection**: Download artifacts from Actions UI to verify contents

---

## Rollback Plan

If the artifact approach doesn't work:

1. Remove artifact upload steps from `shared-lock.yml`
2. Remove test workflow file
3. Revert to current approach (less granular conditional logic)
4. Consider alternative: Use separate lock jobs (no matrix) in caller workflow

---

## Future Enhancements (Out of Scope for This Plan)

1. Add retry logic for lock acquisition
2. Include lock duration in artifact
3. Store lock history for audit purposes
4. Add notifications when locks fail
5. Integrate with deployment dashboards
