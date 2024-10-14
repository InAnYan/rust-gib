# Implementation

## Main idea and requirements

- Make bot extendable: we will use the term "feature" to refer to a part of functionality of a bot.
- Support different Git hostings: done via the `GitHost` trait and `Repository`, `Issue`, and `User` abstractions.
- Support different LLMs: done via `Llm` trait and `ChatMessage` + `CompletionParameters` abstractions.

## Data flow

### Connection to the world

The two main components that communicate with the world are:
- `GitBot`: responsible for processing `GitEvent`s.
- Webhook server (`githost::impls::github::webhook_server::listen_to_events`): responsible for receiveing `GitEvent`s

These two components are connected via a Tokio channel. The webhook server sends events over this channel, and `GitBot` processes them.

### `GitHost`

`GitHost` is a very crucial trait that is responsible for **performing actions in Git host**. Speaking in AI terms, `GitHost` is an actuator, while the webhook server is perceptor.

`GitHost` is stored in `GitBot`, and then passed as an argument to all features.

### `GitBot`

`GitBot` consists of `BotFeatures`: collection of all enabled features. `BotFeatures` contains `ImproveFeature` and `LabelFeature`.

When `GitBot` receives a `GitEvent` it just sends them to `BotFeatures`, which then propagates the events to `LabelFeature` and `ImproveFeature`.

`GitBot` will also send a referene to `GitHost`, as stated earlier.

That is the main "framework" of the project. Every other description is related to **currently implemented** features. What I mean is: `GitBot` is not dependend and not related to an LLM service or a database. Every other dependency is stored in a feature struct.

### `ImproveFeature` and `LabelFeature`

(I decided to group these two features together, because their logic is *of the same similarity*).

These features rely on `Llm`, and so they require it as a dependency, and store an `Llm` in their struct (actually, transitively via an `LlmAgent`).

When a `GitEvent` ocurrs, it handles it to an `LlmAgent` struct, which returns AI output. It will be discussed in the next section, but for now you only need to know that it responsible for generating 

Everyone of them check if the AI output starts with "EMTPY" (without quotes). If it starts, then these features won't send any messages or perform other Git actions. But if the message doesn't start with "EMPTY", then it means that AI wants to say something.

`ImproveFeature` just sends the AI output as a message.

`LabelFeature` will parse the output in this way:

1. Split the output by ", ".
2. Trim all the parts.
3. Treat parts as label names and assign them to issue.

(All of that means that the prompts should be written in a special way).

### `LlmAgent`

This is just a utility struct that is made for reducing code duplication.

Its purpose is simple: construct a chat with LLM with two messages: a system message and a user message. These messages are generated with templates ('Tera' is used as a template engine). After constructing this chat, it sends it to LLM and returns the LLM output.

## Coding decisions

### Configuration

I used configuration structs (structs with some parameters that is deserializable) extensively in the code, so that `config` crate could be used, and so that each components manages its own configuration.

### Construction

Each of structs has two kinds of consturctors: one with a config struct and one "raw" (no config structs, file source instead of a path). The first ones is used while running the bot, the second ones are used for testing.

### Managing dependencies

Full disclosure: I came from Java background. Ultra-mega full disclosure: I used `dyn` and `Arc<Mutex<...>>` extensively. If the bot was written in Java I would have interfaces for `Llm`, `GitHost`, `BotFeature`, and would use them extensively with full abstraction. Initally, the bot even stored a vector of `BotFeature`s.

Of course I went against this approach. Rust forces to think you straightly, more closer to the architecture. (Java is like an OOP, and Rust is like a relational database: it's hard to fully model OOP relationships and principles in a relational database, RDB is flatter, hope you got the idea).

Instead of interfaces, the code has generic parameters (typically, `G` for `G: GitHost`, `L` for `L: Llm`).

I chose to pass a concrete type of errors for `GitHost` and `Llm`. And so, for exampe, you can see this line in `bot::errors` module:

```rust
#[derive(Debug, thiserror::Error)]
pub enum GitBotError<GE, LE> {
    #[error("issue-improve feature returned an error")]
    ImproveFeatureError(#[from] ImproveFeatureError<GE, LE>),

    #[error("issue-label feature returned an error")]
    LabelFeatureError(#[from] LabelFeatureError<GE, LE>),
}

pub type Result<T, GE, LE> = std::result::Result<T, GitBotError<GE, LE>>;
```

And that leads to a bit ugly method definitions:

```rust
pub async fn process_event(&self, event: &GitEvent) -> Result<(), G::Error, L::Error> {
    ...
}
```
