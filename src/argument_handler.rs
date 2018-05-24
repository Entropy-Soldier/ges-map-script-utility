use clap::{Arg, App};

use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::{Error, ErrorKind};

/// Struct that holds the core arguments of the program.
pub struct Arguments
{
    pub rootdir: PathBuf,
    pub baseweight: i32,
    pub minplayers: i32,
    pub maxplayers: i32,
    pub resintensity: i32,
    pub teamthresh: i32,
    pub verbose: bool,
}

pub fn parse_and_validate_arguments() -> Result<( Arguments, String ), Error>
{
    let program_arguments = parse_arguments();
    let map_name = get_map_name( &program_arguments );

    if program_arguments.verbose
    {
        println!( "Running with arguments:" );
        println!( "\t{} as the root directory!", program_arguments.rootdir.display() );
        println!( "\t{} as the baseweight!", program_arguments.baseweight );
        println!( "\t{} as the minplayers!", program_arguments.minplayers );
        println!( "\t{} as the maxplayers!", program_arguments.maxplayers );
        println!( "\t{} as the resintensity!", program_arguments.resintensity );
        println!( "\t{} as the teamthresh!", program_arguments.teamthresh );
        println!( "" );
        println!( "Determined map name to be {}!", map_name );
    }

    // Make sure all of our arguments make sense, exit if not.
    check_arguments( &program_arguments, &map_name )?;

    // Everything is good!  Return our valid program arguments.
    Ok((program_arguments, map_name))
}

/// Collects the arguments into an easy to reference struct.
fn parse_arguments() -> Arguments
{
    let matches = App::new("GE:S Map Release Assistant for 5.0")
        .version("1.0")
        .author("Entropy-Soldier <entropysoldierprojects@gmail.com>")
        .about("Assists with the release of GoldenEye: Source 5.0 maps by automatically creating several key files.")
        .arg(Arg::with_name("rootdir")
            .short("r")
            .long("rootdir")
            .value_name("DIRECTORY")
            .help("The root directory of your map file tree.  If none is supplied the current directory is assumed to be the root.")
            .index(1))
        .arg(Arg::with_name("weight")
            .short("w")
            .long("weight")
            .value_name("INT")
            .help("Baseweight of the map")
            .takes_value(true))
        .arg(Arg::with_name("minplayers")
            .short("n")
            .long("minplayers")
            .value_name("INT")
            .help("Minimum amount of players in the server for the map to be considered for selection")
            .takes_value(true))
        .arg(Arg::with_name("maxplayers")
            .short("x")
            .long("maxplayers")
            .value_name("INT")
            .help("Maximum amount of players in the server for the map to be considered for selection")
            .takes_value(true))
        .arg(Arg::with_name("resintensity")
            .short("s")
            .long("resintensity")
            .value_name("INT")
            .help( "Approximation of how much texture memory the map uses.  10 = 500 MB, 0 = 0 MB" )
            .takes_value(true))
        .arg(Arg::with_name("teamthresh")
            .short("t")
            .long("teamthresh")
            .value_name("INT")
            .help( "How many players need to be present before we switch to teamplay" )
            .takes_value(true))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help( "Should the program display output to inform the user of what it's doing?" )
            .takes_value(false))
        .get_matches();

    // Gets the root directory if supplied, otherwise assumes it to be the directory the program is running in.
    let rootdir_arg = match matches.value_of("rootdir")
    {
        Some(x) => PathBuf::from(x), // User specified a root directory
        None    => env::current_dir().unwrap(), // But if not we'll always have a valid current directory.
    };

    let baseweight_arg = match matches.value_of("weight").unwrap_or("500").parse::<i32>()
    {
        Ok(x) => x, // User specified a valid int
        Err(_) => { println!("[Warning] Invalid value given for baseweight!  Assuming 500."); 500}, // But if not we'll just assume a midline value   
    };

    let minplayers_arg = match matches.value_of("minplayers").unwrap_or("0").parse::<i32>()
    {
        Ok(x) => x, // User specified a valid int
        Err(_) => { println!("[Warning] Invalid value given for minplayers!  Assuming 0."); 0}, // But if not we'll just assume a midline value   
    };

    let maxplayers_arg = match matches.value_of("maxplayers").unwrap_or("16").parse::<i32>()
    {
        Ok(x) => x, // User specified a valid int
        Err(_) => { println!("[Warning] Invalid value given for maxplayers!  Assuming 16."); 16}, // But if not we'll just assume a midline value   
    };

    let resintensity_arg = match matches.value_of("resintensity").unwrap_or("7").parse::<i32>()
    {
        Ok(x) => x, // User specified a valid int
        Err(_) => { println!("[Warning] Invalid value given for resintensity!  Assuming 7."); 7}, // But if not we'll just assume a midline value   
    };

    let teamthresh_arg = match matches.value_of("teamthresh").unwrap_or("12").parse::<i32>()
    {
        Ok(x) => x, // User specified a valid int
        Err(_) => { println!("[Warning] Invalid value given for teamthresh!  Assuming 12."); 12}, // But if not we'll just assume a midline value   
    };

    let verbose_arg = matches.is_present("verbose");

    Arguments
    {
        rootdir: rootdir_arg,
        baseweight: baseweight_arg,
        minplayers: minplayers_arg,
        maxplayers: maxplayers_arg,
        resintensity: resintensity_arg,
        teamthresh: teamthresh_arg,
        verbose: verbose_arg,
    }
}

