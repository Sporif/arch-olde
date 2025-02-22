`arch-olde` is a tool to show details about outdated packages in your
`Arch Linux` system using the <https://repology.org/> database.

This is a fork of [nix-olde](https://github.com/trofi/nix-olde).

# Dependencies

At runtime `arch-olde` uses 2 external packages and expects them in `PATH`:

- `curl` to fetch `repology.org` reports
- `pacman` to query locally installed packages

To build `arch-olde` you will need `rustc` and `cargo`. `Cargo.tml`
contains more detailed description of dependencies.

# Running it

Diff current system against the outdated repology repo:

```
$ arch-olde
```

Diff current system against the entire repology repo.
This takes much longer but is useful if your system contains packages older than the latest Arch Linux repos:

```
$ arch-olde --full-repo
```

# Typical output

```
$ arch-olde
...
Fetching 'repology'
Fetching 'installed'
'installed' done, took 0.03 s.
'repology' done, took 53.86 s.

Number of packages found in repology: 173

repology abseil-cpp "20250127.0" | archlinux abseil-cpp "20240722.1"
repology blake3 "1.6.0" | archlinux b3sum "1.5.5"
repology blake3 "1.6.0" | archlinux libblake3 "1.5.5"
repology clinfo "3.0.25.02.14" | archlinux clinfo "3.0.23.01.25"
repology composefs "1.0.8" | archlinux composefs "1.0.4"
repology cppdap "1.65" | archlinux cppdap "1.58.0"
repology db "18.1.40" | archlinux db5.3 "5.3.28"
...
repology virglrenderer "1.1.0" | archlinux virglrenderer "1.0.1"
repology vulkan-headers "1.4.309" | archlinux vulkan-headers "1.4.303"
repology vulkan-loader "1.4.309" | archlinux vulkan-icd-loader "1.4.303"
repology vulkan-tools "1.4.309" | archlinux vulkan-tools "1.4.303"
repology which "2.23" | archlinux which "2.21"
repology x265 "4.1" | archlinux x265 "4.0"
repology zstd "1.5.7" | archlinux zstd "1.5.6"
114 of 1111 (10.26%) installed packages are outdated according to https://repology.org.

repology jxrlib-unclassified <none> | archlinux jxrlib "0.2.4"
repology luajit <none> | archlinux luajit "2.1.1736781742"
2 of 1111 (0.18%) installed packages have no latest version at https://repology.org.
```

# Other options

There are a few options:

```
$ ./arch-olde --help

A tool to show outdated packages in current system according to repology.org database

Usage: arch-olde [OPTIONS]

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
      --full-repo   Use the full repology repo (instead of adding '&outdated=1' to the fetch url)
  -h, --help        Print help
  -V, --version     Print version
```

# How `arch-olde` works

The theory is simple: fetch data from various data sources and join
them together. Each data source returns a tuple of package name,
package version and possibly a bit of metadata around it (name in other
systems, possible status).

Currently used data sources are:

- installed packages: uses `pacman -Q`.
  Provides fields:
  * `name` (example: `7zip`)
  * `version` (example: `24.09-3`)
- <https://repology.org/> `json` database: uses
  `https://repology.org/api/v1/projects/` `HTTP` endpoint. Provides
  fields:
  * `repo`: package repository (example: "arch")
  * `visiblename`: (example: `7zip`)
  * `version` (example: `24.09-3`)
  * `status`: package status in repository (examples: "newest",
    "outdared").

# License

`arch-olde` is distributed under
[MIT license](https://opensource.org/licenses/MIT).
