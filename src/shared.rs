// Copyright 2018 Entropy-Soldier
//
// Licensed under the MIT license: http://opensource.org/licenses/MIT
// This file may not be copied, modified, or distributed except according to those terms.

// ----------------------------------------------------------------------------
// shared: Contains utility functions and code shared between various modules.
// ----------------------------------------------------------------------------

use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind};
use std::error::Error as ErrorTrait; // Use an alias as it will conflict with the error object otherwise.

use std::sync::Mutex;
use std::ops::DerefMut;

use std::fs;

use walkdir::WalkDir;

use argument_handler::Arguments;

/// Gets the file paths of all files in a given directory, relative to the root path supplied.
pub fn get_files_in_directory( files_dir: &PathBuf, target_extension: &str, excluded_extensions: &[&str] ) -> Result<Vec<String>, Error>
{
    // This is where the relative paths of our desired files will go.
    // For larger sets a hashmap would be better for the constant lookup time, but the linear lookup time
    // in Vec is actually quite fast and most map release directory structures have less than 100 entries.
    // With smaller data sets, vec is usually going to be much faster, and no arbitrary insertion is ever needed.
    // Fullcheck mode will involve an upwards of 20,000 files, but this is a more esoteric use of the program
    // and even then the search time on the vectors is nearly imperceptable when scanning through ~50 music scripts.
    let mut file_names: Vec<String> = Vec::new(); 

    // Grab the directory path here for later.
    let dir_path = files_dir.to_str();

    if dir_path == None 
    {  
        return Err(Error::new( ErrorKind::InvalidInput, "Could not construct directory path string!"));
    }

    // We just made sure it's not None so we can unwrap it.
    let dir_path = dir_path.unwrap();

    // Make sure our  directory exists and if so scan it for files.
    if files_dir.is_dir()
    {
        for entry in WalkDir::new( files_dir ) 
        {
            let entry = entry?;
            let entrypath = entry.path();

            // Not a file we have access to, don't worry about it.
            if !entrypath.is_file() { continue; }

            // Grab the file extension for comparison.
            let file_extension = get_file_extension(entrypath);

            // If we only want a particular type of file, ignore all others.
            if !target_extension.is_empty() && file_extension.to_lowercase() != target_extension { continue; }

            // If we don't want a particular type of file, ignore it.
            if !excluded_extensions.is_empty() && excluded_extensions.contains( &file_extension.to_lowercase().as_str() ) { continue; }

            // Grab the full file path as a string so we can turn it into a relative path.
            let path_string = entrypath.to_str();
            if path_string == None { continue; }

            let path_string = path_string.unwrap();

            // The path string is a child of the sound_dir_path string, so it will always be longer.
            // With this info we cut out the parent path + the final slash to get our script path.
            let path_string = 
            {
                let mut path_string = &path_string[dir_path.len()..];

                if path_string.starts_with("\\") || path_string.starts_with("/")
                {
                    path_string = &path_string[1..]
                }

                path_string
            };

            // Source engine uses forward slashes in the file paths its script files, so make sure all
            // slashes are forward slashes.  Also go to lowercase for easy compairisons since windows is
            // not case sensitive.
            // This also gives us our final String object to push into the array.
            let final_path_string = path_string.replace("\\", "/").to_lowercase();

            file_names.push( final_path_string );
        }
    }

    Ok(file_names)
}

/// Get the extension of the given path as a &str.  
/// If it doesn't have one or the extension can't be converted, return "".
pub fn get_file_extension( filepath: &Path ) -> &str
{
    match filepath.extension()
    {
        Some(x) => 
        {
            match x.to_str()
            {
                Some(y) => y,
                None => "",
            }
        },
        None => "",
    }
}

/// Checks every file in the given directory with the given extension using the supplied function.
pub fn check_all_files_in_dir_with_func( args: &Arguments, dir: &PathBuf, extension: &str, print_type: &str, check_func: fn( args: &Arguments, music_script_path: &PathBuf ) -> Result<(), Error> ) -> Result<(), Error>
{
    if args.verbose
    {
        println!("Scanning {} in {}!\n", print_type, dir.display());
    }

    let mut scanned_file_count = 0;

    // Make sure our sound directory exists and if so scan it for files.
    for entry in WalkDir::new( &dir )
    {
        let entry = entry?;
        let entrypath = entry.path();

        // Not a file we have access to, don't worry about it.
        if !entrypath.is_file() { continue; }

        let file_extension = get_file_extension( entrypath );

        // Only check the specified file type.
        if file_extension.to_lowercase() != extension { continue; }

        // Run the check func, appending the file that caused the error to the error message if it failed.
        match check_func( args, &PathBuf::from(entrypath) )
        {
            Ok(_) => (),
            Err(e) => 
            {
                let mut error_text = String::new();
                error_text.push_str("While proccessing ");
                error_text.push_str( entrypath.to_str().unwrap_or("an unidentifiable file") );
                error_text.push_str(" the following error was encountered:\n");
                error_text.push_str(e.description());

                return Err(Error::new(ErrorKind::InvalidData, error_text ));
            }
        }
        scanned_file_count += 1; // We've successfully scanned a file, so add it to the final count.

        if args.verbose
        {
            println!("{} is formatted correctly!", entrypath.to_str().unwrap_or("an unidentifiable file"));
        }
    }

    // Let the user know of our success.
    println!("\nAll {} {} in {} are formatted correctly!", scanned_file_count, print_type, dir.display());

    Ok(())
}

