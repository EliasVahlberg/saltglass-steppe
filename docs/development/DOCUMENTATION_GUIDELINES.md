# Documentation Guidelines

This document establishes standards for creating, maintaining, and organizing project documentation.

## File Organization

### Directory Structure

```
docs/
├── README.md                    # Main documentation index
├── DOCUMENT_DATABASE.md         # Complete document listing
├── architecture/               # Technical architecture docs
├── design/                     # Game design documents
├── development/                # Development guides and tools
├── features/                   # Feature specifications
├── narrative/                  # Lore and story content
└── testing/                    # QA and testing documentation
```

### File Naming Conventions

- Use `UPPER_CASE.md` for major documents (e.g., `README.md`, `CHANGELOG.md`)
- Use `Title_Case.md` for specific documents (e.g., `Game_Concept.md`)
- Use descriptive, concise names that clearly indicate content
- Avoid spaces in filenames; use underscores or hyphens
- Include dates in changelog/version files: `YYYYMMDD_DESCRIPTION.md`

## Document Structure

### Standard Header Format

```markdown
# Document Title

Brief description of the document's purpose and scope.

## Table of Contents (for longer documents)

- [Section 1](#section-1)
- [Section 2](#section-2)

## Content sections...

---

*Last updated: YYYY-MM-DD*
*Author: [Name/Role]*
```

### Required Elements

1. **Clear title** - Descriptive and specific
2. **Purpose statement** - What this document covers
3. **Table of contents** - For documents >500 words
4. **Consistent formatting** - Use standard markdown
5. **Update metadata** - Date and author information

## Content Guidelines

### Writing Style

- **Clear and concise** - Avoid unnecessary complexity
- **Active voice** - "The system processes data" vs "Data is processed"
- **Present tense** - For current state descriptions
- **Consistent terminology** - Use established project vocabulary
- **Audience-aware** - Consider who will read this document

### Technical Documentation

- Include **code examples** where relevant
- Provide **step-by-step instructions** for procedures
- Use **diagrams and flowcharts** for complex systems
- Include **prerequisites** and **dependencies**
- Add **troubleshooting sections** for common issues

### Design Documentation

- Start with **high-level concepts** before details
- Include **rationale** for design decisions
- Provide **examples and use cases**
- Reference **related systems** and dependencies
- Include **implementation notes** where helpful

## Maintenance Procedures

### Regular Updates

- **Review quarterly** - Check for outdated information
- **Update after changes** - Modify docs when code/design changes
- **Version control** - Track significant document changes
- **Link validation** - Ensure internal links remain valid

### Document Lifecycle

1. **Creation** - Follow templates and guidelines
2. **Review** - Technical and editorial review before merge
3. **Maintenance** - Regular updates and corrections
4. **Archival** - Move outdated docs to archive folder
5. **Deletion** - Remove obsolete documents with team approval

## Templates

### Feature Specification Template

```markdown
# Feature Name

## Overview
Brief description of the feature and its purpose.

## Requirements
- Functional requirements
- Non-functional requirements
- Dependencies

## Design
High-level design approach and architecture.

## Implementation
Technical implementation details and considerations.

## Testing
Testing approach and acceptance criteria.

## Timeline
Development phases and milestones.
```

### Technical Design Document Template

```markdown
# System Name - Technical Design

## Problem Statement
What problem does this system solve?

## Goals and Non-Goals
What this system will and won't do.

## Architecture
High-level system architecture and components.

## Detailed Design
Implementation details, APIs, data structures.

## Alternatives Considered
Other approaches and why they were rejected.

## Testing Strategy
How the system will be tested and validated.

## Rollout Plan
Deployment and rollout considerations.
```

## Quality Standards

### Content Quality

- **Accuracy** - Information must be correct and current
- **Completeness** - Cover all necessary aspects
- **Clarity** - Easy to understand for target audience
- **Consistency** - Follow established patterns and terminology

### Technical Quality

- **Valid markdown** - Proper syntax and formatting
- **Working links** - All references must be accessible
- **Proper structure** - Logical organization and hierarchy
- **Searchable** - Use clear headings and keywords

## Review Process

### Before Publishing

1. **Self-review** - Author checks content and formatting
2. **Technical review** - Subject matter expert validation
3. **Editorial review** - Grammar, style, and clarity check
4. **Link validation** - Verify all internal and external links

### Review Criteria

- [ ] Content is accurate and up-to-date
- [ ] Structure follows guidelines
- [ ] Writing is clear and concise
- [ ] All links work correctly
- [ ] Formatting is consistent
- [ ] Metadata is complete

## Tools and Resources

### Recommended Tools

- **Markdown editors** - Typora, Mark Text, or VS Code
- **Diagram tools** - Mermaid, Draw.io, or ASCII diagrams
- **Link checkers** - markdown-link-check or similar
- **Spell checkers** - Built-in editor tools

### Reference Materials

- [Markdown Guide](https://www.markdownguide.org/)
- [GitHub Flavored Markdown](https://github.github.com/gfm/)
- [Technical Writing Guidelines](https://developers.google.com/tech-writing)

## Common Patterns

### Cross-References

```markdown
See [Architecture Overview](../architecture/ARCHITECTURE_OVERVIEW.md) for details.
```

### Code Blocks

```markdown
```rust
// Rust code example
fn main() {
    println!("Hello, world!");
}
```
```

### Tables

```markdown
| Feature | Status | Priority |
|---------|--------|----------|
| Combat  | Done   | High     |
| Trading | WIP    | Medium   |
```

### Callouts

```markdown
> **Note:** Important information that readers should notice.

> **Warning:** Critical information about potential issues.

> **Tip:** Helpful suggestions for better results.
```

---

*Last updated: 2025-12-24*
*Author: Lead Developer*
