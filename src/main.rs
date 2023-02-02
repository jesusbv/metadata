use clap::{App, Arg, Error};
use hyper::status::StatusCode;
use hyper::Client;
use std::collections::HashMap;
use std::io::Read;
// use std::path::{Path, PathBuf};

// static YAML_FILE: &str = "/tmp/foo.yaml";
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
        // println!("{:?}", response.status);
        // println!("{:?}", response.status.to_string());
        // println!("{:?}", url);
        return response.status.to_string();
    }
    let mut result = String::new();
    match response.read_to_string(&mut result) {
        Ok(_) => (),
        Err(e) => {
            println!("Error! {}", e);
            return "".to_string(); // TODO: RETURN PROPER ERROR
        }
    };
    return result;
}

fn display_data(entry: String, data: String, xml: bool, all_info: bool) {
    let mut response: String = format!("{}", data);
    if xml {
        response = format!("<{}>{}</{}>", entry, data, entry);
    }
    if all_info {
        response = format!("{}: {}", entry, data);
    }

    println!("{} ", response);
}

fn get_api_versions() -> String {
    return _make_query(METADATA_URL);
}

fn fetch_options(
    url: &str,
    map: &mut HashMap<String, String>,
    args: &mut Vec<String>,
    all_info: bool,
) {
    if args.is_empty() && !all_info {
        return ();
    }

    let options = _make_query(&url);

    for option in options.split("\n") {
        if option.ends_with("/") {
            let mut new_url = url.to_owned() + &option;
            if !url.ends_with("/") {
                new_url = url.to_owned() + "/" + &option;
            }
            fetch_options(&(new_url), map, args, all_info);
        } else {
            // get the value from the URL
            let mut new_url = url.to_owned() + &option;

            if url.ends_with("/") {
                let param = "--".to_owned() + option;
                if args.contains(&"--public-keys".to_string())
                    && option.contains("=")
                    && url.contains("public-keys")
                {
                    let vec: &Vec<&str> = &vec![option.split("=").collect()][0];
                    new_url = url.to_owned() + &vec[0] + "/openssh-key";
                    let value = _make_query(&new_url);
                    map.insert("public-keys".to_string(), value);
                    if !all_info {
                        let index = args.iter().position(|arg| *arg == "--public-keys").unwrap();
                        args.remove(index);
                    }
                } else {
                    if args.contains(&(param)) || all_info {
                        let value = _make_query(&new_url);
                        if !value.contains("Not Found") {
                            map.insert(option.to_string(), value);
                        }
                        if !all_info {
                            let index = args.iter().position(|arg| *arg == param).unwrap();
                            args.remove(index);
                        }
                    }
                }
            } else {
                new_url = url.to_owned() + "/" + &option;
                if all_info {
                    let value = _make_query(&new_url);
                    if !value.contains("Not Found") {
                        map.insert(option.to_string(), value);
                    }
                }
            }
        }
        if args.is_empty() && !all_info {
            return ();
        }
    }
}

fn display(map: HashMap<String, String>, xml: bool, all_info: bool) {
    // check if indexmap has values() and keys ()
    for (key, value) in map {
        display_data(key, value, xml, all_info);
    }
}

// fn write_args_yaml(options: &mut std::collections::hash_map::Keys<String, String>) {
//     let cli_args = "name: metadata
// version: \"1.0\"
// author: Public Cloud Team <some_email_pct@domain.com>
// about: Get instance metadata - this is a test
// args:
//   - api:
//       long: api
//       required: false
//       takes_value: true
//       help: Version of the API
//   - xml:
//       long: xml
//       required: false
//       takes_value: false
//       help: Show output as XML
//   - multiple-options:
//       long: multiple-options
//       required: false
//       takes_value: false
//       help: Combine multiple options in the command. Default false.
// ";
//     let mut cli_args_formatted: String = "".to_string();
//     let spaces = "  ";
//     for option in options {
//         // if option.is_empty() || option.starts_with("0") {
//         //     continue;
//         // }

