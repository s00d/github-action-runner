# GitHub Actions Runner

![img](https://github.com/s00d/github-action-runner/blob/main/assets/2023-12-04%2022.57.59.gif?raw=true)

## Description

`gar` (GitHub Actions Runner) is a command-line utility written in Rust for working with GitHub Actions. It allows you to run GitHub Actions workflows directly from the command line.

## Installation

You can download a precompiled binary from the [Releases page](https://github.com/s00d/github-action-runner/releases) on GitHub. Choose the version that matches your operating system and architecture.

If you want to compile the project from source, follow these steps:

```bash
git clone https://github.com/s00d/github-action-runner.git
cd gar
cargo build --release
```

This will create an executable `gar` file in the `target/release` directory.

## crates.io

Before installing the `github-action-runner` package, you need to install Rust. Rust is a programming language that the package is built with. Here are the steps to install Rust:

1. Open a terminal or command prompt.

2. Visit the official Rust website at [https://www.rust-lang.org/](https://www.rust-lang.org/).

3. Follow the instructions on the website to download and install Rust for your operating system.

4. After the installation is complete, verify that Rust is installed correctly by running the following command in your terminal:

```shell
rustc --version
```

You can install the `github-action-runner` package using the `cargo` utility. Make sure you have Rust compiler and `cargo` tool installed.

1. Open a terminal or command prompt.

2. Run the following command to install the package:

```shell
cargo install github-action-runner
```


## Usage

### Preparation

Before using the `gar` utility, you need to obtain a GitHub token. This can be done in the settings of your account on GitHub. The token should have the following permissions:

-  Read `code` and `metadata`
-  Read and write `actions`, `administration`, and `workflows`

Save the token in a `.github_token` file in the root directory of the project or set it as an environment variable `GAR_TOKEN`.

The utility will first look for the `GAR_TOKEN` environment variable. If it is not found, it will then look for the `.github_token` file. If neither is found, it will prompt you to enter the token manually.

### Running Workflows

To run a workflow, simply execute the `gar` file. The utility will automatically determine the owner and repository name based on the `origin` remote repository in your local Git repository. It will then prompt you to select a workflow to run from a list of available workflows in your repository.

### Global Usage

To make the `gar` executable globally available, you can move it to a directory that is in your `PATH`.

On **Linux**:

```bash
sudo mv gar /usr/local/bin/
chmod +x /usr/local/bin/gar
```

On  **macOS**:

```bash
sudo mv gar /usr/local/bin/
chmod +x /usr/local/bin/gar
xattr -cr /usr/local/bin/gar
```


On **Windows**:

1. Open "System" in Control Panel.
2. Click on "Advanced system settings".
3. "Advanced" tab > "Environment Variables".
4. In the "System variables" section, select "Path" > "Edit".
5. "New" > Enter the path to the directory where the **gar** executable is located.
6. Now you can run gar from anywhere in the command line.

Now you can run `gar` from anywhere in the command line.

## Args

| Argument   | Short | Description                                                                                          | Default Value                           |
|------------|-------|------------------------------------------------------------------------------------------------------|-----------------------------------------|
| `--ref`    | `-r`  | The name of the repository branch on which the action will be run.                                   | The name of the current Git branch      |
| `--owner`  | `-o`  | The owner of the repository where the action is located.                                             | The owner of the current Git repository |
| `--repo`   | `-p`  | The name of the repository where the action is located.                                              | The name of the current Git repository  |
| `--token`  | `-t`  | The token used for authentication. If not provided, the GAR_TOKEN environment variable will be used. | None                                    |
| `--inputs` | `-i`  | The name of the event that triggers the action.                                                      | An empty string                         |

Please note that all the parameters are optional, and if not provided, default values will be used.

```
gar --ref <branch-name>
gar --owner <owner-name> --repo <repository-name>
gar --token <personal-access-token>
gar --inputs <input-name1>=<value1>,<input-name2>=<value2>
```

## History Command

The `gar history` command provides a historical record of the runs of a selected workflow. Here's an example of how you might use it:

```bash
$ gar history
```

When you run this command, it will first prompt you to select a workflow from your repository. After you select a workflow, it will fetch and display the last 10 runs of the selected workflow in a table format.

Here's an example of what the output might look like:

```text
ID         Branch   Status   Conclusion   Created At              Updated At              Url
123456789  main     completed success      2023-04-12T23:05:34Z   2023-04-12T23:06:00Z   https://github.com/owner/repo/actions/runs/123456789
987654321  feature  completed failure      2023-04-12T22:05:34Z   2023-04-12T22:06:00Z   https://github.com/owner/repo/actions/runs/987654321
...

```

Each row in the table represents a single run of the workflow. The columns provide the following information:

- `ID`: The unique identifier of the run.
- `Branch`: The branch where the run was triggered.
- `Status`: The status of the run (e.g., `completed`, `in_progress`, `queued`).
- `Conclusion`: The outcome of the run if it has completed (e.g., `success`, `failure`). If the run is not yet completed, this field will show `N/A`.
- `Created At`: The time when the run was created.
- `Updated At`: The time when the run was last updated.
- `Url`: The URL where you can view the run on GitHub.

## Help

For more information about the available commands and options, you can refer to the package documentation or run github-action-runner --help in your terminal.

## Action example

https://github.com/s00d/github-action-runner/blob/main/.github/workflows/runner.yml

## License

`gar` is distributed under the MIT license. See the `LICENSE` file for details.