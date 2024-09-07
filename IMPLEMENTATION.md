# Implementation

1. An external program listens for `GitEvent`s. (Because GitHub API is not as easy to use)
2. The events are transfered to `GitBot`.
3. `GitBotBot` then transfers events to `GitBotFeature`s.
4. `GitBotFeature` processes the events, and if it decides to do something, it performs actions through the `GitHost`.
5. If `GitBotFeature` wants to call LLM, then it uses `Llm` type.

# Implementation details

The bot is designed to be modular and retargetable. Features can be turned off/on. This bot can be retargeted to other Git hosting (like GitLab or BitBucket).
