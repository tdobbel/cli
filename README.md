This repository contains a bunch of random utility scripts that I developed over time. Some of them provide useful functionalities for my workflow, others are just silly. A few of them are implemented in several languages just for fun. 

## Bash scripts

| Script name | Description |
| ---- | --- |
| `copy-content` | Copies the content of a file to the clipboard. This one is jut a stupid one-liner using `xclip`. I could have just used an alias |
|`count-files` | Emulate the function used to count files on the geo servers, by ignoring a bunch of files and directories |
| `ftp-dobby`| Provide functions to connect, upload and download files to/from the FTP servers that I used to share large files with the US |
| `make_gif`| Converts frames to a gif/video using `ffmpeg`. |
| `pdf-tools`| Provides functions to manipulate PDF files, such as merging, extracting pages, and compressing using `ghostscript`. |
| `pyplot-cmaps`| Show matplotlib colormaps in a terminal. |
| `pyplot-colors`| Show matplotlib named colors in a terminal. |
| `shwiq`| Implementation of `wiq` in Bash (see below) |

## Python scripts

Scripts to extract:
- land polygons and coastlines from OpenStreetMap
- reef polygons from UNEP

within a given bounding box or over the extent of a mesh file.

## Other projects in Rust (and other languages)

### sort-bib

CLI tool to sort the entries of a BibTeX file alphabetically (implemented in Rust, Python, C and Zig)

### soupes

Extract the soups of the week from the [D'un Pain à l'Autre](https://www.uclouvain.be/fr/resto-u/d-un-pain-a-l-autre-lln) website. Also checks if the restaurant is closed using a regex (so probably not robust at all...). (implemented in Rust and Python).

### tree

A reimplementation of the `tree` command in Rust. I developed this one because `tree` was not available ion the geo servers.

###  wiq (Who In Queue)

A CLI tool that reads the queue on SLURM and gives a sorted list of the users in the queue, with the number of running/pending jobs, and the partitions on which they were submitted. (Implemented in Rust, Python, C, Zig and Bash)

### tetris

A TUI tetris game implemented in Rust using `rata-tui`.
![tetris screenshot](tetris/tetris-screenshot.png)

