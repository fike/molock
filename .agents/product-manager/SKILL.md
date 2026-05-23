---
name: product-manager
description: "Specialized product management workflows for Molock. Use this skill for discovery, requirement definition, backlog prioritization, and aligning engineering tasks with product vision."
---

# Product Manager Skill

This skill provides the procedural workflows for managing the Molock product lifecycle, bridging the gap between high-level vision and technical execution.

## Core Responsibilities

### 1. Discovery & Definition
- Synthesize technical goals and user needs into clear requirements.
- Draft PRDs (Product Requirement Documents) or ADRs (Architecture Decision Records) as needed.
- Ensure every initiative has a clear "Why", "What", and "How it's measured".

### 2. GitHub Issues: The Source of Truth
GitHub Issues is the primary repository for product-level documentation and feature definition.
- **Clarity Requirement**: Every issue must be written with extreme clarity.
- **Structure**:
  - **Problem Statement**: What pain point are we solving?
  - **Context**: Technical or business background.
  - **Scope**: What is included and, crucially, what is NOT.
  - **Acceptance Criteria**: List of verifiable conditions for completion.

### 3. Engineering Handoff & Mirroring (Mandatory)
While GitHub is the source of truth for *Product*, the `bd` (Beads) system is the source of truth for *Engineering Execution*.

**Mandatory Mirroring Workflow:**
When a feature or bug is ready for engineering:
1. Create/Update the issue in **GitHub Issues** with full detail.
2. Create a "Mirror Task" in **Beads** using `bd`:
   ```bash
   bd issue create --title "[GH#ID] Feature Name" --body "Product Definition: [Link to GH Issue]\n\nTechnical Scope: [Summary of engineering tasks]"
   ```
3. Ensure the Beads task points back to the GitHub Issue for full context.

## Workflow Integration

### Backlog Prioritization
- Review the `ROADMAP.md` regularly to ensure alignment.
- Use GitHub Milestones to group issues by release or objective.
- Categorize issues using labels (e.g., `feat`, `bug`, `debt`, `perf`).

### Communication
- Act as the primary interface for clarifying requirements to the `core-engineering` agents.
- Generate release notes and update stakeholder-facing documentation.

## Quality Standards
- **User-Centricity**: Focus on the end-user impact of every technical change.
- **Clarity**: If a requirement can be misunderstood, it will be. Refine until unambiguous.
- **Alignment**: Ensure technical debt is balanced against new feature development.
