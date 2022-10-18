# Command line tools

The software is bundled with 2 different command line software:

1. **quibitous**: the node;
2. **qcli**: Jörmungandr Command Line Interface, the helpers and primitives to run and interact with the node.

## Installation

### From a release

This is the recommended method. Releases are all available
[here](https://github.com/the-blockchain-company/quibitous/releases).

### From source

Jörmungandr's code source is available on
[github](https://github.com/the-blockchain-company/quibitous#how-to-install-from-sources).
Follow the instructions to build the software from sources.

## Help and auto completion

All commands come with usage help with the option `--help` or `-h`.

For `qcli`, it is possible to generate the auto completion with:

```sh
qcli auto-completion bash ${HOME}/.bash_completion.d
```

Supported shells are:

- bash
- fish
- zsh
- powershell
- elvish

**Note:**
Make sure `${HOME}/.bash_completion.d` directory previously exists on your HD.
In order to use auto completion you still need to:

```sh
source ${HOME}/.bash_completion.d/qcli.bash
```

You can also put it in your `${HOME}/.bashrc`.
