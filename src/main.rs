use mongodb::{Client, options::{ClientOptions, ResolverConfig}, Database};
use std::error::Error;
use tokio;
use log::{Log, Level, info, debug, LevelFilter, warn};
use std::sync::Mutex;
use futures::executor;
use dotenv::dotenv;

use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug)]
struct LogMessage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<bson::oid::ObjectId>,
    time : bson::DateTime,
    level : String,
    target : String,
    message : String
}

impl LogMessage{

    fn new(level: Level, target: String, message: String) -> Self {

        Self {
              id: None,
              time: bson::DateTime::from_chrono(chrono::Utc::now()), 
              level: level.to_string(),
              target,
              message
        }
    }
}

struct MongoLogger {

    database :Database,
    collection_name : String,
    logs : Mutex<Vec<LogMessage>>,
}

impl MongoLogger 
{
    fn new(db: Database, collection_name: String) -> Self{

        Self {database : db, collection_name: collection_name, logs : Mutex::new(vec![])}
    }
}




impl Log for MongoLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // metadata.level() == Level::Info
        true
    }
    fn log(&self, record: &log::Record) {
        
        if self.enabled(record.metadata()){
            println!("[{}] {} - {} - {}",chrono::Utc::now(), record.target(), record.level(), record.args());
            let document = LogMessage::new(record.level(), record.target().into(), record.args().to_string());
            // self.logs.lock().unwrap().push(document);
            executor::block_on(self.database.collection(&self.collection_name)
                .insert_one(document, None));
        }
    }
    fn flush(&self) 
    {

        // let log_collection = self.database.collection(self.collection_name.as_ref());
        // let logs = *(self.logs.lock().unwrap()).drain(..);
        // log_collection.insert_many(docs, options)
    }
}

fn test(){

    info!("Info Log ");
    debug!("Debug Log");
    warn!("Warning {} ", 2);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    // env_logger::init();
    

    let mongo_uri = std::env::var("MONGO_URL")?;
    let options = ClientOptions::parse_with_resolver_config(mongo_uri,ResolverConfig::cloudflare()).await?;
    let client = Client::with_options(options)?;
    for name in client.list_database_names(None, None).await? {
                println!("- {}", name);
    }
    let db = client.database("logger");
    
    log::set_boxed_logger(Box::new(MongoLogger::new(db, "logs".into())))
        .map(|()| log::set_max_level(LevelFilter::Info))?;
    test();
    Ok(())
}