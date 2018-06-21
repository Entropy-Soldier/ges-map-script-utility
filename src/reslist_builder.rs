// Copyright 2018 Entropy-Soldier
//
// Licensed under the MIT license: http://opensource.org/licenses/MIT
// This file may not be copied, modified, or distributed except according to those terms.

// ---------------------------------------------------------------------------------------------------------
// restlist_builder: Contains functions for analyzing and building reslist files for GoldenEye: Source maps.
// ---------------------------------------------------------------------------------------------------------

use std::fs;
use std::io::prelude::*;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::io::BufReader;

use shared;

use regex::Regex;


// Grab all files in our installation except for the disallowed file types, to make sure everything is included.
// BSP files are not allowed as it wouldn't make sense to include the map itself in the reslist or any other maps with it.
// Res files are not allowed as the reslist itself doesn't need to be included for clients to download.
// Exe files are not allowed as executable files are useless for a map's purposes and most likely this category would just be
// including the ges_mapreleaser.exe file if it was used with no parameters and placed in the root directory.
static DISALLOWED_FILETYPES: &[&'static str] = &["bsp", "res", "exe"];


/// Generates or checks the reslist used for map asset downloads
/// Returns Ok() if successful and an error if not.
pub fn create_or_verify_reslist( args: &Arguments, map_name: &str ) -> Result<(), Error>
{
    // Reslists go in the maps directory, which must exist for the program to even start.
    let mut relist_path = args.rootdir.clone();
    relist_path.push("maps");
    relist_path.push( map_name );
    relist_path.set_extension("res");

    if !relist_path.is_file()
    {
        create_reslist( args, &relist_path )?;
        println!("Created reslist for {}!", map_name);
    }
    else
    {
        check_reslist( args, &relist_path )?;
        println!("Existing reslist for {} is valid!", map_name);
    }

    Ok(())
}

/// Checks every reslist in the provided or autodetected GE:S directory.
pub fn fullcheck_reslist_files( args: &Arguments ) -> Result<(), Error>
{
    let mut map_dir = args.gesdir.clone();
    map_dir.push("maps");

    if !map_dir.is_dir()
    {
        return Err(Error::new( ErrorKind::InvalidData, "Maps directory does not exist!  Is this really a valid GE:S install?" ));
    }

    shared::check_all_files_in_dir_with_func( args, &map_dir, "res", "reslists", check_reslist )?;

    Ok(())
}

/// Creates a reslist that includes every file in the local directory.
fn create_reslist( args: &Arguments, reslist_path: &PathBuf ) -> Result<(), Error>
{
    // Grab every file in the directory so we can make sure the server will download
    // them to clients when the time comes.
    // We don't want to include the map bsp itself however as it will get downloaded regardless.
    // We also don't want to include any reslists or exe files.
    let file_list = generate_directory_tree( args )?;

    // This should never happen in normal operation since the other script files should be created or validated
    // before this part of the program is run, and they must exist in the root directory else it would have errored out.
    // There is the possibility that in the future there will be demand for a reslist-only parameter however so it
    // doesn't hurt to program defensively in this case.  If there are no files to download there's no point in making the reslist!
    if file_list.is_empty()
    {
        println!("[Warning] Root directory seems to be empty!  There are no files to include in the reslist so it will be skipped.");
        return Ok(());
    }

    // The reslist has a rather simple format, just stick all included files into it in this format:
    // "[path/to/file]" "file"
    // It's the reverse of the music files...not entirely sure why as I didn't design either but it's not a problem.
    let mut contents = String::new();
    contents.push_str("\"resources\"\r\n");
    contents.push_str("{\r\n");

    for file in file_list
    {
        contents.push_str("\t\""); contents.push_str(&file); contents.push_str("\"\t\"file\"\r\n");
    }

    contents.push_str("}\r\n");

    // Make it official and write the final string to the file.
    let mut reslist_file = fs::File::create(reslist_path)?;
    reslist_file.write_all(contents.as_bytes())?;

    Ok(())
}

/// Makes sure every file in the local directory tree is included in the provided reslist, that the reslist is
/// formatted correctly, and that every file in the reslist exists in the local directory path.
fn check_reslist( args: &Arguments, reslist_path: &PathBuf ) -> Result<(), Error>
{
    let reslist_file = fs::File::open(reslist_path)?;
    let mut reader = BufReader::new(reslist_file);

    let mut contents = String::new();
    reader.read_to_string( &mut contents )?;

    // Reslist file format is simpler than the music list format and as such is a bit easier to handle.
    // It consists of a "resources" bracketed section with entries using the format:
    // "[path/to/file]" "file"  
    // No other complications or fancy setup to look for.
    // Using [Rr] instead of the (?i) flag since the (?i) flag seems to increase runtimes significantly,
    // and people probably don't need to call it "ReSoUrCeS" or something like that.
    lazy_static! // Using lazy static as reccomended by the Rust documentation for optimization purposes.
    {
        static ref FILE_RE: Regex = Regex::new(r#"(?x)^\s*(("[Rr]esources")|([Rr]esources))\s*
                                (\{
                                (\s*(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s+(("file")|(file))\s*)+
                                \})\s*$"#).unwrap();
    }
    
    if !FILE_RE.is_match(&contents)
    {
        return Err(Error::new( ErrorKind::InvalidData, "Script contains core format mistake!\n  Make sure every \
                                                        bracket and quotation mark has a partner, the main section \
                                                        is labeled \"resources\", each file path has a \"file\"\
                                                        section after it, no bracketed sections are empty,\
                                                        and that there are no nested bracketed sections inside\
                                                        the main bracketed section."));
    }

    // If we made it here it means we have a valid file with at least one file entry.  Check those file entries
    // to make sure they're formatted correctly and point to a valid file that we're including with the map.

    // We need to have all the files in the directory to make sure that they're being included in our script.
    // Otherwise the mapper could be sending out a file they don't need to, or forgot to put in the reslist.
    // Incurs a sizable performance hit on fullcheck mode, but with caching and a large number of reslists to
    // scan through it performs alright.
    let file_list = generate_directory_tree( args )?;

    let mut checked_file_list: Vec<String> = Vec::new(); 

    lazy_static!
    {
        static ref RE: Regex = Regex::new(r#"\s*(("[^"\{\}]*")|([\S&&[^"\{\}]]+))\s+(("file")|(file))\s*"#).unwrap();
    }

    for cap in RE.captures_iter(&contents)
    {
        // We've already verified we've got a capture, and slot 1 is mandatory for us to have one.
        let fixed_path = cap[1].replace("\"", "").replace("\\", "/").to_lowercase(); // Remove possible quotation marks and standardize slashes.

        // Make sure we're not using a disallowed extension.
        if DISALLOWED_FILETYPES.contains( &shared::get_string_file_extension( &fixed_path.as_str() ).to_lowercase().as_str() )
        { 
            let mut error_text = String::new();
            error_text.push_str("Resource file ");
            error_text.push_str(&fixed_path);
            error_text.push_str(" is of a filetype that should not be included in the reslist!  \
                                  Map files and the reslist itself do not need to be included in the reslist.");

            return Err(Error::new(ErrorKind::InvalidData, error_text ));
        }

        // Check to see if our MP3 file is one of the files we've detected in the relevant directories.
        // if not, our script is pointing to an invalid file and isn't ready for release!
        if !file_list.contains(&fixed_path)
        {
            let mut error_text = String::new();
            error_text.push_str("Failed to locate resource file ");
            error_text.push_str(&fixed_path);
            error_text.push_str("\nEnsure that the file path is valid, and that the file exists.");

            return Err(Error::new(ErrorKind::InvalidData, error_text ));
        }
        else // It's a valid file, but might be repeated.
        {
            // Make sure we don't have multiple lines referencing the same resource.
            if checked_file_list.contains(&fixed_path)
            {
                let mut error_text = String::new();
                error_text.push_str("Resource file ");
                error_text.push_str(&fixed_path);
                error_text.push_str(" is referenced multiple times!  Please remove the redundant references.");

                return Err(Error::new(ErrorKind::InvalidData, error_text ));
            }

            // Now that we've checked it, push the path to our checked array so we'll catch it if it comes up again.
            checked_file_list.push(fixed_path.clone());
        }
    }

    // If we're in fullcheck mode we're scanning an entire GE:S install so many of the files will not
    // be included in any particular reslist.  Opt out of that particular check for fullcheck mode.
    if args.fullcheck
    {
        return Ok(());
    }

    // We've just made sure that all of the files included in our reslist will be destributed with the map...
    // but we also want to make sure that all the files being destributed are included in our reslist!

    let mut missing_file_list: Vec<&str> = Vec::new(); 

    // file_list will live just as long as missing_file_list, so to save runtime let's just
    // take references to the entries in file list instead of copying the values.
    for file in file_list
    {
        // If we never checked it, it wasn't in the reslist.
        if !checked_file_list.contains(&file)
        {
            missing_file_list.push(file);
        }
    }

    // If we have missing files our script isn't ready for release!
    if !missing_file_list.is_empty()
    {
        let mut error_text = String::new();
        error_text.push_str("Resource files ");

        for missing_file in missing_file_list
        {
            error_text.push_str(&missing_file); error_text.push_str(" ");
        }

        error_text.push_str(" aren't included in the reslist!  Be sure to include entries for them or remove them from the destribution folder.");

        return Err(Error::new(ErrorKind::InvalidData, error_text ));
    }

    // The reslist is in the correct format, all of our files are included, and no others.
    // The reslist is ready for release!
    Ok(())
}


use std::sync::Mutex;

/// Provides a reference to a vector storing strings that correspond to the relative paths of every file in
/// the provided directory.  Subsequent calls return the cached value of the first call.
pub fn generate_directory_tree( args: &Arguments ) -> Result<&'static Vec<String>, Error>
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
        return shared::compute_or_get_safe_reference_to_directory_cache( vec![&args.rootdir], "", DISALLOWED_FILETYPES, &DIRLIST_INIT_STATE, &mut DIRLIST );
    }
}

#[cfg(test)]
mod tests 
{
    use shared::get_barebones_args;
    use shared::get_root_test_directory;
    use shared::do_validity_test;
    use shared::test_script_creator;
    use super::*;

    #[test]
    fn test_valid_reslists() 
    {
        let mut valid_reslist_dir = get_root_test_directory();
        valid_reslist_dir.push("reslist_tests");
        valid_reslist_dir.push("valid");

        let args = get_barebones_args();

        do_validity_test(&args, &valid_reslist_dir, "Reslist", check_reslist, true);
    }

    #[test]
    fn test_invalid_reslists() 
    {
        let mut invalid_reslist_dir = get_root_test_directory();
        invalid_reslist_dir.push("reslist_tests");
        invalid_reslist_dir.push("invalid");

        let args = get_barebones_args();

        do_validity_test(&args, &invalid_reslist_dir, "Reslist", check_reslist, false);
    }

    #[test]
    fn test_reslist_creator() 
    {
        // Now that we've confirmed the script checker works...let's create a file and use it to check it!
        test_script_creator( &get_barebones_args(), "test_map.res", create_reslist, check_reslist );
    }
}