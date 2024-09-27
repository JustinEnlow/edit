use crate::document::Document;
use std::{collections::HashMap, error::Error, path::PathBuf};



/// Type alias for usize
/// ```
/// # use nlo_text_editor_server::editor::ClientIDManager;
/// 
/// let mut id_manager = ClientIDManager::default();
/// let _ = id_manager.assign_id();
/// id_manager.release_id(0);
/// assert!(id_manager.assign_id() == 0);
/// ```
type ClientID = usize;  // only an alias. doesn't enforce passed value is a ClientID and not just a usize.

/// Generates an ID for each client(individual front end instance or separate tabs 
/// inside an instance) connected to the editor.
/// # Example
/// ```
/// # use nlo_text_editor_server::editor::ClientIDManager;
/// 
/// let mut manager = ClientIDManager::default();
/// let id0 = manager.assign_id();
/// let id1 = manager.assign_id();
/// manager.release_id(id0);
/// let id3 = manager.assign_id();
/// assert!(id1 == 1);
/// assert!(id3 == 0);
/// ```
#[derive(Default)]
pub struct ClientIDManager{
    next_id: ClientID,
    available_ids: Vec<ClientID>,  //should this be newtyped as ClientID(usize)? or type aliased?
}
impl ClientIDManager{
    pub fn assign_id(&mut self) -> ClientID{
        if let Some(id) = self.available_ids.first().cloned(){
            self.available_ids.remove(0);
            id
        }else{
            let id = self.next_id;
            self.next_id = self.next_id + 1;
            id
        }
    }

    pub fn release_id(&mut self, id: ClientID){
        self.available_ids.push(id);
        self.available_ids.sort();
    }
}

/// An Editor holds documents. Document interactions are performed through the editor
#[derive(Default)]
pub struct Editor{
    documents: HashMap<String, Document>,   //HashMap<ClientID, Document>
    client_id_manager: ClientIDManager,
}
impl Editor{
    /// Returns a reference to the document associated with ClientID, if one exists.
    pub fn document(&self, client_address: &str) -> Option<&Document>{
        if let Some(doc) = self.documents.get(client_address){
            return Some(doc);
        }

        None
    }
    /// Returns a mutable reference to the document associated with ClientID, if one exists.
    pub fn document_mut(&mut self, client_address: &str) -> Option<&mut Document>{
        if let Some(doc) = self.documents.get_mut(client_address){
            return Some(doc);
        }

        None
    }
    /// Attempts to open specified document, and associate it with ClientID.
    pub fn open_document(&mut self, path: &PathBuf, client_address: &str) -> Result<(), Box<dyn Error>>{
        let doc = Document::open(path)?;
        self.documents.insert(client_address.to_string(), doc);

        Ok(())
    }
    /// Removes the document associated with ClientID, if one exists.
    // TODO: if doc is associated with multiple client ids, remove association with this ClientID
    pub fn close_document(&mut self, client_address: &str){
        if self.documents.contains_key(client_address){
            self.documents.remove(client_address);
        }
    }

    pub fn id_manager_mut(&mut self) -> &mut ClientIDManager{
        &mut self.client_id_manager
    }
}
