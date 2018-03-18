extern crate rusoto_core;
extern crate rusoto_ec2;
use rusoto_core::{ProfileProvider, Region};
use rusoto_core::reactor::RequestDispatcher;
use rusoto_ec2::{DescribeInstancesRequest, DescribeInstancesResult, Ec2, Ec2Client, Instance,
                 Reservation};

extern crate docopt;
extern crate rustc_serialize;
use docopt::Docopt;

extern crate futures;
use futures::future::Future;

use std::error::Error;
use std::str::FromStr;

struct InstanceStatus {
    id: String,
    ip_address: String,
}

impl InstanceStatus {
    fn from_instance(instance: &Instance) -> InstanceStatus {
        InstanceStatus {
            id: instance.instance_id.clone().unwrap_or("(unknown)".to_owned()),
            ip_address: instance.public_ip_address.clone().unwrap_or("".to_owned()),
        }
    }
}

impl std::fmt::Display for InstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.id, self.ip_address)
    }
}

const USAGE: &'static str = "
Usage:
    list-ec2-instances
    list-ec2-instances [--profile=<profile>] [--region=<region>]
    list-ec2-instances --help

Options:
    --profile=<profile>  Use a specific profile from your credential file.
    --region=<region>    Specify a region.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
    flag_profile: Option<String>,
    flag_region: Option<String>,
}

fn parse_args() -> Args {
    Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit())
}

fn find_provider(args: &Args) -> Result<ProfileProvider, String> {
    match ProfileProvider::new() {
        Ok(mut provider) => {
            if let Some(ref name) = args.flag_profile {
                provider.set_profile(name.to_string());
            }
            Ok(provider)
        }
        Err(_) => Err(String::from("Failed to find provider")),
    }
}

fn find_region(args: &Args) -> Result<Region, String> {
    match args.flag_region {
        Some(ref value) => Region::from_str(value).or_else(|e| Err(String::from(e.description()))),
        None => Ok(Region::ApNortheast1),
    }
}

fn show_instances(is: &Vec<Instance>) {
    for i in is {
        println!("{}", InstanceStatus::from_instance(i));
    }
}

fn show_reservations(rs: &Vec<Reservation>) {
    for r in rs {
        r.instances.as_ref().map(|is| show_instances(&is));
    }
}

fn show_result(result: DescribeInstancesResult) {
    result.reservations.map(|rs| show_reservations(&rs));
}

fn main() {
    let args: Args = parse_args();

    let provider = find_provider(&args).unwrap();
    let region = find_region(&args).unwrap();
    let client = Ec2Client::new(RequestDispatcher::default(), provider, region);
    let request = DescribeInstancesRequest::default();

    let _ = client
        .describe_instances(&request)
        .map(|result| show_result(result))
        .map_err(|e| eprintln!("Error: {}", e))
        .wait();
}