/// Removes all files in the given directory tree with the given extension.
pub fn remove_files_in_directory( files_dir: &PathBuf, target_extension: &str ) -> Result<(), Error>
{
    // Make sure our  directory exists and if so scan it for files.
    if files_dir.is_dir()
    {
        for entry in WalkDir::new( files_dir ) 
        {
            let entry = entry?;
            let entrypath = entry.path();

            // Not a file we have access to, don't worry about it.
            if !entrypath.is_file() { continue; }

            // Grab the file extension for comparison.
            let file_extension = get_file_extension(entrypath);

            let file_extension = file_extension.split(".").last().unwrap_or("");

            // If we only want a particular type of file, ignore all others.
            if !target_extension.is_empty() && file_extension.to_lowercase() != target_extension { continue; }

            // Looks like everything checks out...delete the file.
            fs::remove_file(entrypath)?;
        }
    }

    Ok(())
}

/// Counts all files in the given directory tree.
pub fn count_files_in_directory( files_dir: &PathBuf ) -> Result<u32, Error>
{
    let mut file_count = 0;

    // Make sure our  directory exists and if so scan it for files.
    if files_dir.is_dir()
    {
        for entry in WalkDir::new( files_dir ) 
        {
            let entry = entry?;
            let entrypath = entry.path();

            // Not a file we have access to, don't worry about it.
            if !entrypath.is_file() { continue; }

            file_count += 1;
        }
    }

    Ok(file_count)
}

/// Walks each directory in cache_dirs and runs get_files_in_directory on them with the target_filetype and disallowed_filetype
/// parameters.  After completion, the results will be stored in the contents of directory_cache and mutex will be set to true and
/// a reference to the contents of directory_cache will be returned.
/// On subsequent calls with references to the same two variables, the computation is skipped and the contents of
/// directory cache are returned directly.  This saves us from having to walk a directory set multiple times when
/// the contents will not change between invocations.
pub fn compute_or_get_safe_reference_to_directory_cache( cache_dirs: Vec<&PathBuf>, target_filetype: &str, disallowed_filetypes: &[&str], mutex: &'static Mutex<bool>, directory_cache: &'static mut Option<Vec<String>> ) -> Result<&'static Vec<String>, Error>
{
    // First grab the mutex guard for the init variable.  If we're uninitalized, then we'll grab this and
    // do the computations, and set the value to true.  If we're in the proccess of initalizing, we'll wait
    // for the lock and after we aquire it we'll be initalized.  If we're initalized, we'll get the lock to
    // confirm that and then just return a reference to dirlist.  Once we hit the end of the unsafe block we
    // will drop the lock and let the next iteration take over.
    let mut init_guard = mutex.lock().unwrap();
    let has_init = init_guard.deref_mut();

    if !*has_init
    {
        *directory_cache = Some(Vec::new());
    }

    let dirlist_ref = match *directory_cache
    {
            Some(ref mut x) => &mut *x,
            None => return Err(Error::new(ErrorKind::Other, "Failed to create directory cache!")),
    };

    if !*has_init
    {
        for dir in cache_dirs
        {
            dirlist_ref.append(&mut get_files_in_directory( &dir, target_filetype, disallowed_filetypes )?);
        }
        *has_init = true;
    }

    return Ok(dirlist_ref);
}

#[cfg(test)]
/// Tests every file in the given directory using the given parameters.
pub fn do_validity_test( args: &Arguments, dir: &PathBuf, print_type: &str, check_func: fn( args: &Arguments, script_path: &PathBuf ) -> Result<(), Error>, should_pass: bool )
{
    for entry in WalkDir::new( dir )
    {
        let entry = entry.unwrap();
        let entrypath = entry.path();

        if !entrypath.is_file() { continue; }

        let check_result = check_func( &args, &entrypath.to_path_buf() );

        if should_pass == false
        {
            assert!( check_result.is_err(), "{} {} was detected as valid when it should be invalid!", print_type, entrypath.display() );
        }
        else
        {
            assert!( !check_result.is_err(), "{} {} was detected as invalid when it should be valid!  Error was {}", print_type, entrypath.display(), check_result.err().unwrap() );
        }
    }
}

#[cfg(test)]
/// Tests the result of a given script creator with the given check function, passing if the check is valid and failing if it is not.
pub fn test_script_creator( args: &Arguments, 
                        file_name: &str, 
                        create_func: fn( args: &Arguments, script_path: &PathBuf ) -> Result<(), Error>,
                        check_func: fn( args: &Arguments, script_path: &PathBuf ) -> Result<(), Error> ) 
{
    // Now that we've confirmed the script checker works...let's create a file and use it to check it!
    let mut temp_dir = get_root_test_directory();
    temp_dir.push("temp");

    let mut script_path = temp_dir.clone();
    script_path.push(file_name);

    // Remove any previous script files that may have existed here before.
    if script_path.is_file()
    {
        fs::remove_file(&script_path).unwrap();
    }

    create_func( &args, &script_path ).unwrap();
    check_func( &args, &script_path ).unwrap();

    // If we got here with no erors we passed the test!
}

#[cfg(test)]
/// Creates a set of barebones arguments for testing.
pub fn get_barebones_args() -> Arguments
{
    let test_rootdir = get_root_test_directory();

    let mut test_gesdir = test_rootdir.clone();
    test_gesdir.push("gesdir");
    test_gesdir.push("gesource");

    let mut test_rootdir = test_rootdir.clone();
    test_rootdir.push("rootdir");
    test_rootdir.push("gesource");
    

    Arguments
    {
        rootdir: test_rootdir,
        gesdir: test_gesdir,
        baseweight: 700,
        minplayers: 0,
        maxplayers: 16,
        resintensity: 7,
        teamthresh: 12,
        compress: false,
        recompress: false,
        verbose: false,
        fullcheck: false,
        noexitprompt: true,
    }
}

#[cfg(test)]
/// Locates the project's root directory
pub fn get_root_test_directory() -> PathBuf
{
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.push("resources");
    test_dir.push("tests");

    test_dir
}