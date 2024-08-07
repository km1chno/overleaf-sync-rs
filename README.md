<h1 align="center">overleaf-sync</h1>

<p align="center">CLI for synchronizing LaTeX projects between Overleaf and your local machine 🦀 </p>

## ❓ Why and how?

If you like working on your LaTeX projects offline or using your favourite text editor on your local machine,
but still want to use Overleaf to share your work with others and collaborate, `overleaf-sync` might be something you
find useful. It provides a bunch of git-like commands for synchronizing the state of your Overleaf projects between
local storage and Overleaf servers.

`overleaf-sync` will store data in two ways. Firstly, when you first login to your Overleaf account using the tool, it will create a `~/.olsyncinfo`
file with your user details and authorization cookies. Secondly, every time you clone an existing Overleaf project with `overleaf-sync`, it will create a folder with project files
and `.olsync/` folder, which keeps track of project details (you can think about it like an analog of `.git/` in git repositories).

## 🌱 Limitations

This is a fairly fresh project developed by one person during their free time, so there are some limitations you should be aware of.
Obviously, the list is dynamic and hopefully the tool will get more reliable and versatile as the project grows.

- The project not been properly field-tested yet, therefore it is **not advised to use it with crucial and/or large projects, the tool is overriding
  files both on your local machine and Overleaf servers, so unforeseen bugs can have destructive effects. <ins>The developers of `overleaf-sync` do not take
  responsibility for any harm the tool causes</ins>**.

- No one knows what happens if you try to synchronize projects that someone else is currently working on.

- Currently, **<ins>you can only push files to the root directory of project on Overleaf</ins>**, not to subdirectories. Getting rid of this limitation is planned and you can track it here [#5](https://github.com/km1chno/overleaf-sync-rs/issues/5).

## 💡 Example usage

<p align="center">
  <img 
    style="width: 80%;"
    src="https://github.com/user-attachments/assets/ab3c9848-7b14-4ce3-9ade-caf66a7c76af"/>
</p>

## 📦 Dependencies

Make you have `python` and `pipx`, **nightly** `rustc >= 1.81.0-nightly` with `cargo` and `google-chrome` (used for login to Overleaf via the tool) on your system.

## 🚀 Installation

#### Build from source

```
git clone git@github.com:km1chno/overleaf-sync-rs.git
cd overleaf-sync-rs
./install.sh
```

#### AUR

By the way, `overleaf-sync` is available on AUR repository. You can install it using your favourite AUR client like `yay` or `aura`.

```
aura -A overleaf-sync
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
