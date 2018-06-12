
use std::fs;
use std::io::prelude::*;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::io::BufReader;

use shared;

/// Generates the map script file used for random selection behavior.  
/// Returns Ok() if successful and an error if not.
pub fn create_or_verify_map_script_file( args: &Arguments, map_name: &str ) -> Result<(), Error>
{
    let mut map_script_dir = args.rootdir.clone();
    map_script_dir.push("scripts");
    map_script_dir.push("maps");

    if !map_script_dir.is_dir()
    {
        fs::create_dir_all(&map_script_dir)?;
    }

    // Just build the map path off of the existing script dir builder.
    let mut map_script_path = map_script_dir;

    map_script_path.push(map_name);
    map_script_path.set_extension("txt");

    if !map_script_path.is_file()
    {
        create_map_script_file( args, &map_script_path )?;
        println!("Created map script for {}!", map_name);
    }
    else
    {
        check_map_script_file( args, &map_script_path )?;
        println!("Existing map script file for {} is valid!", map_name);
    }

    Ok(())
}

/// Checks every map script in the provided or autodetected GE:S directory.
pub fn fullcheck_map_script_files( args: &Arguments ) -> Result<(), Error>
{
    let mut map_script_dir = args.gesdir.clone();
    map_script_dir.push("scripts");
    map_script_dir.push("maps");

    if !map_script_dir.is_dir()
    {
        return Err(Error::new( ErrorKind::InvalidData, "Map script directory does not exist!  Is this really a valid GE:S install?" ));
    }

    shared::check_all_files_in_dir_with_func( args, &map_script_dir, "txt", "map scripts", check_map_script_file )?;

    Ok(())
}

