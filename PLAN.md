# Purpose

Boost performance of maintaining GitHub (or any other service) repository.

The bot should automate tasks, that are made by humans.

The bot utilizes modern AI techonologies such as embedding and large language models.

The bot should be retargetable, so it means it can be plugged up to any GitHub repository. It should learn available information from the repo's issues, and also it should learn along the way when new issues are added.

# Features

## Label issues

### How this will look

The bot will learn what are the available issues in the repo, and for each new issue it will try to label it.

The bot will send a message like: "Possible labels `...`, `...`, `...`".

After that, the collaborator can decide on it's own whether the labels are useful. To apply the labels, the collaborator can write either:
- An allow list: "+`...`, `...`, `...`" - only label with `+` will be added.
- A deny list: "+`...`, `...`, `...`" - every label will be added, except ones mentioned in the deny list.

### Why this is useful

Saves time for labeling issues. Labeling issues gives more opportunities to search, classification, and statistics.

### Algorithm

A better way to do this is to make a multi-labelled classification model. But it's a topic for a deep research.

A more faster way to develop is to supply LLM with issue body and available labels, and then parse LLM output and propose the labels.

### Appendix

To further eliminate human assistance, we can ask LLM to separate labels into two categories: "definitely" labels and "possibly" labels. "Definitely" will be applied right away, and "possibly" will be decided by collaborators.

## Improve user issues

### How this will look

For each new issue, the bot will carefully read it, and if it's underdeveloped, or requires more information, it will ask that for user.

For example, user has encountered a bug, but the user hasn't supplied enough information. The bot will analyze the issue body, and write message: "Could you please tell us the OS you were using? Also which version of program had you used? Did you tried to test the program with the latest development branch?".

### Why this is useful

Computers and LLM often faster than humans. So important information can be extracted quickly.

### Algorithm

We only need to make a good LLM prompt. And after that supply the issue body. That's it, it's the simplest feature.

## Find duplicates of issues

### How this will look

The bot will learn the issues of a repo, and for each new issue the bot will check, if there is a similar issue that was raised previously.

In case bot finds it, the bot will send message "Possible duplicate of #..., #...".

After that the collaborator can decide on its own.

### Why this is useful

Collaborators save time for searching, as the bot provides them with the relevant results.

### Algorithm

The algorithm consists of two stages:
- Finding similar issues using vector search: bot will generate the embedding of the body of the original issue and all past issues (they will be stored already in DB). Then using vector search, it will grab `N` (for example, `N = 5`) most similar issues.
- Ask the LLM is the issue is a duplicate. For each found issue from the previous step we supply the LLM with the body of the original issue and found similar issues. After that the LLM will finally decide whether the issues are duplicates.

Vector search step is needed because the could be thousands of issues in the repo. It's too expensive to use LLM on each of them.

LLM step is required for final verification, so that no false positive errors occurs.

The algorithm could be researched more, for example, we can set a threshold on vector similarity, and that way we can eliminate the second step at all. But it's solid enough.

## Answer to FAQs

### How this will look

The bot will learn FAQs that should be supplied.

For every new issue on GitHub (or new discussion), the bot will look for the questions, and then, if that question is frequently asked, it will tell the user the stored answer.

### Why this is useful

Eliminating human interaction. There are different types of users on GitHub, and let's agree most of the time we are too lazy to look into the documentation or FAQ page of a project.

### [TODO] Algorithm

Need to think more. But probably something that uses vector search for questions. And then we just pass the AI with the issue body, questions and answers, and then it will finally decide:
- Which questions are relevant.
- How to tailor answer to the specific issue raised by user.

Also need to think of a way to supply FAQs...



MAYBE: Let bot comment on every push in pull request and look at the file difference, then make a comment describing those differences.
