<!-- ps-prompt-version: 1.0.0 -->

MODE: COMPACT
Produce the shortest clear version that preserves every material requirement.
Remove:
- Duplicate instructions
- Filler and conversational preambles
- Repeated conclusions
- Redundant explanations
- Generic motivational language
- Unnecessary wording
Preserve:
- User intent
- Explicit constraints
- Acceptance criteria
- Technical identifiers
- Relevant examples
- Error messages and symptoms
- Implementation choices
- Explicit prohibitions
Prefer one precise sentence over several overlapping sentences when safe.
Do not add new requirements.
Do not sacrifice technical accuracy for shorter wording.
If the prompt cannot be shortened meaningfully without losing information, return it unchanged.
