
use std::fs;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};
use std::io;

use std::ffi::OsString;

use bzip2::Compression;
use bzip2::read::BzEncoder;

use std::fs::OpenOptions;
use std::thread;

use reslist_builder;
use shared;

pub fn construct_compressed_filesystem( args: &Arguments, map_name: &str ) -> Result<(), Error>
{
    // Our fastdownload server should have everything the reslist has, plus the map itself.
    // Split these into two threads because the map is a lot bigger than the other files usually.

    // First figure out where our compressed files will be going.
    let compressed_dir = get_compressed_directory( &args.rootdir )?;

    // If our compressed directory already exists, and we've opted-in to a complete recompress,
    // just delete every .bz2 file in the directory.
    if args.recompress && compressed_dir.is_dir()
    {
        println!( "Removing all .bz2 files in directory {}!", compressed_dir.display() );
        shared::remove_files_in_directory( &compressed_dir, "bz2" )?;
        println!( "Finished removal.");

        if shared::count_files_in_directory( &compressed_dir )? != 0
        {
            return Err(Error::new( ErrorKind::InvalidData, "gesource_compressed directory contains uncompressed or unremovable files!  Try deleting it and re-running the program." ));
        }
    }

    if args.verbose
    {
        println!("Starting file compression!");
    }

    // The map will easily be over half the filesize so let it take its own thread.
    let map_name_copy = String::from(map_name);
    let args_copy = args.clone();
    let compressed_dir_copy = compressed_dir.clone();

    let map_compress_handle = thread::spawn( move || 
    {
        let mut map_path = PathBuf::from("maps");
        map_path.push(map_name_copy);
        map_path.set_extension("bsp");

        compress_file( &args_copy, &args_copy.rootdir, &compressed_dir_copy, &map_path )
    });

    // Make use of our cached result from the previous directory mapping.
    let relevant_file_list = reslist_builder::generate_directory_tree( args )?;

    for file_path in relevant_file_list
    {
        let os_path = OsString::from(file_path);
        let relative_path = PathBuf::from(&os_path);

        compress_file( args, &args.rootdir, &compressed_dir, &relative_path )?;
    }

    // Unwrap the first result so that if the child thread hit a panic it will carry up through to us.
    // The second result carries an error that can be handled though so make sure that gets sent to 
    // the calling function.
    map_compress_handle.join().unwrap()?;

    println!("gesource_compressed directory is ready for upload.");

    Ok(())
}

fn compress_file( args: &Arguments, root_path: &PathBuf, c_root_path: &PathBuf, relative_path: &PathBuf ) -> Result<(), Error>
{
    // First get the path of the original file.
    let mut uncompressed_pathbuf = root_path.clone();
    uncompressed_pathbuf.push(relative_path);

    let mut compressed_pathbuf = c_root_path.clone();
    compressed_pathbuf.push( relative_path );
    compressed_pathbuf.set_extension( create_compressed_extension(&uncompressed_pathbuf) );

    // If we don't want to remake the file, then it's good enough that it exists.
    if !args.recompress && compressed_pathbuf.is_file()
    {
        return Ok(());
    }

    // We only need to read our input file.
    let input_file = OpenOptions::new().read(true).open(uncompressed_pathbuf)?;

    // Make sure the parent exists...but mostly just make sure that compressed_parent_folder
    // falls out of scope after we create the parent directory.
    if compressed_pathbuf.parent() != None
    {
        let compressed_parent_folder = compressed_pathbuf.parent().unwrap();
        fs::create_dir_all(&compressed_parent_folder)?;
    }

    // For the output file we want to be sure we're always overwriting any pre-existing files.
    // If it currently exists, it could be an old file.  If it's not old, we'll just get the same result.
    // This avoids unintentional desyncs between compressed and uncompressed files.  It might be worth
    // having an option to avoid overwriting files for savy server owners, however.
    let mut output_file = OpenOptions::new().write(true).truncate(true).create(true).open(compressed_pathbuf)?; 
    let mut compressor = BzEncoder::new(input_file, Compression::Best);

    io::copy(&mut compressor, &mut output_file)?;

    if args.verbose
    {
        println!( "Compressed {}", relative_path.display() );
    }

    Ok(())
}

fn create_compressed_extension( uncompressed_pathbuf: &PathBuf ) -> OsString
{
    // Source expects a sort of double-extension of xxx.bz2
    let mut compressed_extension;
    if uncompressed_pathbuf.extension() == None // No extension so we'll just be .bz2
    {
        compressed_extension = OsString::from("");
    }
    else // xxx.bz2
    {
        compressed_extension = OsString::from( uncompressed_pathbuf.extension().unwrap() );
        compressed_extension.push("."); // PathBuf can't add this for us this time.
    }
     
    compressed_extension.push("bz2");

    compressed_extension
}

fn get_compressed_directory( root_path: &PathBuf ) -> Result<PathBuf, Error>
{
    // Now determine where we want the compressed version to go.
    if root_path.parent() == None
    {
        return Err(Error::new( ErrorKind::InvalidData, "The root gesource directory must have valid parent for the compression routine to place files into." ));
    }

    let mut compressed_root_pathbuf = root_path.parent().unwrap().to_path_buf();
    compressed_root_pathbuf.push("gesource_compressed");
    compressed_root_pathbuf.push("gesource");

    Ok(compressed_root_pathbuf)
}