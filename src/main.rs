// External Crates
extern crate walkdir;
extern crate clap;
extern crate regex;
#[macro_use] extern crate lazy_static;

// Standard Library
use std::io;
use std::io::prelude::*;
use std::thread;

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
        Err(e) => { println!("[Error] failed argument parsing with error:\n{}", e); pause_then_exit( true, 0x0001 ); return; }, // Error 0x0001: invalid arguments.
    };

    if !args.fullcheck // Default program behavior, check the script files for a given map release.
    {
        create_or_verify_map_script_files( args, map_name );
    }
    else // Fullcheck behavior, verify all script files in a given GE:S install.
    {
        fullcheck_ges_directory( args );
    }
}

/// Runs on the provided rootdir, checking to make sure that every script file exists and is valid.
/// If a script file does not exist, it will be created.
fn create_or_verify_map_script_files( args: argument_handler::Arguments, map_name: String )
{
    // If we made it here, we can assume we can read our target directory and the required files
    // and directory structure are in place.  Time to start making our script files!  First let the user know.
    if args.verbose
    {
        println!( "Preparing to write script files for {}!", map_name );
    }

    // Clone the program input so rust will be happy.
    let args_maps = args.clone();
    let map_name_maps = map_name.clone();

    if args.verbose
    {
        println!( "Verifying all script files in {}!", args.gesdir.display() );
    }

    // Multithreading for the peformance boost and to take advantage of rust's nicer features.
    // The error code of each thread is added and returned at the end.
    let map_script_handle = thread::spawn( move || {
    match map_script_builder::create_or_verify_map_script_file( &args_maps, &map_name_maps )
    {
        Ok(_) => 0x0000,
        Err(e) => { println!("[Error] Failed map script section with error:\n{}\n", e); 0x0002 },
    }});

    let mut error_code = match music_script_builder::create_or_verify_music_script_file( &args, &map_name )
    {
        Ok(_) => 0x0000,
        Err(e) => { println!("[Error] Failed reslist section with error:\n{}\n", e); 0x0008 },
    };

    // We need to join here on the chance we're creating a reslist.
    // If we start making our reslist before the other files have a chance to be made, 
    // we could fail to include them in it!
    error_code += map_script_handle.join().unwrap_or(0x0002);

    error_code += match reslist_builder::create_or_verify_reslist( &args, &map_name )
    {
        Ok(_) => 0x0000,
        Err(e) => { println!("[Error] Failed reslist section with error:\n{}\n", e); 0x0008 },
    };

    // We made it to the end!  Return our error code, which is the combined result of each module that may have failed.
    pause_then_exit( !args.noexitprompt, error_code );
}

/// Runs fullcheck mode on the GE:S directory, checking every single script file for validity.
fn fullcheck_ges_directory( args: argument_handler::Arguments )
{
    let args_maps = args.clone();
    let args_music = args.clone();

    if args.verbose
    {
        println!( "Verifying all script files in {}!", args.gesdir.display() );
    }

    // Multithreading for the peformance boost and to take advantage of rust's nicer features.
    // The error code of each thread is added and returned at the end.
    let map_script_handle = thread::spawn( move || {
    match map_script_builder::fullcheck_map_script_files( &args_maps )
    {
        Ok(_) => 0x0000,
        Err(e) => { println!("[Error] Failed map script section with error:\n{}\n", e); 0x0002 },
    }});

    let music_script_handle = thread::spawn( move || {
    match music_script_builder::fullcheck_music_script_files( &args_music )
    {
        Ok(_) => 0x0000,
        Err(e) => { println!("[Error] Failed music script section with error:\n{}\n", e); 0x0004 },
    }});

    let mut error_code = match reslist_builder::fullcheck_reslist_files( &args )
    {
        Ok(_) => 0x0000,
        Err(e) => { println!("[Error] Failed reslist section with error:\n{}\n", e); 0x0008 },
    };
    
    error_code += music_script_handle.join().unwrap_or(0x0004);
    error_code += map_script_handle.join().unwrap_or(0x0002);

    // We made it to the end!  Return our error code, which is the combined result of each module that may have failed.
    pause_then_exit( !args.noexitprompt, error_code );
}

/// If enabled, provides a prompt to the user and then exits the program with the provided error code.
fn pause_then_exit( show_exit_prompt: bool, exit_code: i32 )
{
    // Prompt the user for input then proceed once that input has been given.
    if show_exit_prompt // But only if we haven't disabled it.
    {
        println!("\nPress Enter to continue.");
        let _ = io::stdin().read(&mut [0u8]);
    }

    std::process::exit( exit_code );
}