/// Creates a map script file with the given path and arguments in the standard GE:S map script format.
fn create_map_script_file( args: &Arguments, map_script_path: &PathBuf ) -> Result<(), Error>
{
    let mut map_script_file = fs::File::create(map_script_path)?;

    // Stick our program parameters into the core map details.
    // Individual weaponset and gamemode overrides didn't make a ton of sense to include as program
    // inputs, since it would probably be easier to just enter those manually.
    let mut contents = String::new();
    contents.push_str("// Map Script File Generated by GE:S Map Release Assistant for 5.0 - Report Any Issues to Entropy-Soldier\r\n");
    contents.push_str("\r\n");
    contents.push_str("// The game will try not to pick this map when the playercount is outside the range specified here.\r\n");
    contents.push_str("// The BaseWeight of the map controls how likely the map is to be chosen in random selection.\r\n");
    contents.push_str("// The map will not be chosen if the server playercount is below MinPlayers or above MaxPlayers\r\n");
    contents.push_str("// The baseweight scales with how far the playercount is from the average of MinPlayers and MaxPlayers.\r\n");
    contents.push_str("// because of this, maps with large ranges are not very likely to be picked at the edges of them.\r\n");
    contents.push_str("// ResIntensity is a measure of how much data in unique assets a map has.\r\n");
    contents.push_str("// It will avoid switching between maps with a combined intensity score of 10 or greater to avoid client crashes.\r\n");
    contents.push_str("\r\n");
    contents.push_str("BaseWeight\t"); contents.push_str(&args.baseweight.to_string()); contents.push_str("\r\n");
    contents.push_str("MaxPlayers\t"); contents.push_str(&args.maxplayers.to_string()); contents.push_str("\r\n");
    contents.push_str("MinPlayers\t"); contents.push_str(&args.minplayers.to_string()); contents.push_str("\r\n");
    contents.push_str("ResIntensity\t"); contents.push_str(&args.resintensity.to_string()); contents.push_str("\r\n");
    contents.push_str("TeamThreshold\t"); contents.push_str(&args.teamthresh.to_string()); contents.push_str("\r\n");
    contents.push_str("\r\n");
    contents.push_str("// Overrides the default weaponset weights if any sets are specified here.  Can be used as a blacklist.\r\n");
    contents.push_str("// Will only override weaponsets that are already in rotation, to prevent overriding gamemode specific lists.\r\n");
    contents.push_str("WeaponsetWeights\r\n");
    contents.push_str("{\r\n");
    contents.push_str("\tslappers\t\t0\r\n"); // slappers example
    contents.push_str("}\r\n");
    contents.push_str("\r\n");
    contents.push_str("// Weights for each gamemode if the map is switched to below the team threshold.\r\n");
    contents.push_str("// Overrides whatever weight is specified in default.txt, if there is one.\r\n");
    contents.push_str("// If a gamemode is not listed here or in default.txt it won't be used.\r\n");
    contents.push_str("GamemodeWeights\r\n");
    contents.push_str("{\r\n");
    contents.push_str("\tYOLT\t\t0\r\n"); // YOLT example.
    contents.push_str("}\r\n");
    contents.push_str("\r\n");
    contents.push_str("// Gamemode weights used when the map is switched to while playercount is above the team threshold.\r\n");
    contents.push_str("TeamGamemodeWeights\r\n");
    contents.push_str("{\r\n");
    contents.push_str("\tCaptureTheFlag\t\t0\r\n"); // CTF example.
    contents.push_str("}\r\n");
    contents.push_str("\r\n");

    // Write out our new file!
    map_script_file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Checks the map script file for format and parameter validity.
/// Take arguments here even though we don't use them so our function signature matches the other check functions.
fn check_map_script_file( _args: &Arguments, map_script_path: &PathBuf ) -> Result<(), Error>
{
    let map_script_file = fs::File::open(map_script_path)?;
    let reader = BufReader::new(map_script_file);

    // All of the terms we're hoping to find.
    // value terms are on their own line, in the format [term] [value]
    // bracket terms consist of multiple lines, with a [term] followed by a set of bracketed value terms.
    let mut needed_value_terms = vec!["BaseWeight", "MaxPlayers", "MinPlayers", "ResIntensity", "TeamThreshold"];
    let mut needed_bracket_terms = vec!["WeaponsetWeights", "GamemodeWeights", "TeamGamemodeWeights"];

    let mut checking_term = String::from("");

    // Need to mimic the original GE:S map script parser here since that's what will read our files\
    // ...even if it's not how I would have made it today.
    // It has a rather inflexible format with how comments and the bracketing work but is otherwise straightforward.
    // I'll probably remake the format for 5.1 in such a way that it's backwards compatable with this one and much more intuitive.

    // Surprisingly I've never gotten a complaint about this format, even though it utterly defies the standards it implies it uses.
    for line in reader.lines() 
    {
        let line = line?;
        
        // Comments only count if the first two characters are double slashes
        if line.starts_with("//")
        {
            continue;
        }

        let mut line_iter = line.split_whitespace();

        if checking_term.is_empty()
        {
            let line_identifier = line_iter.next();

            if line_identifier == None
            {
                continue;
            }

            let line_identifier = line_identifier.unwrap();

            if needed_value_terms.contains(&line_identifier)
            {
                check_line_value_validity(line_identifier, line_iter.next())?;
                needed_value_terms.retain(|x| x != &line_identifier);
            }
            else if needed_bracket_terms.contains(&line_identifier)
            {
                checking_term = String::from(line_identifier);
            }
        }
        else
        {
            // GE:S just assumes opening bracket means the rest of the line is blank...
            if line.starts_with( "{" )
            {
                continue;
            }

            // Same with closing bracket, except it also means we've hit the end of this section.
            if line.starts_with( "}" )
            {
                needed_bracket_terms.retain(|x| x != &checking_term);
                checking_term = String::from("");
                continue;
            }

            let line_identifier = line_iter.next();

            if line_identifier == None
            {
                let mut error_text = String::new();
                error_text.push_str("[Map Script Validate Error] Subvalue section for ");
                error_text.push_str( &checking_term );
                error_text.push_str(" contains an blank line when it must not contain any!");

                return Err(Error::new(ErrorKind::InvalidData, error_text ));
            }

            let line_identifier = line_identifier.unwrap();
            check_line_value_validity(line_identifier, line_iter.next())?;

            // If we had a closing bracket anywhere on that line GE:S assumes that means it was right at the end.
            if line.contains( "}" )
            {
                needed_bracket_terms.retain(|x| x != &checking_term);
                checking_term = String::from("");
                continue;
            }
        }
    }

    // Now let's make sure we're ending in the correct state.
    // We should have no script terms unused, and we shouldn't currently be parsing a text block.

    if !checking_term.is_empty()
    {
        let mut error_text = String::new();
        error_text.push_str("[Map Script Validate Error] Script ends in the middle of the ");
        error_text.push_str( &checking_term );
        error_text.push_str("Section!");

        return Err(Error::new(ErrorKind::InvalidData, error_text ));
    }

    if !needed_value_terms.is_empty()
    {
        let mut error_text = String::new();
        error_text.push_str("[Map Script Validate Error] Absent value terms: ");
        for term in needed_value_terms
        {
            error_text.push_str( term );
            error_text.push_str( " " );
        }

        return Err(Error::new(ErrorKind::InvalidData, error_text ));
    }

    if !needed_bracket_terms.is_empty()
    {
        let mut error_text = String::new();
        error_text.push_str("[Map Script Validate Error] Absent bracket terms: ");
        for term in needed_bracket_terms
        {
            error_text.push_str( term );
            error_text.push_str( " " );
        }

        return Err(Error::new(ErrorKind::InvalidData, error_text ));
    }

    Ok(())
}

// Makes sure the given line value for the provided line identifier exists and is valid.
fn check_line_value_validity( line_identifier: &str, line_value: Option<&str> ) -> Result<(), Error>
{
    if line_value == None
    {
        let mut error_text = String::new();
        error_text.push_str("[Map Script Validate Error] Expected value for parameter ");
        error_text.push_str( line_identifier );

        return Err(Error::new(ErrorKind::InvalidData, error_text ));
    }

    // We just made sure it's not None.
    let line_value = line_value.unwrap();

    match line_value.parse::<i32>()
    {
        Ok(_) => {}, // If we can cast correctly so can GE:S.
        Err(_) => 
        {
            let mut error_text = String::new();
            error_text.push_str("[Map Script Validate Error] Parameter for ");
            error_text.push_str( line_identifier );
            error_text.push_str(" not a valid whole number value!");

            return Err(Error::new(ErrorKind::InvalidData, error_text ));
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests 
{
    use shared::get_barebones_args;
    use shared::get_root_test_directory;
    use shared::do_validity_test;
    use super::*;

    #[test]
    fn test_valid_map_scripts() 
    {
        let mut valid_map_script_dir = get_root_test_directory();
        valid_map_script_dir.push("map_script_tests");
        valid_map_script_dir.push("valid");

        let args = get_barebones_args();

        do_validity_test(&args, &valid_map_script_dir, "Map Script", check_map_script_file, true);
    }

    #[test]
    fn test_invalid_map_scripts() 
    {
        let mut invalid_map_script_dir = get_root_test_directory();
        invalid_map_script_dir.push("map_script_tests");
        invalid_map_script_dir.push("invalid");

        let args = get_barebones_args();

        do_validity_test(&args, &invalid_map_script_dir, "Map Script", check_map_script_file, false);
    }
}