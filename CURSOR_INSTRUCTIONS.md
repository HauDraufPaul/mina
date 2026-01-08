# Cursor Agent Instructions

**CRITICAL**: Read this entire document before making ANY changes to the codebase. These instructions prevent common AI mistakes.

## 🚨 MANDATORY PRE-FLIGHT CHECKS

Before making ANY code changes, you MUST:

1. **Read the entire file you're editing** - Never make partial edits without full context
2. **Search for existing implementations** - Check if similar functionality already exists
3. **Check related files** - Read imports, dependencies, and related modules
4. **Review existing patterns** - Follow the same patterns used elsewhere in the codebase
5. **Check linter errors** - Run `read_lints` after EVERY edit
6. **Verify no breaking changes** - Ensure existing functionality still works

## ❌ COMMON MISTAKES TO AVOID

### 1. File Reading & Context
- ❌ **NEVER** edit a file without reading it completely first
- ❌ **NEVER** make assumptions about file structure or existing code
- ❌ **NEVER** edit multiple files without understanding their relationships
- ✅ **ALWAYS** read the full file before editing
- ✅ **ALWAYS** search for similar implementations before creating new ones
- ✅ **ALWAYS** read related files (imports, dependencies, parent components)

### 2. Breaking Existing Functionality
- ❌ **NEVER** remove or modify existing functionality unless explicitly requested
- ❌ **NEVER** change function signatures without checking all call sites
- ❌ **NEVER** refactor without ensuring backward compatibility
- ✅ **ALWAYS** preserve existing behavior when adding new features
- ✅ **ALWAYS** check all usages before changing APIs
- ✅ **ALWAYS** test that existing features still work after changes

### 3. Code Duplication & Reusability
- ❌ **NEVER** create duplicate code when utilities already exist
- ❌ **NEVER** copy-paste code from one place to another
- ❌ **NEVER** create new components when existing ones can be reused
- ✅ **ALWAYS** search for existing utilities before creating new ones
- ✅ **ALWAYS** extract common patterns into reusable functions/components
- ✅ **ALWAYS** check `src/utils/` and `src/components/ui/` for existing solutions

### 4. Dependencies & Imports
- ❌ **NEVER** add new dependencies without checking if alternatives exist
- ❌ **NEVER** add dependencies that duplicate existing functionality
- ❌ **NEVER** break existing imports
- ✅ **ALWAYS** check `package.json` and `Cargo.toml` for existing dependencies
- ✅ **ALWAYS** verify imports work after changes
- ✅ **ALWAYS** use existing dependencies when possible

### 5. TypeScript & Type Safety
- ❌ **NEVER** use `any` type - this project uses strict TypeScript
- ❌ **NEVER** skip type definitions
- ❌ **NEVER** ignore TypeScript errors
- ✅ **ALWAYS** provide proper types for all functions, props, and variables
- ✅ **ALWAYS** fix TypeScript errors immediately
- ✅ **ALWAYS** use existing type definitions when available

### 6. React Patterns & Best Practices
- ❌ **NEVER** use `alert()` for errors - use proper error UI components
- ❌ **NEVER** forget `useEffect` cleanup functions (intervals, timeouts, subscriptions)
- ❌ **NEVER** miss dependencies in `useEffect` dependency arrays
- ❌ **NEVER** create components without proper error handling
- ✅ **ALWAYS** include cleanup in `useEffect` for intervals/timeouts/subscriptions
- ✅ **ALWAYS** include all dependencies in `useEffect` arrays
- ✅ **ALWAYS** handle errors with proper UI feedback, not `console.error()` alone
- ✅ **ALWAYS** validate user inputs before submitting

### 7. Rust Error Handling
- ❌ **NEVER** use `.unwrap()` - this project has 105+ instances that need fixing
- ❌ **NEVER** ignore `Result` types
- ❌ **NEVER** panic on errors
- ✅ **ALWAYS** use proper error handling with `?` operator or `map_err()`
- ✅ **ALWAYS** return `Result` types from functions that can fail
- ✅ **ALWAYS** handle database errors gracefully (mutex poisoning, etc.)

