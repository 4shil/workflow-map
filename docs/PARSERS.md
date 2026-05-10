# Parser Implementation Guide

This document covers how each framework parser works, what patterns it detects, and how to extend it.

## AutoGen Parser (`src/parsers/autogen.rs`)

**File type:** `.py` (Python)

**Detection patterns:**
- `import autogen` or `from autogen import`
- `AssistantAgent`, `UserProxyAgent`, `GroupChat` class usage

**Parsed elements:**
- `AssistantAgent("name", ...)` -> StepType::Agent
- `UserProxyAgent("name", ...)` -> StepType::Agent
- `GroupChat(agents=[...], ...)` -> StepType::Chain with agent children

**Limitations:**
- Only detects direct instantiations, not dynamic agent creation
- Speaker selection logic in GroupChat is not parsed

## CrewAI Parser (`src/parsers/crewai.rs`)

**File type:** `.yaml`, `.yml`

**Detection patterns:**
- Top-level `agents:` and `tasks:` keys

**Parsed elements:**
- Each agent -> StepType::Agent with tools as children
- Each task -> StepType::Task with agent assignment
- Task dependencies from `context` field
- Process type (sequential/hierarchical) stored in metadata

**Limitations:**
- Only parses standard CrewAI YAML format
- Custom agent/task configurations may not be fully captured

## DSPy Parser (`src/parsers/dspy.rs`)

**File type:** `.py` (Python)

**Detection patterns:**
- `import dspy`
- `class Foo(dspy.Module)` subclass definitions
- `dspy.Predict(...)`, `dspy.ChainOfThought(...)`, `dspy.Retrieve(...)`

**Parsed elements:**
- Module subclasses -> StepType::Module with forward() sub-calls as children
- Predict/ChainOfThought -> StepType::Predict
- Retrieve -> StepType::Retrieve

**Limitations:**
- Only parses direct `self.method_name()` calls in forward()
- Dynamic module composition is not detected

## Generic Parser (`src/parsers/generic.rs`)

**File type:** `.yaml`, `.yml`, `.json`

**Detection patterns:**
- YAML/JSON with `steps:` key containing array of step objects

**Schema:**
```yaml
name: "Workflow Name"
steps:
  - id: "step1"
    name: "Step Name"
    type: agent | tool | chain | task | step
    description: "Optional description"
    depends_on: ["other_step_id"]
edges:
  - from: "step1"
    to: "step2"
    type: sequential | conditional | parallel
```

**Limitations:**
- Requires strict schema adherence
- No framework-specific metadata extraction

## Hermes Agent Parser (`src/parsers/hermes.rs`)

**File type:** `SKILL.md` or `*_skill.md`

**Detection patterns:**
- File named `SKILL.md` or ending with `_skill.md`
- YAML frontmatter between `---` delimiters

**Parsed elements:**
- Frontmatter fields (name, description, category) -> metadata
- Each `## Section Header` -> StepType::Step
- Sequential edges between sections

**Limitations:**
- Only parses markdown section headers, not nested content
- Code blocks within sections are not analyzed

## LangChain Parser (`src/parsers/langchain.rs`)

**File type:** `.py` (Python)

**Detection patterns:**
- `import langchain` or `from langchain... import`
- Pipe chains: `chain = prompt | llm | parser`
- `AgentExecutor(...)`, `RunnableParallel(...)`, `RunnableLambda(...)`

**Parsed elements:**
- Pipe chains -> StepType::Chain with components as children
- AgentExecutor -> StepType::Agent
- RunnableParallel -> StepType::Chain
- RunnableLambda -> StepType::Lambda
- Other class instantiations -> StepType::Step

**Limitations:**
- Regex-based: may miss complex multi-line pipe chains
- Lambda functions get generic names
- Dynamic chain construction at runtime is not detected

## Adding a New Parser

1. Create `src/parsers/<name>.rs` with:
   ```rust
   use crate::model::*;
   use anyhow::Result;
   use std::path::Path;

   pub fn parse(content: &str, path: &Path) -> Result<Workflow> {
       // Your parsing logic
   }
   ```

2. Add detection to `src/parsers/detector.rs`:
   ```rust
   // In detect_file() or a new detection function
   if content.contains("your_framework_marker") {
       return Some(Framework::YourFramework);
   }
   ```

3. Register in `src/parsers/mod.rs`:
   ```rust
   pub mod your_framework;
   // In parse_workflow():
   Framework::YourFramework => your_framework::parse(&content, path)?,
   ```

4. Add tests and fixtures (see TESTING.md)

5. Update README.md Supported Frameworks table
