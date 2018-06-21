# GoldenEye: Source Map Script Utility

**A command line utility to prepare and check GE:S map script files, for use by map creators and server owners.**

## Features

* Automatically creates GE:S map script, music script, and reslist files if they don't already exist in the proper locations.
* Checks already existing map script, music script, and reslist files if they do exist.  Reports any issues if found.
* Can check all script files in a given GE:S install, to detect possible errors with custom maps that are already installed.
* Can automatically compress all relevant files to .bz2 format for easy uploading to a fast-download server.

## General Usage

The program is designed to be run on a map release directory.  This includes the map itself, and all the files it requires to run correctly, in a file tree that represents their final destinations within the GE:S install.  A typical map release file tree might look something like this:

```
gesource  
|    
+---maps  
|       target_map.bsp  
|         
+---scripts  
|   |   soundscapes_target_map.txt  
|   |     
|   +---maps  
|   |       target_map.txt  
|   |         
|   \---music  
|           level_music_target_map.txt  
```

After preparing or downloading such a file tree, either run ges_maprelease.exe while in the root gesource folder, or specify the root gesource directory as the first positional argument to the program.  The application will then scan through said directory, scanning any existing script files for validity, and creating any files that do not exist.  

A local GE:S install is required for complete music script scans, though syntax can still be checked without it.  If the application is failing to locate your local GE:S install, the path to it can be specified using the -g parameter.

If you're a server owner downloading a custom map, running the application with the -c parameter will individually compress all relevant files to .bz2 format following a successful script validation.  The resulting file tree can then be uploaded straight to your fast download server!  Such a command would look like this:

```
ges_scriptutility path/to/map/download/rootdir -g path/to/local/ges/install  -c
```

## Fullcheck Mode

Running the program with the -f flag will cause it to scan every script file in the specified GE:S install.  This is useful if you haven't been checking your scripts up to this point and want to make sure they're all working correctly.

```
ges_scriptutility -g path/to/target/ges/install  -f
```

## Build

* [Install Rust if not already installed](https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html)  
* Navigate to the project's root directory  
* Run the following command:

```
cargo build
```

## Contributing 

The scope of the program is rather narrow, but if there's a feature you'd like to add or a bug you'd like to fix, feel free to submit a pull request!  All contributions to this project must be licensed under the MIT license without any additional terms or conditions.

## Support

This application will be updated with each new release of the game so that it's up-to-date with the latest scripts and script formats.

## License

This project is licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

## Special Thanks

I'd like to express my thanks to the authors of the following libraries for making their respective tasks much easier than they otherwise would be!  
[walkdir](https://crates.io/crates/walkdir) - [clap](https://crates.io/crates/clap) - [bzip2](https://crates.io/crates/bzip2)