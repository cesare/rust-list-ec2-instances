extern crate getopts;
extern crate rusoto_core;
extern crate rusoto_ec2;

use std::default::Default;
use std::env;
use std::fmt;

use getopts::Options;
use rusoto_core::Region;
use rusoto_ec2::{Filter, Ec2, Ec2Client, DescribeInstancesRequest, Instance, Reservation};

#[derive(Debug)]
struct Args {
    name_pattern: Option<String>,
}

struct InstanceSummary {
    instance: Instance,
}

impl fmt::Display for InstanceSummary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let missing_value = || "-".to_string();
        let public_ip_address = self.find_public_ip_address().unwrap_or_else(missing_value);
        let name = self.find_name_in_tags().unwrap_or_else(missing_value);
        let id = self.find_instance_id().unwrap_or_else(missing_value);
        write!(f, "{}\t{}\t{}", id, name, public_ip_address)
    }
}

impl InstanceSummary {
    fn find_name_in_tags(&self) -> Option<String> {
        self.instance.tags.as_ref()
            .and_then(|ts| ts.iter().find(|&tag| tag.key == Some("Name".to_string())))
            .and_then(|tag| tag.value.as_ref().and_then(|ref v| Some(v.to_string())))
    }

    fn find_public_ip_address(&self) -> Option<String> {
        self.instance.public_ip_address.as_ref().map(|s| s.to_string())
    }

    fn find_instance_id(&self) -> Option<String> {
        self.instance.instance_id.as_ref().map(|s| s.to_string())
    }
}


fn show_instances(is: &Vec<Instance>) {
    for i in is {
        let summary = InstanceSummary { instance: i.to_owned() };
        println!("{}", summary);
    }
}

fn show_reservations(rs: &Vec<Reservation>) {
    for r in rs {
        r.instances.as_ref().map(|is| show_instances(&is));
    }
}

fn create_running_instance_filter() -> Filter {
    Filter {
        name: Some("instance-state-name".to_string()),
        values: Some(vec!["running".to_string()])
    }
}

fn create_tag_name_filter(pattern: &String) -> Filter {
    let name_pattern = format!("*{}*", pattern);
     Filter {
        name: Some("tag:Name".to_string()),
        values : Some(vec![name_pattern]),
    }
}

fn create_filters(args: &Args) -> Option<Vec<Filter>> {
    let mut filters = vec![
        create_running_instance_filter(),
    ];

    if let Some(ref pattern) = args.name_pattern {
        filters.push(create_tag_name_filter(pattern));
    }
    Some(filters)
}

fn create_request(args: &Args) -> DescribeInstancesRequest {
    DescribeInstancesRequest {
        filters: create_filters(args),
        ..Default::default()
    }
}

fn print_usage(program: &str, options: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", options.usage(&brief));
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();

    let mut options = Options::new();
    options.optopt("n", "name-pattern", "specify pattern of instance name", "PATTERN");
    options.optflag("h", "help", "print usage");

    let matches = options.parse(&args[1..]).unwrap_or_else(|f| panic!(f.to_string()));
    if matches.opt_present("h") {
        let program = args[0].clone();
        print_usage(&program, options);
        std::process::exit(0);
    }

    Args {
        name_pattern: matches.opt_str("n"),
    }
}

fn main() {
    let args = parse_args();
    let client = Ec2Client::new(Region::ApNortheast1);
    let request = create_request(&args);

    let _result = client.describe_instances(request).sync()
        .map(|result| result.reservations.map(|rs| show_reservations(&rs)))
        .map_err(|error| eprintln!("Error: {:?}", error));
}
