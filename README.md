<h1 align="center">Rawst Download Manager</h1>

[![LBuild](https://img.shields.io/github/actions/workflow/status/Jupiee/rawst/ci.yml)]() [![Latest stable release](https://img.shields.io/github/release/jupiee/rawst.svg?maxAge=3600)](https://github.com/jupiee/rawst/releases) [![GitHub license](https://img.shields.io/github/license/jupiee/rawst.svg)](https://github.com/jupiee/rawst/blob/master/LICENSE) [![Total downloads](https://img.shields.io/github/downloads/jupiee/rawst/total.svg)](https://github.com/jupiee/rawst)

> **Content**
> - [About](#about)
> - [How to install Rawst](#-how-to-install-rawst)
> - [Usage](#%EF%B8%8F-usage)
> - [Screenshots](#screenshots)
> - [Planned features](#-planned-features)

### **About**
Snag your files efficiently with Rawst downloader, written in rust for blazingly fast execution. It's lightweight with less dependencies

### 💡 **Features**
- Sequential streamed downloads
- Concurrent downloads with multiple segments
- Multiple file downloads from a text file
- Resumable downloads support
- Recordable history
- Configurable config file
- Detailed progress bars
- Blazingly fast execution time
- Lightweight binary size

### 📦 **How to install Rawst?**
<details>
    <summary>Using cargo</summary>

- Make sure you have rust nightly installed
- Run `cargo install rawst_dl`

</details>

<details>
    <summary>Linux</summary>

- Download [Linux installer](../../releases/download/0.4.0/linux.sh) from releases and run it

</details>

<details>
    <summary>Windows</summary>

- Download [Windows installer](../../releases/download/0.4.0/windows.bat) from releases and run it

</details>

<details>
    <summary>Build from source</summary>

- **Requirements**
  - rust nightly is required
- run ``cargo build --release``
- move the binary to corresponding directories
  - Windows => ``C:\Users\%USERNAME%\AppData\Local\Microsoft\WindowsApps``
  - Linux => ``/usr/local/bin``

</details>

### ⚙️ **Usage**
```
Usage: rawst [OPTIONS] [IRIS]... [COMMAND]

Commands:
  download  Download files
  resume    Resume partial downloads
  history   Inspect download history
  config    Edit config settings
  help      Print this message or the help of the given subcommand(s)

Arguments:
  [IRIS]...
          The IRIs to download

Options:
  -v, --verbosity <VERBOSITY>


      --log-verbosity <LOG_VERBOSITY>


      --color <WHEN>
          Controls when to use color

          [default: auto]
          [possible values: auto, always, never]

  -t, --threads <THREADS>
          Maximum amount of threads used to download

          Limited to 8 threads to avoid throttling

          [default: 8]

  -i, --input-file <INPUT_FILE>
          File where to look for download IRIs

      --output-file-path <OUTPUT_FILE_PATH>
          PATH where the files are downloaded along with custom file name

          passing only custom file name without PATH will download the file with custom name in the default download directory

          eg. `foo\bar\custom_name.exe` or `custom_name.exe`

      --headers-file-path <HEADERS_FILE_PATH>
          Path to JSON file containing request headers

      --generate <GENERATOR>
          [possible values: bash, elvish, fish, powershell, zsh]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### **Screenshots**
**Download & Resume**
![WindowsTerminal_bKJ2jlrLXb](https://github.com/user-attachments/assets/5d6edebe-c5dd-437b-aac7-d88f6a44dedd)
**Multiple url downloads from a file**
![WindowsTerminal_NzPqW8o1fX](https://github.com/user-attachments/assets/b5948fc8-fbb2-4611-a9dd-a0a7453be3d2)

### 🎯 **Planned features**
* [ ] Torrent support
* [ ] Proxy support
* [ ] Scheduled downloads
* [ ] Priority downloads
* [x] Custom headers support
* [x] Resumable downloads
* [ ] Parallel downloads using cores
* [x] Download history
* [ ] Checksum with sha256
* [x] Config files
* [ ] Website link grabber
* [ ] GUI wrapper with [Iced](https://iced.rs/)
* [ ] Rewrite with better design

### **License**
[GNU General Public License v3.0](LICENSE)