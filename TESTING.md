# Testing

## Main components

- `GitBot`:
  - Passes `GitEvent` to `BotFeatures`.
  Not possible to make a test.
- `GithubWebhookServer`:
  - Receives issue open event and converts it to `GitEvent`.
  Done.

## `GitBot`

- `BotFeatures`:
  - Passes `GitEvent` to `ImproveFeature` and `LabelFeature`.
  Not possible to make a test.
- `ImproveFeature`:
  - On bad issue: writes message.
  - On good issue: does nothing.
- `LabelFeature`:
  - Should apply some labels.

## `GitHost`

- `GitHub`:
  - Each trait method should call a respective GitHub REST API endpoint.

## `Llm`

- `OpenAiLlm`
  - Each trait method should call a respective GitHub REST API endpoint.
