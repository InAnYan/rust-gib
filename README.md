# Git Intellectual Bot (GIB)

## Project Description

Git Intellectual Bot (GIB) is an intelligent bot designed to manage user issues in Git repositories, streamlining the development workflow through automation.

### Key Features:
- **Issue Analysis**: GIB reads newly opened issues and asks clarifying questions to improve the quality and completeness of the report.
- **Automatic Labeling**: GIB analyzes the issue content and assigns relevant labels automatically to categorize the issues.

GIB leverages modern AI technologies, including large language models (LLMs), to improve efficiency and accuracy in issue handling. The bot is built with flexibility in mind, allowing integration with different LLM providers and Git hosting platforms. Currently, GIB supports GitHub and the OpenAI API.

This project is part of the capstone for the Ukrainian Summer Rustcamp 2024, showcasing the skills learned during the intensive bootcamp.

## How to Run the Project

### Step 1: Create a GitHub App

Follow the [official GitHub documentation](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/registering-a-github-app) to create a GitHub app.

GIB listens for new events in your repository via webhooks, which enable GitHub to send events to your server. You will need to set up a webhook server for this functionality.

While GIB handles the webhook server logic internally, you need to make your server accessible to GitHub. You have two options:
1. **Public IP**: If you have a server with a public IP, you’re all set. (Note: Public IP addresses are usually provided by your Internet Service Provider, and may require payment.)
2. **Tunneling Services**: Alternatively, you can use services like [`ngrok`](https://ngrok.com/) or [`localtunnel`](https://theboroer.github.io/localtunnel-www/) to create a tunnel to your local server. Follow their respective setup guides to generate the URL and port required.

Once you have your webhook URL, enter it in the "Webhook" section of your GitHub App's settings during the app creation process.

Make sure to save the **App ID** and download the private key after creating the app.

### Step 2: Install Your App on GitHub

Refer to the [GitHub documentation](https://docs.github.com/en/apps/using-github-apps/installing-your-own-github-app) for instructions on how to install your app on your account or organization.

Once installed, navigate to your GitHub settings page to retrieve the **Installation ID**. The URL will look like this: `https://github.com/settings/installations/55411026`. The numbers at the end are the Installation ID. Save this number for later use.

### Step 3: Get an OpenAI API Key

Follow [this tutorial](https://medium.com/@lorenzozar/how-to-get-your-own-openai-api-key-f4d44e60c327) to obtain an OpenAI API key, which will be required for GIB's language model functionality.

### Step 4: Configure GIB

Create a configuration file by using the example file located in `examples/config.yaml` as a reference.

To customize templates for analyzing and labeling issues, refer to the templates found in the `examples/rust-gib/templates` directory.

### Step 5: Start the Bot

Before running GIB, ensure that the OpenAI API key is set in your environment under the `GIB_OPENAI_KEY` variable.

To start the bot, run the following command:

```bash
cargo run
```

If you are using a custom configuration file, specify its path using the `GIB_CONFIG_FILE` environment variable. Be mindful of the current working directory (CWD) when running `cargo`, as it may affect relative paths.
