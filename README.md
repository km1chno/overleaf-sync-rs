&nbsp;

<p align="center">
  <img 
    style="width: 60%;"
    src="docs/assets/banner.svg"/>
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/km1chno/overleaf-sync-rs?style=flat-square&label=latest release&color=green" alt="release" />
  <img src="https://img.shields.io/aur/version/overleaf-sync?style=flat-square&label=AUR version" alt="aur-version" />
  <img src="https://img.shields.io/github/license/km1chno/overleaf-sync-rs?style=flat-square&color=orange" alt="license" />
</p>

<p align="center"><b>CLI for synchronizing LaTeX projects between Overleaf and your local machine</b></p>

&nbsp;

## ❓ Why?

If you like working on your LaTeX projects offline or using your favourite text editor on your local machine,
but still want to use Overleaf to share your work with others and collaborate, `overleaf-sync` might be something you
find useful. It provides a bunch of git-like commands for synchronizing the state of your Overleaf projects between
local storage and Overleaf servers.

## 💡 Demo

<p align="center">
  <img 
    style="width: 95%;"
    src="docs/assets/demo.gif"/>
</p>

## 🔍 How?

`overleaf-sync` will store data in two ways. Firstly, when you first login to your Overleaf account using the tool, it will create a `~/.olsyncinfo`
file with your user details and authorization cookies. Secondly, every time you clone an existing Overleaf project with `overleaf-sync`, it will create a directory with project files and `.olsync/` folder, which keeps track of project details (you can think about it like an analog of `.git/` in git repositories).

## 🌱 Limitations

This is a fairly fresh project developed by one person during their free time, so there are some limitations you should be aware of.
Obviously, the list is dynamic and hopefully the tool will get more reliable and versatile as the project grows.

- The project not been properly field-tested yet, therefore it is **not advised to use it with crucial and/or large projects, the tool is overriding
  files both on your local machine and Overleaf servers, so unforeseen bugs can have destructive effects. <ins>The developers of `overleaf-sync` do not take
  responsibility for any harm the tool causes</ins>**.

- No one knows what happens if you try to synchronize projects that someone else is currently working on.

- Currently, **<ins>you can only push files to the root directory of project on Overleaf</ins>**, not to subdirectories. Getting rid of this limitation is planned and you can track it here [#5](https://github.com/km1chno/overleaf-sync-rs/issues/5).

## 📦 Dependencies

To build the project, you need `cargo-nightly`. In runtime you need `python`, `pipx` and `google-chrome` (used for login to Overleaf via the tool) on your system.

## 🚀 Installation

#### Build from source

```
git clone git@github.com:km1chno/overleaf-sync-rs.git
cd overleaf-sync-rs
chmod +x install.sh
./install.sh
```

By default, the binary is placed in `~/.local/bin` so make sure to add it to your `PATH`. You can modify where the binary is placed in `install.sh` script.

#### AUR

By the way, `overleaf-sync` is available on AUR repository. You can install it using your favourite AUR client like `yay`.

```
yay -Sc
yay -Sy overleaf-sync
```

## ⚙️ Features

`olsync` consists of several subcommands for authorization and interacting with Overleaf projects.

#### whoami

```
➜ olsync whoami --help
Print current session info

Usage: olsync whoami
```

#### login

```
➜ olsync login --help
Log into Overleaf account

Usage: olsync login
```

#### logout

```
➜ olsync logout --help
Log out of currently used Overleaf account

Usage: olsync logout
```

#### clone

```
➜ olsync --help clone
Clone remote project

Usage: olsync clone [OPTIONS]

Options:
  -n, --name <name>  Project name
  -i, --id <id>      Project id
```

#### pull

```
➜ olsync pull --help
Override local state with remote project

Usage: olsync pull [OPTIONS]

Options:
      --no-backup  Skip creating backup of local state before pulling
      --force      Skip confirm prompt
```

#### push

```
➜ olsync push --help
Push local files to remote project

Usage: olsync push [OPTIONS] <files>...

Arguments:
  <files>...  List of files to push

Options:
      --force  Skip confirm prompt
```

## 🤝 Feedback and contribution

We hope you like `overleaf-sync`, but if you have some ideas how the project could grow further, or want to contribute yourself,
feel free to open an issue or pull request with your propositions. The maintainers will be more than happy (in 99% of cases) to hear you out!

## 📋 License

`overleaf-sync` is licensed under the [MIT License](LICENSE).
