use prebindgen::prebindgen;

#[prebindgen]
#[derive(Debug, Clone)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
    pub email: Option<String>,
}

#[prebindgen]
#[derive(Debug, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending { reason: String },
    Suspended { until: String, reason: String },
}

#[prebindgen]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub ssl_enabled: bool,
}

// In a build script (build.rs), you could then read the generated file:
// include!(concat!(env!("OUT_DIR"), "/prebindgen.rs"));

fn main() {
    let person = Person {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        age: 30,
        email: Some("john.doe@example.com".to_string()),
    };
    
    let status = Status::Pending { 
        reason: "Awaiting verification".to_string() 
    };
    
    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
        ssl_enabled: true,
    };
    
    println!("Person: {:?}", person);
    println!("Status: {:?}", status);
    println!("Config: host={}, port={}, ssl={}", config.host, config.port, config.ssl_enabled);
    
    println!("\nThe definitions have been copied to prebindgen.rs in OUT_DIR");
    println!("You can access this file using: include!(concat!(env!(\"OUT_DIR\"), \"/prebindgen.rs\"));");
}
