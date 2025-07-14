use std::fs;
use std::error::Error;
use std::io::BufWriter;
use crate::application::Application;

/// Saves the document's content to its file path.
pub fn application_impl(app: &mut Application) -> Result<(), Box<dyn Error>>{
    if let Some(path) = &app.buffer.file_path{ // does nothing if path is None    //maybe return Err(()) instead?
        app.buffer./*inner.*/write_to(BufWriter::new(fs::File::create(path)?))?;
    }
    else{
        //return ApplicationError
    }
    
    Ok(())
}

//not sure how to test this here. has been tested by using fn from frontend code...
