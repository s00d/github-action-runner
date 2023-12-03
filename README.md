# GitHub Actions Runner

## Description

`gar` (GitHub Actions Runner) is a console utility written in Rust for working with GitHub Actions. It allows you to trigger GitHub Actions workflows right from your command line.

## Installation

You can download the pre-compiled binary from the [Releases](https://github.com/s00d/github-action-runner/releases) page on GitHub. Choose the version that suits your operating system and architecture.

If you want to compile the project from source, follow these steps:

```bash
git clone https://github.com/s00d/github-action-runner.git
cd gar
cargo build --release
```

This will create an executable file `gar` in the `target/release` directory.

## Usage

### Preparation

Before using the `gar` utility, you need to obtain a GitHub token. This can be done in the settings of your GitHub account. The token should have the following permissions:

-  Read access to `code` and `metadata`
-  Read and Write access to `actions`, `administration`, and `workflows`

Save the token in a `.github_token` file in the root directory of the project or set it as an environment variable `GAR_TOKEN`.

The utility will first look for the `GAR_TOKEN` environment variable. If it's not found, it will then look for the `.github_token` file. If neither is found, it will prompt you to enter the token manually.

### Running Workflows

To trigger a workflow, simply run the `gar` executable. The utility will automatically determine the owner and repository name based on the `origin` remote repository in your local Git repository. It will then prompt you to select a workflow to trigger from a list of available workflows in your repository.

### Global Usage

To make the `gar` executable globally available, you can move it to a directory that is in your `PATH`.

On **Linux** or **macOS**:

```bash
sudo mv gar /usr/local/bin/
```

On **Windows**:

1. Open "System" in Control Panel.
2. Click on "Advanced system settings".
3. "Advanced" tab > "Environment Variables".
4. Under "System variables" select "Path" > "Edit".
5. "New" > Enter the path to the directory where the **gar** executable is located.
6. Now you can run gar from anywhere in the command line.



Now you can run `gar` from anywhere in the command line.

## License

`gar` is distributed under the MIT license. See the `LICENSE` file for details.