/// Infer the map name from the arguments supplied
fn get_map_name( args: &Arguments ) -> String
{
    let mut mapsdir_path = args.rootdir.clone();

    mapsdir_path.push("maps");

    match fs::read_dir( mapsdir_path )
    {
        Ok(x) => 
        {
            for pathstring in x
            {
                let path = pathstring.expect("Error during file scan of maps directory!").path();

                if path.is_file()
                {
                    if match path.extension() { Some(x) => x == "bsp", None => false }
                    {
                        match path.file_stem()
                        {
                            Some(x) => return String::from( x.to_str().unwrap() ),
                            None => {},
                        }
                    }
                }
            }
        },
        Err(_) => {}, // We don't worry about printing errors here since they'll be exposed in a more informative way in the validate function.
    }

    return String::from("invalid");
}

/// Ensure all the supplied arugments are valid and make sense.
fn check_arguments( args: &Arguments, map_name: &str ) -> Result<(), Error>
{
    // Check to make sure the root directory exists and we have read/write access to it.
    if !args.rootdir.is_dir()
    {
        if args.rootdir.is_file()
        {
            return Err(Error::new(ErrorKind::InvalidInput, "[Error] Supplied root directory is a file, not a directory!  Aborting!" ));
        }
        else
        {
            return Err(Error::new(ErrorKind::InvalidInput, "[Error] Supplied root directory isn't a valid directory with write access!  Aborting!" ));
        }
    }

    // Force the root directory to be called gesource to conform to convention and provide a
    // layer of security against typos and execution mistakes.
    match args.rootdir.as_path().file_name()
    {
        Some(x) => 
        {
            if x != "gesource"
            {
                return Err(Error::new(ErrorKind::InvalidInput, "[Error] Root directory must end in \"gesource\"!" ));
            }
        },
        None => 
        { 
            return Err(Error::new(ErrorKind::InvalidInput, "[Error] Root directory must have an ending!" ));
        },
    }

    // Make sure maps directory exists.
    let mut mapsdir = args.rootdir.clone();
    mapsdir.push("maps");

    if !mapsdir.is_dir()
    {
        return Err(Error::new(ErrorKind::InvalidInput, "[Error] Root directory has no maps directory!" ));
    }

    // Check that map file actually exists and can be read.  
    let mut map_path = mapsdir.clone();
    map_path.push( map_name );
    map_path.set_extension("bsp");

    if !map_path.is_file()
    {
        return Err(Error::new(ErrorKind::InvalidInput, "[Error] Failed to locate any readable .bsp files in maps directory!" ));
    }

    // Check to see if there's a music directory
    let mut musicdir = args.rootdir.clone();
    musicdir.push("sound");
    musicdir.push("music");

    if !musicdir.is_dir()
    {
        println!( "[Warning] Root directory {} has no music directory!  A default music file will be provided.", args.rootdir.display() );
    }

    if args.minplayers > args.maxplayers
    {
        println!( "[Warning] Minplayers is greater than maxplayers!  
                   Your map will never be picked for normal rotation." );
    }
    else if args.maxplayers < 0 || args.minplayers > 16
    {
        println!( "[Warning] Your player range is outside the possible range of playercounts.  
                   Your map will never be picked for normal rotation." );
    }

    if args.resintensity <= 0
    {
        println!( "[Warning] Your resintensity is an impossibly low value!  
                   While this will make servers switch to it more often, it will also cause client crashes." );
    }
    else if args.resintensity > 8
    {
        println!( "[Warning] Your resintensity is incredibly high!  If your map really has > 400MB worth of 
                    assets it needs to load into RAM it would be best to cut some content instead of setting 
                    this value above 8." );        
    }

    Ok(())
}