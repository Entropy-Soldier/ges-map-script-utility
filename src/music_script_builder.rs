
use std::fs;
use std::io::prelude::*;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::io::BufReader;

use shared;

use regex::Regex;

/// Generates the music script file used for music selection on the map
/// Returns Ok() if successful and an error if not.
pub fn create_or_verify_music_script_file( args: &Arguments, map_name: &str ) -> Result<(), Error>
{
    let mut music_script_dir = args.rootdir.clone();
    music_script_dir.push("scripts");
    music_script_dir.push("music");

    if !music_script_dir.is_dir()
    {
        fs::create_dir_all(&music_script_dir)?;
    }

    // Just build the music script path off of the existing script dir builder.
    let mut music_script_path = music_script_dir;

    let mut music_script_name = String::new();

    music_script_name.push_str("level_music_");
    music_script_name.push_str(map_name);

    music_script_path.push( music_script_name );
    music_script_path.set_extension("txt");

    if !music_script_path.is_file()
    {
        create_music_script_file( args, &music_script_path )?;
        println!("Created music script for {}!", map_name);
    }
    else
    {
        check_music_script_file( args, &music_script_path )?;
        println!("Existing music script file for {} is valid!", map_name);
    }

    Ok(())
}

/// Checks every music script in the provided or autodetected GE:S directory.
pub fn fullcheck_music_script_files( args: &Arguments ) -> Result<(), Error>
{
    let mut music_script_dir = args.gesdir.clone();
    music_script_dir.push("scripts");
    music_script_dir.push("music");

    if !music_script_dir.is_dir()
    {
        return Err(Error::new( ErrorKind::InvalidData, "Music script directory does not exist!  Is this really a valid GE:S install?" ));
    }

    shared::check_all_files_in_dir_with_func( args, &music_script_dir, "txt", "music scripts", check_music_script_file )?;

    Ok(())
}

