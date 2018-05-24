// External Crates
extern crate walkdir;
extern crate clap;

//use walkdir::WalkDir;

// Internal Modules
mod argument_handler;
mod map_script_builder;
mod music_script_builder;
mod reslist_builder;

fn main()
{
    let (program_arguments, map_name) = match argument_handler::parse_and_validate_arguments()
    {
        Ok(x) => x,
        Err(e) => { println!("{}", e); std::process::exit(0x0001); }, // Error 0x0001: invalid arguments.
    };

    // If we made it here, we can assume we can read our target directory and the required files
    // and directory structure are in place.  Time to start making our script files!  First let the user know.
    if program_arguments.verbose
    {
        println!( "Preparing to write script files for {}!", map_name );
    }

    match map_script_builder::create_or_verify_map_script_file( &program_arguments, &map_name )
    {
        Ok(_) => {},
        Err(e) => { println!("Failed map script section with error:\n{}", e); std::process::exit(0x0002); }, // Error 0x0002: Failed to create map script.
    }

    match music_script_builder::create_or_verify_music_script_file( &program_arguments, &map_name )
    {
        Ok(_) => {},
        Err(e) => { println!("Failed music script section with error:\n{}", e); std::process::exit(0x0004); }, // Error 0x0002: Failed to create map script.
    }

    match reslist_builder::create_or_verify_reslist( &program_arguments, &map_name )
    {
        Ok(_) => {},
        Err(e) => { println!("Failed reslist section with error:\n{}", e); std::process::exit(0x0008); }, // Error 0x0002: Failed to create map script.
    }
    
}