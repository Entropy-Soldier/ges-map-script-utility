
use std::fs;
use std::io::prelude::*;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};

use walkdir::WalkDir;

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
        check_music_script_file( &music_script_path )?;
        println!("Existing music script file for {} is valid!", map_name);
    }

    Ok(())
}

fn create_music_script_file( args: &Arguments, map_script_path: &PathBuf ) -> Result<(), Error>
{
    let mut music_files_dir = args.rootdir.clone();
    music_files_dir.push("sound");

    let mut music_file_names = get_mp3_files_in_directory( music_files_dir )?;

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
    let mut music_script_file = fs::File::create(map_script_path)?;
    music_script_file.write_all(contents.as_bytes())?;

    Ok(())
}


use std::io::BufReader;

fn check_music_script_file( music_script_path: &PathBuf ) -> Result<(), Error>
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
    let file_re = Regex::new(r#"(?x)^\s*(("music")|(music))\s*
                        (\{
                        (
                        (\s*(("file")|(file))\s+(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*)
                        |
                        (
                        (("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*
                        \{\s*
                        (\s*(("file")|(file))\s+(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*)+
                        \}\s*
                        )
                        )+
                        \})\s*$"#).unwrap();
    
    if !file_re.is_match(&contents)
    {
        return Err(Error::new( ErrorKind::InvalidData, "Script contains core format mistake!\n  Make sure every \
                                                        bracket and quotation mark has a partner, the main section \
                                                        is labeled \"music\", each file path has a \"file\"\
                                                        section before it, no bracketed sections are empty,\
                                                        and that there are no nested bracketed sections inside\
                                                        nested bracketed sections."));
    }
    
    // If we made it here it means we have a valid file with at least one file entry.  Check those file entries
    // to make sure they're formatted correctly.

    let re = Regex::new(r#"\s*(("file")|(file))\s+(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s*"#).unwrap();

    for cap in re.captures_iter(&contents)
    {
        // We've already verified we've got a capture, and slot 4 is mandatory for us to have one.
        let file_path = &cap[4].replace("\"", ""); // Remove possible quotation marks.
        println!("File = {}", file_path);
    }

    Ok(())
}

fn get_mp3_files_in_directory( music_files_dir: PathBuf ) -> Result<Vec<String>, Error>
{
    let mut music_file_names: Vec<String> = Vec::new(); 

    // Grab the sound directory here for later.
    let music_dir_path = music_files_dir.to_str();

    if music_dir_path == None 
    {  
        return Err(Error::new( ErrorKind::InvalidInput, "Could not construct sound directory path string!"));
    }

    // We just made sure it's not None so we can unwrap it.
    let music_dir_path = music_dir_path.unwrap();

    // Make sure our sound directory exists and if so scan it for files.
    if music_files_dir.is_dir()
    {
        for entry in WalkDir::new( &music_files_dir ) 
        {
            let entry = entry?;
            let entrypath = entry.path();
            // We only want to include MP3 files in our music file listing, and only the
            // part of the path following our root path.

            // Not a file we have access to, don't worry about it.
            if !entrypath.is_file() { continue; }

            // We don't have an extension so we can't be an MP3 file!
            if entrypath.extension() == None { continue; }

            // Can unwrap to check since we already know Nonetypes have been skipped over.
            let file_extension = entrypath.extension().unwrap().to_str();

            // Make sure we can actually convert this to a normal string.
            if file_extension == None { continue }

            // We only want MP3s.
            if file_extension.unwrap().to_lowercase() != "mp3" { continue; }

            // Grab the full file path as a string so we can turn it into a relative path.
            let path_string = entrypath.to_str();
            if path_string == None { continue; }

            let path_string = path_string.unwrap();

            // The path string is a child of the sound_dir_path string, so it will always be longer.
            // With this info we cut out the parent path + the final slash to get our music script path.
            let path_string = &path_string[music_dir_path.len() + 1..];

            // The traditional standard for music scripts use forward slashes in the paths for some reason.
            // this also gives us our final String object to push into the array.
            let final_path_string = path_string.replace("\\", "/");

            music_file_names.push( final_path_string );
        }
    }

    Ok(music_file_names)
}