### 8. Design System & UI Components
- ❌ **NEVER** create new UI components when existing ones exist
- ❌ **NEVER** break the glassmorphism design system
- ❌ **NEVER** use inline styles instead of Tailwind classes
- ❌ **NEVER** hardcode colors - use design tokens
- ✅ **ALWAYS** use existing components from `src/components/ui/` (Button, Card, Modal, etc.)
- ✅ **ALWAYS** use Tailwind classes with glassmorphism styles (`glass-card`, `glass-button`, etc.)
- ✅ **ALWAYS** use design tokens from `src/styles/tokens.css`
- ✅ **ALWAYS** use neon colors from Tailwind config (`neon-cyan`, `neon-green`, etc.)

### 9. Architecture & File Structure
- ❌ **NEVER** create files in wrong locations
- ❌ **NEVER** break the modular architecture
- ❌ **NEVER** mix concerns (UI logic in storage, etc.)
- ✅ **ALWAYS** follow the existing file structure:
  - React components: `src/components/modules/` or `src/components/ui/`
  - Rust commands: `src-tauri/src/commands/`
  - Rust storage: `src-tauri/src/storage/`
  - Rust providers: `src-tauri/src/providers/`
  - Utilities: `src/utils/` (frontend) or `src-tauri/src/utils/` (backend)
- ✅ **ALWAYS** maintain separation of concerns

### 10. Hardcoded Values & Configuration
- ❌ **NEVER** hardcode magic numbers (dimensions, limits, timeouts)
- ❌ **NEVER** hardcode strings that should be configurable
- ❌ **NEVER** hardcode API endpoints or URLs
- ✅ **ALWAYS** extract magic numbers to named constants
- ✅ **ALWAYS** use configuration files for configurable values
- ✅ **ALWAYS** use environment variables for API keys and endpoints

### 11. Testing & Validation
- ❌ **NEVER** skip input validation
- ❌ **NEVER** assume data is in the correct format
- ❌ **NEVER** ignore edge cases
- ✅ **ALWAYS** validate all user inputs
- ✅ **ALWAYS** handle edge cases (empty arrays, null values, etc.)
- ✅ **ALWAYS** check for existing tests before adding new functionality

### 12. Documentation & Comments
- ❌ **NEVER** skip documentation for public functions
- ❌ **NEVER** leave complex logic unexplained
- ❌ **NEVER** use unclear variable names
- ✅ **ALWAYS** add doc comments for public Rust functions
- ✅ **ALWAYS** add comments for complex algorithms
- ✅ **ALWAYS** use descriptive variable and function names

### 13. Performance & Optimization
- ❌ **NEVER** create unnecessary re-renders
- ❌ **NEVER** forget to memoize expensive computations
- ❌ **NEVER** create memory leaks (unclosed intervals, subscriptions)
- ✅ **ALWAYS** use `useMemo` and `useCallback` for expensive operations
- ✅ **ALWAYS** clean up all resources (intervals, timeouts, WebSocket connections)
- ✅ **ALWAYS** debounce high-frequency updates

### 14. Tauri Commands
- ❌ **NEVER** create commands without proper error handling
- ❌ **NEVER** skip input validation in Tauri commands
- ❌ **NEVER** expose sensitive operations without permission checks
- ✅ **ALWAYS** follow existing command patterns in `src-tauri/src/commands/`
- ✅ **ALWAYS** return proper `Result` types from commands
- ✅ **ALWAYS** validate inputs in Rust commands

## 📋 PROJECT-SPECIFIC RULES

