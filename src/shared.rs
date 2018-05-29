use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind};
use std::error::Error as ErrorTrait; // Use an alias as it will conflict with the error object otherwise.

use walkdir::WalkDir;

use argument_handler::Arguments;

/// Gets the file paths of all files in a given directory, relative to the root path supplied.
pub fn get_files_in_directory( files_dir: &PathBuf, target_extension: &str, excluded_extensions: &[&str] ) -> Result<Vec<String>, Error>
{
    // This is where the relative paths of our desired files will go.
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