/// Creates a music script file at the given path using the files provided in the sound directory.
/// If none are provided, it will create a default script instead.
fn create_music_script_file( args: &Arguments, music_script_path: &PathBuf ) -> Result<(), Error>
{
    let mut music_files_dir = args.rootdir.clone();
    music_files_dir.push("sound");

    let mut music_file_names = shared::get_files_in_directory( &music_files_dir, "mp3", &[] )?;

    // We don't have a sound directory, or it's empty, so let's provide some example music instead!
    if music_file_names.is_empty() 
    {
        music_file_names.push(String::from("music/classy.mp3"));
        music_file_names.push(String::from("music/spy.mp3"));
        music_file_names.push(String::from("music/always_better.mp3"));
        music_file_names.push(String::from("music/shaken_and_stirred.mp3"));
        music_file_names.push(String::from("music/martini.mp3"));
        music_file_names.push(String::from("music/standard_operating_procedure.mp3"));
    }

    // Now use our collected map names to write out our file contents.
    let mut contents = String::new();
    contents.push_str("\"music\"\r\n");
    contents.push_str("{\r\n");

    for music_file in music_file_names
    {
        contents.push_str("\t\"file\"\t\""); contents.push_str(&music_file); contents.push_str("\"\r\n");
    }

    contents.push_str("}\r\n");

    // Make it official and write the final string to the file.
    let mut music_script_file = fs::File::create(music_script_path)?;
    music_script_file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Ensures that the music script file follows the correct format and that every file reference is valid.
fn check_music_script_file( args: &Arguments, music_script_path: &PathBuf ) -> Result<(), Error>
{
    let music_script_file = fs::File::open(music_script_path)?;
    let mut reader = BufReader::new(music_script_file);

    let mut contents = String::new();
    reader.read_to_string( &mut contents )?;

    // We'll use regular expressions to verify our format.
    // We will have a music tag to start our file, then a large bracketed section.
    // the bracketed section may have addtional bracketed sections inside it for X music and 
    // area specific music, but these sections will not contain more bracketed sections.
    // The main bracketed section and the subsections will contain lines like so:
    // "file"   "[path/to/file]"
    // where "file" is the exact text that appears in that part of the line and [path/to/file]
    // contains the path where that file can be found.
    // This makes a regex one of the more clean ways to verify the format is followed and then
    // scan the individual entries to make sure the tracks are entered correctly.
    // [^"\{\}] for every character that isn't a control character and
    // [\S&&[^"\{\}]] for every non-whitespace character that isn't a control character.
    // Lazy static is used to allow for compiler optimizations and to ensure costly regexs aren't compiled
    // multiple times.
    lazy_static!
    {
        static ref FILE_RE: Regex = Regex::new(r#"(?x)^\s*(("music")|(music))\s*
                                        (\{\s*
                                        (
                                        (\s*(("file")|(file))\s+(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*)
                                        |
                                        (
                                        (("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*
                                        \{\s*
                                        (\s*(("file")|(file))\s+(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*)+
                                        \}\s*
                                        )
                                        )*
                                        \})\s*$"#).unwrap();
    }

    if !FILE_RE.is_match(&contents)
    {
        return Err(Error::new( ErrorKind::InvalidData, "Script contains core format mistake!\n  Make sure every \
                                                        bracket and quotation mark has a partner, the main section \
                                                        is labeled \"music\", each file path has a \"file\"\
                                                        section before it, no bracketed sections are empty,\
                                                        and that there are no nested bracketed sections inside\
                                                        nested bracketed sections."));
    }

    // Now let's make sure the music paths are valid!  This involves checking the script paths against the GE:S
    // install and the files in the local directory tree.

    let mut gesource_sound_dir = args.gesdir.clone();
    gesource_sound_dir.push("sound");

    // Couldn't locate sound directory...which in pretty much all cases means that the gesdir isn't valid either
    // and it was mentioned in the program arguments checker.  If not, and the user for some reason has a corrupted
    // GE:S install somehow, the error message still makes a fair bit of sense.
    if !gesource_sound_dir.is_dir()
    {
        println!("[Warning] Without a valid GE:S directory, music file paths will not be checked, though file format will be!");
        return Ok(()); // We've already checked all we can without a GE:S music directory to cross reference our paths with.
    }

    let mut local_music_files_dir = args.rootdir.clone();
    local_music_files_dir.push("sound");

    // Get all possible mp3 files that we can use.
    // You might wonder why this is preferable to just checking if the MP3 files in the script are valid files
    // on an as-needed basis.  Well, this would normally be ideal, but the assumption is that if a file is in
    // the sound directory it will probably be used, so we might as well scan them all at once.  This breaks down
    // a bit with the inclusion of scanning the local GE:S sound directory as well, but it does shave off a large
    // amount of syscalls on fullcheck mode and lets us share a lot of code between us and the reslist checker.
    let mp3_files = generate_mp3_directory_tree( &gesource_sound_dir, &local_music_files_dir, "mp3" )?;

    // If we made it here it means we have a valid file with at least one file entry.  Check those file entries
    // to make sure they're formatted correctly and point to a valid music file.

    lazy_static!
    {
        static ref RE: Regex = Regex::new(r#"\s*(("file")|(file))\s+(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*"#).unwrap();
    }

    for cap in RE.captures_iter(&contents)
    {
        // We've already verified we've got a capture, and slot 4 is mandatory for us to have one.
        let fixed_path = cap[4].replace("\"", "").replace("\\", "/").to_lowercase(); // Remove possible quotation marks and standardize slashes.

        // Make sure we're an mp3...or are at least claiming to be.
        if fixed_path.len() > 3 && fixed_path[fixed_path.len()-3..fixed_path.len()].to_lowercase() != "mp3"
        {
            let mut error_text = String::new();
            error_text.push_str("File ");
            error_text.push_str(&fixed_path);
            error_text.push_str(" is not an MP3 file!  Please convert it to mp3 format.");

            return Err(Error::new(ErrorKind::InvalidData, error_text ));
        }

        // Check to see if our MP3 file is one of the files we've detected in the relevant directories.
        // if not, our script is pointing to an invalid file and isn't ready for release!
        if !mp3_files.contains(&fixed_path)
        {
            let mut error_text = String::new();
            error_text.push_str("Failed to locate music file ");
            error_text.push_str(&fixed_path);
            error_text.push_str(" in either the GE:S or local directory tree\nEnsure that the file path is valid and that the file exists.");

            return Err(Error::new(ErrorKind::InvalidData, error_text ));
        }
    }

    // We made sure the file format is correct and checked all the files for validity!
    // Our music script file is ready for release!
    Ok(())
}

use std::sync::Mutex;

/// Provides a reference to a vector storing strings that correspond to the relative paths of every file in
/// the provided directory.  Subsequent calls return the cached value of the first call.
pub fn generate_mp3_directory_tree( gesource_sound_dir: &PathBuf, local_sound_dir: &PathBuf, target_type: &str ) -> Result<&'static Vec<String>, Error>
{
    lazy_static!
    {
        static ref DIRLIST_INIT_STATE: Mutex<bool> = Mutex::new(false);
    }

    static mut DIRLIST: Option<Vec<String>> = None;

    // Unsafe because the alternative is more convoluted to use, the possibility of a data race is almost 0,
    // and the negative outcome of one would be a performance penalty and nothing else.
    unsafe
    {
        let mut dirs_to_scan = vec![gesource_sound_dir];

        // Don't try to collect local sound files if we don't have a sound directory...which is very
        // possible if the map uses entirely default music.
        if local_sound_dir.is_dir() && local_sound_dir != gesource_sound_dir
        {
            dirs_to_scan.push(local_sound_dir);
        }

        return shared::compute_or_get_safe_reference_to_directory_cache( dirs_to_scan, target_type, &[], &DIRLIST_INIT_STATE, &mut DIRLIST );
    }
}