### TypeScript Configuration
- **Strict mode enabled**: `strict: true` in `tsconfig.json`
- **No unused locals/parameters**: Must remove unused code
- **Path aliases**: Use `@/*` for `./src/*` imports
- **Example**: `import { cn } from "@/utils/cn"`

### Design System
- **Glassmorphism**: All cards/panels use `glass-card` or `glass-button` classes
- **Colors**: Use `neon-cyan`, `neon-green`, `neon-amber`, `neon-red` from Tailwind config
- **Fonts**: Use `font-mono` (JetBrains Mono) for terminal aesthetics
- **Backdrop blur**: Use `backdrop-blur-glass` (28px) for glass effects
- **Phosphor glow**: Use `phosphor-glow-cyan`, etc. for neon text effects

### Component Patterns
- **UI Components**: Located in `src/components/ui/` (Button, Card, Modal)
- **Module Components**: Located in `src/components/modules/`
- **Layout Components**: Located in `src/components/layout/`
- **Always use**: `cn()` utility from `@/utils/cn` for className merging

### Rust Patterns
- **Error Handling**: Use `anyhow::Result` or custom error types, NEVER `.unwrap()`
- **Database**: Use SQLite with proper connection management
- **Commands**: All Tauri commands in `src-tauri/src/commands/` must return `Result`
- **Storage**: All storage modules follow similar patterns - check existing ones first

### State Management
- **Frontend**: Zustand + React Query
- **Backend**: SQLite database with storage modules
- **Real-time**: WebSocket-based streaming (port 17602)

## 🔍 BEFORE MAKING CHANGES CHECKLIST

1. [ ] Read the entire file(s) you're editing
2. [ ] Search for existing similar implementations
3. [ ] Check related files and dependencies
4. [ ] Review existing patterns in the codebase
5. [ ] Verify no breaking changes to existing functionality
6. [ ] Check for existing utilities/components to reuse
7. [ ] Ensure proper TypeScript types (no `any`)
8. [ ] Add proper error handling (no `.unwrap()` in Rust, no `alert()` in React)
9. [ ] Include cleanup in `useEffect` hooks
10. [ ] Validate all user inputs
11. [ ] Use existing UI components and design system
12. [ ] Extract magic numbers to constants
13. [ ] Check linter errors after changes
14. [ ] Test that existing features still work

## 🎯 CODE QUALITY STANDARDS

### TypeScript
- Strict mode enabled
- No `any` types
- All functions properly typed
- No unused variables/parameters

### Rust
- No `.unwrap()` calls
- Proper `Result` handling
- No panics
- Comprehensive error messages

### React
- Proper `useEffect` dependencies
- Cleanup functions for intervals/subscriptions
- Error boundaries and proper error UI
- Input validation
- No `alert()` or `console.error()` without user feedback

### General
- No code duplication
- Proper documentation
- Consistent naming conventions
- Follow existing patterns

## 🚫 ABSOLUTE PROHIBITIONS

1. **NEVER** use `.unwrap()` in Rust code
2. **NEVER** use `any` type in TypeScript
3. **NEVER** use `alert()` for errors in React
4. **NEVER** skip `useEffect` cleanup functions
5. **NEVER** edit files without reading them first
6. **NEVER** break existing functionality
7. **NEVER** create duplicate code when utilities exist
8. **NEVER** hardcode magic numbers
9. **NEVER** skip input validation
10. **NEVER** ignore linter errors

## ✅ ALWAYS DO

1. **ALWAYS** read files completely before editing
2. **ALWAYS** search for existing implementations
3. **ALWAYS** check linter errors after changes
4. **ALWAYS** use existing components/utilities
5. **ALWAYS** follow the design system
6. **ALWAYS** handle errors properly
7. **ALWAYS** validate inputs
8. **ALWAYS** clean up resources
9. **ALWAYS** preserve existing functionality
10. **ALWAYS** follow existing patterns

---

**Remember**: When in doubt, search the codebase first. This project has extensive existing code - reuse it rather than recreating it.
