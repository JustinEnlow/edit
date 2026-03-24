use std::fs;
//use std::error::Error;
use std::io::BufWriter;
use crate::application::Application;

/// Saves the document's content to its file path.
pub fn application_impl(app: &mut Application) -> Result<(), /*Box<dyn Error>*/String>{
    //if let Some(path) = &app.buffer.file_path{ // does nothing if path is None    //maybe return Err(()) instead?
    //    //app.buffer./*inner.*/write_to(BufWriter::new(fs::File::create(path)?))?;
    //    if app.buffer.is_modified(){
    //        app.buffer.write_to(BufWriter::new(fs::File::create(path)?))?;
    //    }else{
    //        //do nothing. we are already synched with file state.   //maybe return a same state error
    //    }
    //}
    //else{
    //    //return ApplicationError
    //}
    match &app.buffer.file_path{
        None => return Err(String::from("cannot save unnamed buffer")),
        Some(path) => {
            if app.buffer.read_only{return Err(String::from(crate::config::READ_ONLY_BUFFER));}
            //
            else if path.is_dir(){return Err(String::from("cannot save buffer text to directory"))}
            //
            else{
                if app.buffer.is_modified(){
                    match fs::File::create(path){
                        Err(e) => return Err(format!("{e}")),
                        Ok(file) => {
                            if let Err(e) = app.buffer.write_to(BufWriter::new(file)){
                                return Err(format!("{e}"));
                            }
                        }
                    }
                }else{
                    return Err(String::from(crate::config::SAME_STATE));
                }
            }
        }
    }
    
    Ok(())
}

//not sure how to test this here. has been tested by using fn from frontend code...
