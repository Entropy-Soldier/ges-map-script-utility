
use std::fs;
use std::io::prelude::*;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};

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

    let mut music_files_dir = args.rootdir.clone();
    music_files_dir.push("sound");
    music_files_dir.push("music");

    let music_contents = fs::read_dir( music_files_dir )?;
    
    

    let mut contents = String::new();



    // Write out our new file!
    music_script_file.write_all(contents.as_bytes())?;

    Ok(())
}


use std::io::BufReader;

fn check_music_script_file( music_script_path: &PathBuf ) -> Result<(), Error>
{
    let music_script_file = fs::File::open(music_script_path)?;
    let reader = BufReader::new(music_script_file);

    // Need to mimic the original GE:S map script parser here since that's what will read our files\
    // ...even if it's not how I would have made it today.
    // It has a rather inflexible format with how comments and the bracketing work but is otherwise straightforward.
    // I'll probably remake the format for 5.1 in such a way that it's backwards compatable with this one and much more intuitive.

    // Surprisingly I've never gotten a complaint about this format, even though it utterly defies the standards it implies it uses.
    for line in reader.lines() 
    {
        let line = line?;
    }

    Ok(())
}