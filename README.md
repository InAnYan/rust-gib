# Git Intellectual Bot (GIB)

## Project Description

Git Intellectual Bot (GIB) is an intelligent bot designed to manage user issues in Git repositories, streamlining the development workflow through automation.

You can try out the bot on this repo by making some issues!

### Key Features:

- **Issue Analysis**: GIB reads newly opened issues and asks clarifying questions to improve the quality and completeness of the report.
- **Automatic Labeling**: GIB analyzes the issue content and assigns relevant labels automatically to categorize the issues.

GIB leverages modern AI technologies, including large language models (LLMs), to improve efficiency and accuracy in issue handling. The bot is built with flexibility in mind, allowing integration with different LLM providers and Git hosting platforms. Currently, GIB supports GitHub and the OpenAI API.

This project is part of the capstone for the Ukrainian Summer Rustcamp 2024, showcasing the skills learned during the intensive bootcamp.

## How to Run the Project

### Step 1: Create a GitHub App

We will list all the necessary steps as of 30.09.2024, but you can also follow the [official GitHub documentation](https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/registering-a-github-app) to create a GitHub app:

1. Create and log in to your GitHub account.
2. Start creating a new app using https://github.com/settings/apps/new.
3. Set up the webhook URL

GIB listens for new events in your repository via webhooks, which enable GitHub to send events to your server. You will need to set up a webhook server for this functionality.

While GIB handles the webhook server logic internally, you need to make your server accessible to GitHub. You have two options:

- **Public IP**: If you have a server with a public IP, you’re all set. (Note: Public IP addresses are usually provided by your Internet Service Provider, and may require payment.)
- **Tunneling Services**: Alternatively, you can use services like [`ngrok`](https://ngrok.com/) or [`localtunnel`](https://theboroer.github.io/localtunnel-www/) to create a tunnel to your local server. Follow their respective setup guides to generate the URL and port required.

4. Once you have your webhook URL, enter it in the "Webhook URL" field in the app creation form.

5. Now be careful, as you need to setup proper permissions for the bot. The permissions should be set up like this:

- "Repository permissions" -> "Metadata" -> "Read-only".
- "Repository permissions" -> "Issues" -> "Read and write".

6. Click "Create GitHub app".

Make sure to save the **App ID** and download the private key after creating the app.

### Step 2: Install Your App on GitHub

Refer to the [GitHub documentation](https://docs.github.com/en/apps/using-github-apps/installing-your-own-github-app) for instructions on how to install your app on your account or organization.

Once installed, navigate to your GitHub settings page to retrieve the **Installation ID**. The URL will look like this: `https://github.com/settings/installations/55411026`. The numbers at the end are the Installation ID. Save this number for later use.

### Step 3: Get an OpenAI API Key

Follow [this tutorial](https://medium.com/@lorenzozar/how-to-get-your-own-openai-api-key-f4d44e60c327) to obtain an OpenAI API key, which will be required for GIB's LLM service, or perform these steps:

1. Log in or create an account on the OpenAI website
2. Go to the "API" section
3. Go to the "Dashboard" (upper-right corner)
4. Go to the "API keys" (left menu)
5. Click "Create new secret key"
6. Click "Create secret key"
7. OpenAI will display the key

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
