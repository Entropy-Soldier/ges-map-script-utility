use clap::{Arg, App};

use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::{Error, ErrorKind};

/// Struct that holds the core arguments of the program.
#[derive(Clone)]
pub struct Arguments
{
    pub rootdir: PathBuf,
    pub gesdir: PathBuf,
    pub baseweight: i32,
    pub minplayers: i32,
    pub maxplayers: i32,
    pub resintensity: i32,
    pub teamthresh: i32,
    pub compress: bool,
    pub recompress: bool,
    pub verbose: bool,
    pub fullcheck: bool,
    pub noexitprompt: bool,
}

/// Takes the program arguments input by the user, validates them, and returns them as an Arguments object.
/// Also infers the map name.
pub fn parse_and_validate_arguments() -> Result<( Arguments, String ), Error>
{
    let program_arguments = parse_arguments();
    let map_name = get_map_name( &program_arguments );

    if program_arguments.verbose
    {
        if program_arguments.fullcheck
        {
            println!( "Running in fullcheck mode with arguments:" );
        }
        else
        {
            // If it failed to find the map name it just prints "map determined to be invalid" which still makes sense.
            println!( "Running on map determined to be {} with arguments:", map_name ); 
        }

        println!( "\t{} as the root directory!", program_arguments.rootdir.display() );
        println!( "\t{} as the GE:S directory!", program_arguments.gesdir.display() );
        println!( "\t{} as the baseweight!", program_arguments.baseweight );
        println!( "\t{} as the minplayers!", program_arguments.minplayers );
        println!( "\t{} as the maxplayers!", program_arguments.maxplayers );
        println!( "\t{} as the resintensity!", program_arguments.resintensity );
        println!( "\t{} as the teamthresh!", program_arguments.teamthresh );
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
        .version("0.9")
        .author("Entropy-Soldier <entropysoldierprojects@gmail.com>")
        .about("Assists with the release of GoldenEye: Source 5.0 maps by automatically creating several key files.")
        .arg(Arg::with_name("rootdir")
            .short("r")
            .long("rootdir")
            .value_name("DIRECTORY")
            .help("The root directory of your map file tree.  If none is supplied the current directory is assumed to be the root.")
            .index(1))
        .arg(Arg::with_name("gesdir")
            .short("g")
            .long("gesdir")
            .value_name("DIRECTORY")
            .help("The root directory of your GE:S install.  If none is supplied the standard locations are searched.")
            .takes_value(true))
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
        .arg(Arg::with_name("fullcheck")
            .short("f")
            .long("fullcheck")
            .help( "With this flag set, the program will instead not do map release checks but instead check all script files in the supplied or detected GE:S directory.  Good for server owners who want to check all of their script files at once." )
            .takes_value(false))
        .arg(Arg::with_name("compress")
            .short("c")
            .long("compress")
            .help( "Generate bzipped version of all relevant files for server upload." )
            .takes_value(false))
        .arg(Arg::with_name("recompress")
            .short("z")
            .long("recompress")
            .help( "Same as compressed, but will delete all existing compressed files before starting.  Its usage implies the compressed flag." )
            .takes_value(false))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help( "Should the program display output to inform the user of what it's doing?" )
            .takes_value(false))
        .arg(Arg::with_name("noexitprompt")
            .short("e")
            .long("noexitprompt")
            .help( "Don't wait for user input to close the program after it finishes, do so immediately." )
            .takes_value(false))
        .get_matches();


    // Fullcheck mode triggers different program behavior and makes the root directory the same as the GE:S directory.
    // If such a mode is enabled, make sure this change is reflected.
    let fullcheck_arg = matches.is_present("fullcheck");

    // Gets the ges directory if supplied, otherwise assumes it to be in one of the default locations.
    let gesdir_arg = match matches.value_of("gesdir")
    {
        Some(x) => PathBuf::from(x), // User specified a ges directory
        None    =>                   // If not let's search for one
        { 
            // gesource MUST be installed in one of these two locations due to a source mod limitation...
            // at least it makes it easy to find.
            let mut ges_path = PathBuf::from("C:\\Program Files (x86)\\Steam\\steamapps\\sourcemods\\gesource\\");
            
            // If it's not in the first location it must be in the second...if not then we'll notice
            // during the next step where we check argument validity.
            if !ges_path.is_dir()
            {
                ges_path = PathBuf::from("C:\\Program Files\\Steam\\steamapps\\sourcemods\\gesource\\");
            }

            ges_path
        }, 
    };

    let rootdir_arg;

    if fullcheck_arg
    {
        // Rootdir is just the gesdir.
        rootdir_arg = gesdir_arg.clone();
    }
    else
    {
        // Gets the root directory if supplied, otherwise assumes it to be the directory the program is running in.
        rootdir_arg = match matches.value_of("rootdir")
        {
            Some(x) => PathBuf::from(x), // User specified a root directory
            None    => env::current_dir().unwrap(), // But if not we'll always have a valid current directory.
        };
    }

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

    let noexitprompt_arg = matches.is_present("noexitprompt");

    let recompress_arg = matches.is_present("recompress");

    // recompress implies compress
    let compress_arg = matches.is_present("compress") || recompress_arg;

    Arguments
    {
        rootdir: rootdir_arg,
        gesdir: gesdir_arg,
        baseweight: baseweight_arg,
        minplayers: minplayers_arg,
        maxplayers: maxplayers_arg,
        resintensity: resintensity_arg,
        teamthresh: teamthresh_arg,
        compress: compress_arg,
        recompress: recompress_arg,
        verbose: verbose_arg,
        fullcheck: fullcheck_arg,
        noexitprompt: noexitprompt_arg,
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
                            Some(x) => return String::from( x.to_str().expect("Encountered invalid BSP name when reading maps directory.") ),
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
    // If we're in fullcheck mode we're not actually releasing a map and don't care about the root directory
    if !args.fullcheck
    {
        // Check to make sure the root directory exists and we have read/write access to it.
        if !args.rootdir.is_dir()
        {
            if args.rootdir.is_file()
            {
                return Err(Error::new(ErrorKind::InvalidInput, "Supplied root directory is a file, not a directory!  Aborting!" ));
            }
            else
            {
                return Err(Error::new(ErrorKind::InvalidInput, "Supplied root directory isn't a valid directory with write access!  Aborting!" ));
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
                    return Err(Error::new(ErrorKind::InvalidInput, "Root directory must end in \"gesource\"!" ));
                }
            },
            None => 
            { 
                return Err(Error::new(ErrorKind::InvalidInput, "Root directory must have an ending!" ));
            },
        }
        
        // Make sure maps directory exists.
        let mut mapsdir = args.rootdir.clone();
        mapsdir.push("maps");

        if !mapsdir.is_dir()
        {
            return Err(Error::new(ErrorKind::InvalidInput, "Root directory has no maps directory!" ));
        }

        // Check that map file actually exists and can be read.  
        let mut map_path = mapsdir.clone();
        map_path.push( map_name );
        map_path.set_extension("bsp");

        if !map_path.is_file()
        {
            return Err(Error::new(ErrorKind::InvalidInput, "Failed to locate any readable .bsp files in maps directory!" ));
        }

        // Check to see if there's a music directory
        let mut musicdir = args.rootdir.clone();
        musicdir.push("sound");
        musicdir.push("music");

        if !musicdir.is_dir()
        {
            println!( "[Warning] Root directory {} has no music directory!  A default music file will be provided.", args.rootdir.display() );
        }
    }
    else // Is fullcheck mode.
    {
        if args.compress
        {
            println!( "[Warning] Cannot compress directory in fullcheck mode but compress flag is set!\nThe compression flag will be ignored." );
        }
    }

    // Check to make sure the GE:S directory exists and we have read/write access to it.
    // Not having a valid GE:S directory only costs a few minor features so we'll still allow
    // program execution in spite of it, unless we're in fullcheck mode in which case the gesdir
    // is the entire point of running the program.
    if !args.gesdir.is_dir()
    {
        if args.gesdir.is_file()
        {
            if args.fullcheck
            {
                return Err(Error::new(ErrorKind::InvalidInput, "Supplied GE:S directory is a file, not a directory!  This is needed for fullcheck mode." ));
            }
            else
            {
                println!( "[Warning] Supplied GE:S directory is a file, not a directory!" );
            }
        }
        else
        {
            if args.fullcheck
            {
                return Err(Error::new(ErrorKind::InvalidInput, "Supplied or Autodetected GE:S directory isn't a valid directory with write access!  This is needed for fullcheck mode." ));
            }
            else
            {
                println!( "[Warning] Supplied or Autodetected GE:S directory isn't a valid directory with write access!" );
            }
        }

        // Can only get here if we're not in fullcheck mode, so complete the warning messages.
        println!( "Without a GoldenEye: Source installation to reference, some program features will be limited." );
    }
    else
    {
        // If the supplied directory is valid...we'll want to make sure it's not any old
        // directory and actually is a GE:S directory, or at least be reasonably sure.
        // If this is not the case, our program may give frustrating, misleading errors as it
        // thinks it has valid GE:S data when it doesn't.  Because of this we'll want to error
        // out instead of just printing a warning.

        // It's very unlikely that a random folder will have goldeneye.fgd, so we can safley assume
        // that if this file exists where we expect we're in a valid GE:S directory.
        let mut ges_file = args.gesdir.clone();
        ges_file.push("goldeneye");
        ges_file.set_extension("fgd");

        if !ges_file.is_file()
        {
             return Err(Error::new(ErrorKind::InvalidInput, "GE:S directory is not the root directory of a valid GE:S installation!" ));
        }
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

#[cfg(test)]
mod tests 
{
    use shared::get_barebones_args;
    use shared::get_root_test_directory;
    use super::*;

    #[test]
    fn test_barebones_argument_set()
    {
        // Our barebones args are a valid set and as such should unwrap correctly.
        check_arguments( &get_barebones_args(), "test_map" ).unwrap();
    }

    #[test]
    fn test_fullcheck_argument_set()
    {
        // Our barebones args are a valid set even with fullcheck and as such should unwrap correctly.
        let mut args = get_barebones_args();
        args.fullcheck = true;

        check_arguments( &args, "test_map" ).unwrap();
    }

    #[test]
    fn test_invalid_fullcheck_argument_set()
    {
        // rootdir is not a valid gesdir so this check should fail.
        let mut args = get_barebones_args();
        args.fullcheck = true;
        args.gesdir = args.rootdir.clone();

        assert!(check_arguments( &args, "test_map" ).is_err());
    }

    #[test]
    fn test_non_ges_rootdir_argument_set()
    {
        // The rootdir must be a directory named "gesource", and so should fail on the root testing directory.
        let mut args = get_barebones_args();
        args.rootdir = get_root_test_directory();

        assert!(check_arguments( &args, "test_map" ).is_err());
    }

    #[test]
    fn test_fullcheck_with_no_rootdir_argument_set()
    {
        // We don't use rootdir in fullcheck mode so the program should still run without a valid one.
        let mut args = get_barebones_args();
        args.fullcheck = true;
        args.rootdir = get_root_test_directory();

        check_arguments( &args, "test_map" ).unwrap();
    }

    #[test]
    fn test_non_gesource_gesdir_argument_set()
    {
        // The gesdir must be a directory named "gesource", and so should fail on the root testing directory.
        let mut args = get_barebones_args();
        args.fullcheck = true;
        args.gesdir = get_root_test_directory();

        assert!(check_arguments( &args, "test_map" ).is_err());
    }

    #[test]
    fn test_invalid_gesdir_argument_set()
    {
        // The gesdir must have certain files in it, which rootdir doesn't have.
        let mut args = get_barebones_args();
        args.fullcheck = true;
        args.gesdir = args.rootdir.clone();

        assert!(check_arguments( &args, "test_map" ).is_err());
    }

    #[test]
    fn test_get_map_name()
    {
        /// See if we're correctly inferring the map name.
        assert_eq!( get_map_name(&get_barebones_args()), "test_map" );
    }
}