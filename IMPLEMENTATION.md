# Implementation

1. An external program listens for `GitEvent`s. (Because GitHub API is not as easy to use)
2. The events are transfered to `GitBot`.
3. `GitBotBot` then transfers events to `GitBotFeature`s.
4. `GitBotFeature` processes the events, and if it decides to do something, it performs actions through the `GitHost`.
5. If `GitBotFeature` wants to call LLM, then it uses `Llm` type.

# Implementation details

The bot is designed to be modular and retargetable. Features can be turned off/on. This bot can be retargeted to other Git hosting (like GitLab or BitBucket).

# Crate hierarchy

- `cli`: used to start the bot.
- `core`: main crates for the bot.
  - `bot`: The Bot.
  - `feature-types`: `GitBotFeature` type (the facade).
  - `githost-types`: `GitHost` and `GitEvent` type (the facade).
- `features`: collection of bot features.
  - `improve-feature`: improves issues by asking the user for more relevant information.
  - `label-feature`: automatic labelling of issues.
- `llm`: crates related to LLM.
  - `llm-types`: `Llm` type (the facade).
  - `llm-openai`: OpenAI API compatible LLM implementation.
- `github-githost`: GitHub `GitHost` implementation.