//         let arg = format!(
//             "{}- {}:\n{}long: {}\n{}takes_value: false\n{}required: false\n{}help: Get {} from metadata\n",
//             spaces, option, spaces.repeat(3), option, spaces.repeat(3),
//             spaces.repeat(3), spaces.repeat(3), option);
//         cli_args_formatted = format!("{}{}", cli_args_formatted, arg);
//     }
//     cli_args_formatted = format!("{}{}", cli_args, cli_args_formatted);
//     // fs::write(YAML_FILE, cli_args_formatted);
//     std::io::stdout().flush().expect("Could not write ");
// }

fn get_args_from_framework(
    version: &str,
    args: &mut Vec<String>,
    all_info: bool,
) -> HashMap<String, String> {
    let vect = vec!["dynamic", "meta-data"];
    let mut map: HashMap<String, String> = HashMap::new();
    for endpoint in vect.iter() {
        let url = METADATA_URL.to_owned() + version + "/" + endpoint;
        fetch_options(&url, &mut map, args, all_info);
    }
    // write_args_yaml(&mut map.keys());
    return map;
}

fn main() {
    // get all arguments passed to app
    let mut args: Vec<_> = std::env::args().collect();
    // let super_arg: Vec<_> = std::env::args().collect();

    args.remove(0);
    // define command line arguments.
    let api_versions = get_api_versions();
    let versions: Vec<&str> = api_versions.lines().collect();

    let mut map: HashMap<String, String>; // = HashMap::new();
    let mut version_from_cli = Some(String::from("latest"));
    let mut all_info = false;
    // check api version
    // in order to know which YAML generate
    let mut xml: bool = false;
    if args.contains(&"--xml".to_string()) {
        xml = true;
        let index = args.iter().position(|r| r == "--xml").unwrap();
        args.remove(index);
    }
    if args.contains(&"--api".to_string()) {
        if args.len() > 2 {
            let index = args.iter().position(|r| r == "--api").unwrap();
            Some(args.remove(index));
            version_from_cli = Some(args.swap_remove(index));
            // TODO: check version is a valid version with api_versions
        }
    }
    if args.contains(&"-h".to_string()) {
        let index = args.iter().position(|r| r == "-h").unwrap();
        Some(args.remove(index));
    }

    if !args.is_empty() {
        map = get_args_from_framework(&version_from_cli.clone().unwrap(), &mut args, all_info);
        if args.is_empty() {
            display(map, xml, all_info);
        } else {
            // Some params were not found
            // Create App with ALL options
            all_info = true;
            map = get_args_from_framework(&version_from_cli.unwrap(), &mut args, all_info);
            let mut all_options: Vec<Arg> = Vec::with_capacity(map.len());
            let mut my_index = 0;
            for param in map.keys() {
                all_options.insert(
                    my_index,
                    Arg::with_name(&**param)
                        .long(param)
                        //.allow_hyphen_values(true)
                        .help("Get {param} from metadata")
                        .takes_value(false)
                        .required(false),
                );
                my_index += 1;
            }
            let mut my_app = App::new("metadata")
                .version("0.1.0")
                .author("Jesus")
                .args(all_options);
            let maxi_help = my_app.print_help();
            let my_message: Vec<&str> = args.iter().map(String::as_str).collect();
            let pert = format!(
                "{}{} {}\n{}\n",
                "Found argument ",
                &my_message.concat(),
                "which wasn't expected, or isn't valid in this context.",
                "Please, check help above"
            );
            match maxi_help {
                Ok(_helpa) => {
                    Error::with_description(pert, clap::ErrorKind::UnknownArgument).exit()
                }
                Err(e) => println!("PUES ERROR {e:?}"),
            }
        }
    } else {
        // metadata without params
        //e.g. ec2metadata or ec2metadata --api <version>
        // use user version for API if any or latest
        // args.append(&mut vec![String::from("--api"), String::from("latest")]);
        all_info = true;
        map = get_args_from_framework(&version_from_cli.unwrap(), &mut args, all_info);
        display(map, xml, all_info);
    }
    println!("Hello, world!");
}
