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
  history   View download history
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
          The path to the downloaded files

      --generate <GENERATOR>
          [possible values: bash, elvish, fish, powershell, zsh]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### **Screenshots**
<a href="https://ibb.co/x5K9fjz"><img src="https://i.ibb.co/nkqdncQ/Capture.png" alt="Capture" border="0"></a>
<a href="https://ibb.co/JHmQz5T"><img src="https://i.ibb.co/2dWNjgr/Capture2.png" alt="Capture2" border="0"></a>

### 🎯 **Planned features**
* [ ] Torrent support
* [ ] Proxy support
* [ ] Scheduled downloads
* [ ] Priority downloads
* [ ] Custom headers support
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