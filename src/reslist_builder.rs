
use std::fs;
use std::io::prelude::*;
use argument_handler::Arguments;

use std::path::PathBuf;
use std::io::{Error, ErrorKind};

/// Generates the reslist used for map asset downloads
/// Returns Ok() if successful and an error if not.
pub fn create_or_verify_reslist( args: &Arguments, map_name: &str ) -> Result<(), Error>
{
    Ok(())
}