
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
    let mut music_script_file = fs::File::create(map_script_path)?;

    let mut music_file_names: Vec<String> = Vec::new(); 
    let mut music_files_dir = args.rootdir.clone();
    music_files_dir.push("sound");

    // Grab the sound directory here for later.
    let sound_dir_path = music_files_dir.to_str();

    if sound_dir_path == None 
    {  
        return Err(Error::new( ErrorKind::InvalidInput, "Could not construct sound directory path string!"));
    }

    // We just made sure it's not None so we can unwrap it.
    let sound_dir_path = sound_dir_path.unwrap();

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
            let path_string = &path_string[sound_dir_path.len() + 1..];

            // The traditional standard for music scripts use forward slashes in the paths for some reason.
            // this also gives us our final String object to push into the array.
            let final_path_string = path_string.replace("\\", "/");

            music_file_names.push( final_path_string );
        }
    }

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

    // Find the first open bracket and the last closing bracket to form a pair.
    let open_bracket_index_1 = match contents.find("{")
    {
        Some(x) => x,
        None => return Err(Error::new( ErrorKind::InvalidData, "Could not locate opening bracket of \"music\" block!")),
    };

    let close_bracket_index_1 = match contents.rfind("}")
    {
        Some(x) => x,
        None => return Err(Error::new( ErrorKind::InvalidData, "Could not locate closing bracket of \"music\" block!")),
    };

    if contents[..open_bracket_index_1].trim() != "\"music\""
    {
        return Err(Error::new( ErrorKind::InvalidData, "Root block is not labeled \"music\"!"))
    }

    let music_block_contents = &contents[open_bracket_index_1 + 1..close_bracket_index_1];

    if music_block_contents.trim().is_empty()
    {
        return Err(Error::new( ErrorKind::InvalidData, "No music is defined!"))
    }

    Ok(())
}