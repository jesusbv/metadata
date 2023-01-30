use clap::{Arg, App, ArgMatches};
use hyper::Client;
use hyper::status::StatusCode;
use std::io::Read;
use yaml_rust::yaml::{Yaml, YamlLoader};
use yaml_rust::YamlEmitter;
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

static YAML_FILE: &str = "/tmp/foo.yaml";
static METADATA_URL: &str = "http://169.254.169.254/";

fn _make_query(url: &str) -> String {
    let client = Client::new();
    let mut response = match client.get(url).send() {
        Ok(response) => response,
        Err(e) => {
            println!("Error! {}", e);
            return "".to_string(); // TODO: RETURN PROPER ERROR
        }
    };

    if response.status != StatusCode::Ok {
        return response.status.to_string();
    }
    let mut buf = String::new();
    match response.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(e) => {
            println!("Error! {}", e);
            return "".to_string(); // TODO: RETURN PROPER ERROR
        }
    };
    return buf;
}

fn get_metadata(api_version: &str, entry: &str) -> String {
    let url = format!(
        "{}{}/meta-data/{}",
        METADATA_URL,
        api_version,
        entry
    );
    println!("QUERYRING {}", url);
    return _make_query(&url);
}

fn display_data(entry: &str, data: &str, xml: bool) {
    let response: String;
    if xml {
        response = format!(
            "<{}>{}</{}>",
            entry,
            data,
            entry
        );
    } else {
        response = data.to_string();
    }
    println!("{} ", response);
}
fn get_api_versions() -> String {
    return _make_query(METADATA_URL);
}

fn fetch_options(url: &str, map: &mut HashMap<String, String>) {
    let foo = _make_query(&url);
    if foo.contains("Not Found") {
        return ();
    }
    let mut options= foo.split("\n");
    for option in options {
        if option.ends_with("/") {
            if url.ends_with("/") {
                fetch_options(
                    &(url.to_owned() + &option),
                    map
                );
            } else {
                // TODO: change to
                // &(Path::new(url.to_owned()).join(&option),
                fetch_options(
                    &(url.to_owned() + "/" + &option),
                    map
                );
            }
        } else {
            if url.ends_with("/") {
                map.insert(option.to_string(), url.to_owned() + &option);
            } else {
                map.insert(option.to_string(), url.to_owned() + "/" + &option);
            }
        }
    }
}

fn swrite_args_yaml(options: HashMap<String, String>) {
    let spaces = "    ";
    let cli_args =
        "name: metadata
version: \"1.0\"
author: Public Cloud Team <some_email_pct@suse.com>
about: Get instance metadata - this is a test
args:
    - xml:
        long: xml
        required: false
        takes_value: false
        help: Show output as XML
    - multiple-options:
        long: multiple-options
        required: false
        takes_value: false
        help: Combine multiple options in the command. Default false.
";
    let mut cli_argsa: String = "".to_string();
    // for (option, option_url) in map.iter() {
    // for option in options {
    //     if option.is_empty() || option.contains("0=jesus_ec2") {
    //         continue;
    //     }

    //     let arg = format!(
    //         "{}- {}:\n{}long: {}\n{}takes_value: false\n{}required: false\n{}help: Get {} from metadata\n",
    //         spaces, option, spaces.repeat(2), option, spaces.repeat(2),
    //         spaces.repeat(2), spaces.repeat(2), option);
    //     cli_argsa = format!("{}{}", cli_argsa, arg);
    // }
    cli_argsa = format!("{}{}", cli_args, cli_argsa);
    fs::write(YAML_FILE, cli_argsa);
}

fn get_args(version: &str) -> HashMap<String, String> {
    let vect = vec!["dynamic", "meta-data"];
    let mut map: HashMap<String, String> = HashMap::new();
    for endpoint in vect.iter() {
        let url = METADATA_URL.to_owned() + version + "/" + endpoint;
        let foo = _make_query(&url);
        if foo.contains("Not Found") {
            continue;
        }
        fetch_options(&url, &mut map);
    }
    let spaces = "    ";

    let cli_args =
        "name: metadata
version: \"1.0\"
author: Public Cloud Team <some_email_pct@suse.com>
about: Get instance metadata - this is a test
args:
    - xml:
        long: xml
        required: false
        takes_value: false
        help: Show output as XML
    - multiple-options:
        long: multiple-options
        required: false
        takes_value: false
        help: Combine multiple options in the command. Default false.
";
    let mut cli_argsa: String = "".to_string();
    for (option, option_url) in map.iter() {
        if option.is_empty() || option.contains("0=jesus_ec2") {
            continue;
        }

        let arg = format!(
            "{}- {}:\n{}long: {}\n{}takes_value: false\n{}required: false\n{}help: Get {} from metadata\n",
            spaces, option, spaces.repeat(2), option, spaces.repeat(2),
            spaces.repeat(2), spaces.repeat(2), option);
        cli_argsa = format!("{}{}", cli_argsa, arg);
    }
    cli_argsa = format!("{}{}", cli_args, cli_argsa);
    fs::write(YAML_FILE, cli_argsa);
    return map;
}

fn main() {
    // get all arguments passed to app
    let args: Vec<_> = std::env::args().collect();
    // Define command line arguments.
    let foo = get_api_versions();
    let versions: Vec<&str> = foo.lines().collect();

    let mut map: HashMap<String, String> = HashMap::new();
    // println!("{:?}", versions);
    // check api version
    // in order to know wich YAML generate
    if args.contains(&"--api".to_string()) {
        let index = args.iter().position(|r| r == "--api").unwrap();
        if (args.len() - 1) <= index {
            // get standard args in YAML
            map = get_args("2008-02-01");
        } else {
            map = get_args(&args[index + 1]); // get the args for that API version
            // USE VERSION YAML
            // let yaml = clap::load_yaml!("/tmp/foo.yaml");
            // let yaml = clap::load_yaml!("cli2.yaml");
            // let matches = App::from_yaml(&yaml).get_matches();
        }
    } else {
        map = get_args("2008-02-01");
    }
    let yaml = clap::load_yaml!("/tmp/foo.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    // TODO: USE STANDARD CLI YAML
    // yaml = clap::load_yaml!("/tmp/foo.yaml");
    // let matches = App::from_yaml(yaml).get_matches();
    // Get value for API, or default to 'latest'
    let api_version = matches.value_of("api").unwrap_or("latest");
    let xml = matches.is_present("xml");
    let pkcs7 = matches.is_present("pkcs7");
    let signature = matches.is_present("signature");
    // show results if any
    if !map.is_empty() {
        for element in args {
            let arg = &element[2..];
            if map.contains_key(arg) {
                let foo = map.get_key_value(arg);
                let result = _make_query(&foo.unwrap().1);
                display_data(arg, result.as_str(), xml);
                // one result at the time
                if !matches.is_present("multiple-options") {
                    return ();
                }
            }
            }
    }
    println!("Hello, world!");
}
