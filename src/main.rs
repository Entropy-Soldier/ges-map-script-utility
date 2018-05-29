// External Crates
extern crate walkdir;
extern crate clap;
extern crate regex;
#[macro_use] extern crate lazy_static;

// Standard Library
use std::io;
use std::io::prelude::*;

// Internal Modules
mod argument_handler;
mod map_script_builder;
mod music_script_builder;
mod reslist_builder;
mod shared;

fn main()
{
    let (args, map_name) = match argument_handler::parse_and_validate_arguments()
    {
        Ok(x) => x,
        Err(e) => { println!("[Error] failed argument parsing with error:\n{}", e); std::process::exit(0x0001); }, // Error 0x0001: invalid arguments.
    };


    if !args.fullcheck // Default program behavior, check the script files for a given map release.
    {
        create_or_verify_map_script_files( &args, &map_name );
    }
    else // Fullcheck behavior, verify all script files in a given GE:S install.
    {
        fullcheck_ges_directory( &args );
    }
}

fn create_or_verify_map_script_files( args: &argument_handler::Arguments, map_name: &str )
{
    // If we made it here, we can assume we can read our target directory and the required files
    // and directory structure are in place.  Time to start making our script files!  First let the user know.
    if args.verbose
    {
        println!( "Preparing to write script files for {}!", map_name );
    }

    match map_script_builder::create_or_verify_map_script_file( args, map_name )
    {
        Ok(_) => {},
        Err(e) => { println!("[Error] Failed map script section with error:\n{}", e); pause_then_exit( args, 0x0002); }, // Error 0x0002: Failed to create map script.
    }

    match music_script_builder::create_or_verify_music_script_file( args, map_name )
    {
        Ok(_) => {},
        Err(e) => { println!("[Error] Failed music script section with error:\n{}", e); pause_then_exit( args, 0x0004); }, // Error 0x0004: Failed to create music script.
    }

    match reslist_builder::create_or_verify_reslist( args, map_name )
    {
        Ok(_) => {},
        Err(e) => { println!("[Error] Failed reslist section with error:\n{}", e); pause_then_exit( args, 0x0008); }, // Error 0x0008: Failed to create reslist
    }

    // We made it without any errors!  We've done all we can and have earned the exit code 0.
    pause_then_exit( args, 0x0000 );
}

fn fullcheck_ges_directory( args: &argument_handler::Arguments )
{
    match map_script_builder::fullcheck_map_script_files( args )
    {
        Ok(_) => {},
        Err(e) => { println!("[Error] Failed map script section with error:\n{}", e); pause_then_exit( args, 0x0002); },
    }

    match music_script_builder::fullcheck_music_script_files( args )
    {
        Ok(_) => {},
        Err(e) => { println!("[Error] Failed music script section with error:\n{}", e); pause_then_exit( args, 0x0004); },
    }

    match reslist_builder::fullcheck_reslist_files( args )
    {
        Ok(_) => {},
        Err(e) => { println!("[Error] Failed reslist section with error:\n{}", e); pause_then_exit( args, 0x0008); },
    }
    
    // We made it without any errors!  We've done all we can and have earned the exit code 0.
    pause_then_exit( args, 0x0000 );
}

fn pause_then_exit( args: &argument_handler::Arguments, exit_code: i32 )
{
    // Prompt the user for input then proceed once that input has been given.
    if !args.noexitprompt // But only if we haven't disabled it.
    {
        println!("\nPress Enter to close.");
        let _ = io::stdin().read(&mut [0u8]);
    }

    std::process::exit( exit_code );
}