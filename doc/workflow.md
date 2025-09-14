# Development Workflow

## Core Principles

**KISS**: Keep it Simple, Stupid - follow the plan strictly, implement incrementally, test early.

---

## Iteration Workflow

### 1. Planning Phase
- **Read tasklist**: Review current iteration requirements
- **Propose solution**: Present implementation approach with:
  - Key code sections/interfaces
  - File structure changes
  - Dependencies needed
- **Wait for approval**: Get explicit confirmation before coding

### 2. Implementation Phase  
- **Follow the plan**: Implement exactly what was agreed upon
- **Test as you go**: Ensure each change compiles and basic functionality works
- **Stay focused**: Complete only the current iteration scope

### 3. Validation Phase
- **Run tests**: Execute all specified tests for the iteration
- **Manual verification**: Confirm the "Testing" criteria from tasklist
- **Wait for confirmation**: Get user approval before marking complete

### 4. Completion Phase
- **Update progress**: Mark iteration as completed in tasklist
- **Add notes**: Document what was implemented and any learnings
- **Commit changes**: Make atomic git commit with clear message
- **Agree next step**: Confirm transition to next iteration

---

## Communication Rules

### Before Implementation
```
PROPOSE: "For Iteration X.Y, I will implement:
- File: src/example.rs - ExampleStruct with methods A, B
- Update: Cargo.toml - add dependency Z
- Test: cargo build should succeed

Code preview:
```rust
// Key implementation details
```

Proceed with implementation?"
```

### After Implementation
```
COMPLETED: "Iteration X.Y finished:
✅ All requirements implemented
✅ Tests passing: [specific test results]
✅ Ready for validation

Please confirm completion and approve next iteration."
```

---

## Tasklist Management

### Progress Updates
- Update status: ⏳ Pending → 🔄 In Progress → ✅ Completed
- Update progress percentage based on sub-tasks
- Add completion date and relevant notes

### Task Breakdown
- Focus on ONE iteration at a time
- Complete all sub-tasks before moving forward
- Test at each checkpoint, not just at the end

---

## Git Workflow

### Commit Message Format
```
[Phase X.Y] Brief description

- Implemented: specific features
- Added: new files/dependencies  
- Testing: verification performed
```

### Commit Timing
- **One commit per iteration**: Atomic, focused changes
- **After user approval**: Never commit without confirmation
- **Include all changes**: Code, tests, documentation updates

---

## Quality Gates

### Before Each Implementation
- [ ] Iteration requirements clearly understood
- [ ] Implementation approach approved by user
- [ ] Dependencies and file changes agreed upon

### After Each Implementation  
- [ ] Code compiles without errors
- [ ] Specified tests pass
- [ ] Iteration criteria met
- [ ] User has validated the changes

### Before Next Iteration
- [ ] Current iteration marked complete in tasklist
- [ ] Progress percentages updated
- [ ] Git commit made with clear message
- [ ] Next iteration agreed upon

---

## Error Handling

### If Implementation Fails
1. **Stop immediately**: Don't continue with broken code
2. **Report issue**: Explain what went wrong and why
3. **Propose fix**: Suggest alternative approach
4. **Wait for guidance**: Get user input before proceeding

### If Tests Fail
1. **Identify root cause**: Understand why tests are failing
2. **Fix incrementally**: Address issues one at a time
3. **Re-test thoroughly**: Ensure fix doesn't break other functionality
4. **Update plan if needed**: Revise approach based on learnings

---

## Key Reminders

- **No surprises**: Always communicate before making changes
- **Stay on track**: Resist scope creep within iterations
- **Test early**: Don't accumulate untested changes
- **Document progress**: Keep tasklist current and accurate
- **Seek approval**: Every major step requires user confirmation

**Goal**: Steady, predictable progress through the planned development phases with full transparency and user control.
