# FlowVersion Test Scenario

This document outlines a complete test scenario for FlowVersion's MVP functionality.

## Prerequisites

1. Rust installed with cargo available
2. FlowVersion compiled (`cargo build --release`)
3. Test project files created in `test_project/` directory

## Test Steps

### 1. Initialize Repository

```bash
cd test_project
../target/release/flow init --name "calculator-app" --ai-mode local
```

**Expected Result:**
- `.flowversion` directory created
- Configuration files initialized
- Success message displayed

### 2. Add Initial Files

```bash
../target/release/flow add main.py --intention "Add main calculator application"
```

**Expected Result:**
- File added to staging area
- Intention associated with the change
- "added: main.py" message displayed

### 3. Create First Commit

```bash
../target/release/flow commit \
  --intention "Implement basic calculator functionality" \
  --context "Initial version with four basic operations" \
  --impact "Establishes foundation for calculator app" \
  --confidence 0.9
```

**Expected Result:**
- Commit created with unique ID
- Intention, context, and impact recorded
- Confidence score stored
- Commit hash displayed

### 4. Add More Files

```bash
../target/release/flow add utils.py --intention "Add mathematical utility functions"
../target/release/flow add README.md --intention "Add project documentation"
```

**Expected Result:**
- Both files staged
- Intentions recorded for each file
- Status messages for each addition

### 5. Create Second Commit

```bash
../target/release/flow commit \
  --intention "Add utility functions and documentation" \
  --context "Enhance calculator with advanced math functions" \
  --impact "Improves functionality and maintainability" \
  --confidence 0.8
```

**Expected Result:**
- Second commit created
- All staged files included in commit
- Commit metadata properly stored

### 6. View Commit History

```bash
../target/release/flow log
```

**Expected Result:**
- List of all commits in reverse chronological order
- Each commit shows: ID, date, goal, context, impact, confidence
- Proper formatting and readability

### 7. View Concise History

```bash
../target/release/flow log --oneline
```

**Expected Result:**
- One line per commit
- Format: `[SHORT_ID] GOAL`
- Chronological order maintained

### 8. View History with Intentions

```bash
../target/release/flow log --intentions
```

**Expected Result:**
- Detailed view including intention tags
- Full context and impact information
- Generated tags based on content analysis

## Validation Checklist

### Core Functionality
- [ ] Repository initialization works
- [ ] Files can be added to staging area
- [ ] Commits are created with full intention metadata
- [ ] Commit history is properly maintained
- [ ] Different log formats work correctly

### Data Integrity
- [ ] Commit IDs are unique and consistent
- [ ] File hashes are calculated correctly
- [ ] Intention data is preserved accurately
- [ ] Timestamps are recorded properly
- [ ] Configuration is maintained between sessions

### Error Handling
- [ ] Graceful handling of missing files
- [ ] Proper error messages for invalid operations
- [ ] Validation of confidence scores (0.0-1.0)
- [ ] Repository state consistency maintained

### User Experience
- [ ] Clear success/error messages
- [ ] Intuitive command structure
- [ ] Helpful output formatting
- [ ] Reasonable performance (sub-second for basic operations)

## Expected File Structure After Testing

```
test_project/
├── .flowversion/
│   ├── objects/
│   │   ├── blobs/
│   │   │   └── [hash-based storage]
│   │   └── commits/
│   │       └── [commit objects]
│   ├── refs/
│   │   └── streams/
│   │       └── main
│   ├── config.json
│   ├── index.json
│   └── HEAD
├── main.py
├── utils.py
└── README.md
```

## Performance Benchmarks (Target)

- `flow init`: < 100ms
- `flow add` (single file): < 50ms
- `flow commit`: < 200ms
- `flow log`: < 100ms

## Success Criteria

The test scenario passes if:
1. All commands execute without errors
2. Expected output is produced at each step
3. File system state matches expectations
4. Data integrity is maintained throughout
5. Performance targets are met

This completes the MVP validation for FlowVersion's core intent-based version control functionality.